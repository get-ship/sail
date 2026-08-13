//! `sail-compiler` — ahead-of-time compiler for `.sail` files.
//!
//! Same semantics as the Phase-2 interpreter
//! (`packages/sail/dom`); everything that can be resolved at build
//! time is. The emitted ESM module imports from `@ship/sail-runtime`
//! and constructs the rendered tree directly, with no parser code in
//! the runtime payload.
//!
//! # Public API
//!
//! ```ignore
//! use sail_compiler::{compile, CompileOpts, CompileMode, ComponentMeta};
//!
//! let out = compile(source, CompileOpts {
//!     setup_url: Some("./view.ts".into()),
//!     mode: CompileMode::Component,
//!     source_path: None,
//!     component_meta: None,
//!     emit_dts: true,
//! })?;
//! println!("{}", out.js);
//! println!("{:?}", out.dts);
//! ```
//!
//! See `crates/sail/compiler/src/parse/` for the parse pipeline,
//! `src/lower/` for IR lowering, and `src/emit/` for JS / CEM
//! generation. Diagnostics carry byte spans pointing at the user's
//! source — see `src/diagnostics.rs`.

use std::path::PathBuf;

pub mod action_manifest;
pub mod ambients;
pub mod analysis;
pub mod animation;
pub mod authoring_bans;
pub mod bindings;
pub mod block_manifest;
pub mod breakpoints;
pub mod check;
pub mod diag_ratchet;
pub mod diagnostics;
pub mod directive_manifest;
pub mod edit;
pub mod emit;
pub mod escape;
pub mod fmt;
pub mod grammar_vocabulary;
pub mod icons;
pub mod ir;
pub mod keys;
pub mod l1_manifest;
pub mod lower;
pub mod out_layout;
pub mod parse;
pub mod policy;
#[cfg(not(target_arch = "wasm32"))]
pub mod profile;
pub mod project_map;
pub mod region_capability;
pub mod runtime_ambient;
pub(crate) mod scan;
pub mod scope_manifest;
pub mod semantic_props;
pub mod stdlib;
pub mod theme_gate;
pub mod theme_surface;
pub mod tokens_generate;
pub mod used_icons;
pub mod used_tags;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

pub use analysis::{
    Analysis, Hover, QueryKeyInfo, SsrTier, Symbol, analyze, analyze_with_tier, classify_ssr_tier,
    completions_at, completions_at_offset, extract_query_keys, hover_at,
};
pub use authoring_bans::{AUTHORING_BANS, AuthoringBan};
pub use check::lint_overrides::{LintOverride, OverrideSeverity};
pub use check::policy::Policy;
pub use diagnostics::{CompileError, DiagSeverity, Diagnostic, DiagnosticTier, Span};
pub use fmt::{
    FormatError, FormatOpts, FormatResult, format_source, format_source_with, is_formatted,
};
pub use grammar_vocabulary::{GrammarVocabulary, to_json as grammar_vocabulary_to_json};
pub use icons::{
    DEFAULT_PACK, ICONIFY_CATALOG, IconConfig, PackCatalogEntry, PackRef, PackSurface,
    RULE_ICON_GLYPH, RULE_UNKNOWN_ICON, RoleBinding, RoleRegistry, RoleSurface, UsedGlyph,
    collect_static_roles, load_glyph_svg, pack_glyph_ids, parse_icon_config, resolve_used_glyphs,
    unknown_role_message,
};
pub use l1_manifest::{
    L1Attribute, L1ComponentToken, L1Entry, L1Js, L1JsKind, L1Kind, L1SlotType, VOID_TAGS,
    l1_by_kind, l1_count, l1_entries, l1_kind,
};
pub use region_capability::{
    CONTROLLER_SIZE_THRESHOLD, REGION_CAPABILITY_MATRIX, RegionCapability,
};
pub use semantic_props::{
    HOST_ANATOMY, HostAnatomy, SEMANTIC_PROPS, SemanticKind, SemanticProp, host_anatomy,
    is_free_aria_attr, lookup as lookup_semantic_prop, lower_attr_name_str,
};
pub use theme_surface::{
    ThemeTokenHover, ThemeTokenKind, ThemeTokenRef, catalog_theme_ids, infer_active_theme_mode,
    resolve_theme_token, theme_attr_completions, theme_attr_partial, theme_complete_at,
    theme_hover_at, theme_override_vocabulary,
};
pub use used_icons::{collect_used_icons, collect_used_icons_from_source};
/// Identity stamp embedded in every emitted artifact (JS leading
/// comment, CEM, schema, meta sidecar) so the daemon and runtime can
/// detect drift between the compiled bundle and the running shell —
/// e.g. a stale `view.js` that lacks an export the runtime expects.
///
/// Cargo version is the crate's `version` field; git sha + dirty flag
/// + build timestamp come from `build.rs` via the shared `build_info`
///   crate. On source-tarball builds without `.git`, `git` is the
///   literal string `"unknown"`.
#[derive(Debug, Clone, Copy)]
pub struct CompilerVersion {
    pub cargo: &'static str,
    pub git: &'static str,
    pub dirty: bool,
    pub built_at: &'static str,
}

impl CompilerVersion {
    /// Serialise as the JSON shape consumers (JS comment, CEM,
    /// schema, meta, `/_deck/meta`) all share. Single source of truth.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "cargo": self.cargo,
            "git": self.git,
            "dirty": self.dirty,
            "builtAt": self.built_at,
        })
    }
}

/// Return the compiler's identity stamp. Cheap (no allocation —
/// every field is a `&'static str` resolved at compile time).
pub fn compiler_version() -> CompilerVersion {
    let bi = build_info::build_info!();
    CompilerVersion {
        cargo: env!("CARGO_PKG_VERSION"),
        git: bi.git_sha,
        dirty: bi.git_dirty,
        built_at: bi.timestamp,
    }
}

/// Compile mode. Every `.sail` compiles to a component; `Auto` resolves to
/// `Component`. The retired `View` mode (static SSR HTML + a hydration island)
/// is gone — SSR is the host's job, run on the same component via ship-js.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CompileMode {
    #[default]
    Auto,
    Component,
}

/// Tag name for a component file. Passed in from the build system so the
/// template source does not carry its own tag. Components are light DOM —
/// the shadow opt-in was removed with the `<scope>` redesign.
#[derive(Debug, Clone)]
pub struct ComponentMeta {
    pub name: String,
}
#[derive(Debug, Clone)]
pub struct CompileOpts {
    /// Path to the paired `.ts` setup module, relative to the emitted
    /// JS module's location. When `None`, no `import setup …` is emitted.
    pub setup_url: Option<String>,
    pub mode: CompileMode,
    /// For diagnostics. Optional.
    pub source_path: Option<PathBuf>,
    /// Component metadata (tag name + shadow flag). When set, `compile()`
    /// treats the source as a component file regardless of `mode`.
    pub component_meta: Option<ComponentMeta>,
    /// HOST GRANT: the route params for this file when the host's routing
    /// convention names it a route (deck routes by file location). `Some`
    /// legalizes + types `route` in the template and controller; `None`
    /// means "not a route" and any `route` read is refused. Structured
    /// params, passed IN — the sail compiler never sniffs the filesystem
    /// for a routing convention it does not own.
    pub route_params: Option<Vec<String>>,
    /// HOST GRANT: the capabilities the host puts on this file's scope —
    /// the app manifest's declared clients plus whatever context the host
    /// injects (deck merges its own `route`/`navigate` in). Emitted as typed
    /// members of the generated `Scope` (see `emit::dts::ScopeCapability`), so
    /// a controller reads `ctx.route` / `ctx.api` precisely and a name the app
    /// never declared is a type error. Empty by default: a file compiled with
    /// no host context gets no ambient members.
    pub scope_capabilities: Vec<emit::dts::ScopeCapability>,
    /// Emit a `.sail.d.ts` declaration file alongside the JS module.
    /// Defaults to `true`. Pass `false` (via `--no-dts`) to suppress.
    pub emit_dts: bool,
    /// Emit a side-effect `import "@sail-tag/<tag>";` for every custom-element
    /// tag the template uses, so components enter the module's import graph and
    /// the bundler can tree-shake exactly what a route needs — the per-route
    /// resolver that replaces the umbrella/islands registry. Pure: the
    /// specifier is a function of the tag; the bundler's `resolve_id` maps
    /// `@sail-tag/<tag>` to the component source. Defaults to `false` — the
    /// registry path is unaffected until the bundler opts in.
    pub emit_component_imports: bool,
    /// Per-rule severity overrides applied at the build gate. The
    /// lint engine emits findings at the rule's natural severity
    /// (Error for the typo-catching rules, Warn for surfaced-but-
    /// non-blocking rules); this list downgrades specific rules'
    /// Error findings to Warning (or drops them) so an external
    /// SAIL consumer can opt out of a strict rule without editing
    /// the rule itself. The internal monorepo leaves this empty
    /// (the default) and stays strict. See
    /// `crate::check::lint_overrides::LintOverride` for the shape.
    pub lint_overrides: Vec<check::lint_overrides::LintOverride>,
}

impl CompileOpts {
    /// Builder-style append: add a per-rule severity override and
    /// return `self` for chaining. A typical call site:
    ///
    /// ```ignore
    /// let opts = CompileOpts::default()
    ///     .with_lint_override("sail-css-token", OverrideSeverity::Off);
    /// ```
    pub fn with_lint_override(
        mut self,
        rule: impl Into<String>,
        severity: check::lint_overrides::OverrideSeverity,
    ) -> Self {
        self.lint_overrides
            .push(check::lint_overrides::LintOverride {
                rule: rule.into(),
                severity,
            });
        self
    }
}

impl Default for CompileOpts {
    fn default() -> Self {
        Self {
            setup_url: None,
            mode: CompileMode::Auto,
            source_path: None,
            component_meta: None,
            route_params: None,
            emit_dts: true,
            scope_capabilities: Vec::new(),
            emit_component_imports: false,
            lint_overrides: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompileOutput {
    pub js: String,
    pub css: Option<String>,
    pub cem: Option<serde_json::Value>,
    pub schema: Option<serde_json::Value>,
    /// Emitted `.sail.d.ts` text, present when `CompileOpts::emit_dts` is true.
    pub dts: Option<String>,
    /// `.sail.meta.json` sidecar — binding info (kinds/exprs/deps/renderKey)
    /// for LSP/IDE consumption. The runtime never loads this; it's purely
    /// developer-tooling fuel.
    pub meta: Option<serde_json::Value>,
    /// Static icon glyph names collected from the template (TOOL-10).
    pub used_icons: std::collections::BTreeSet<String>,
    /// The `collect()` stream from the Error gate. Empty of Error (those
    /// fail `compile()`); Warning/Fix stay so a host can print them
    /// without a second `collect()`.
    pub diagnostics: Vec<Diagnostic>,
}

/// Check a `.sail` source string and return **every** diagnostic, not just
/// the first. Full stream ([`DiagnosticTier::OnSave`]) — taffy layout +
/// compose included. Entry point for CLI / MCP / `sail_edit` / LSP OnSave.
///
/// Prefer [`check_all_with_tier`] for the LSP push path (OnType vs OnSave).
///
/// A fatal parse error returns a single diagnostic (nothing else can run
/// without an AST). `source_path` locates the paired `.controller.ts` for
/// the controller-aware passes; pass `None` for an unsaved/synthetic buffer.
pub fn check_all(
    source: &str,
    source_path: Option<&std::path::Path>,
    overrides: &[LintOverride],
    route_params: Option<&[String]>,
) -> Vec<Diagnostic> {
    check_all_with_tier(
        source,
        source_path,
        overrides,
        route_params,
        DiagnosticTier::OnSave,
    )
}

/// Like [`check_all`], with an explicit [`DiagnosticTier`] (SPEC-TOOLING-1
/// §WI6). [`DiagnosticTier::OnType`] skips taffy layout + compose.
pub fn check_all_with_tier(
    source: &str,
    source_path: Option<&std::path::Path>,
    overrides: &[LintOverride],
    route_params: Option<&[String]>,
    tier: DiagnosticTier,
) -> Vec<Diagnostic> {
    // SPEC-PROJECT-STRUCTURE-1 §4: block order is grammar — same law as
    // `compile()`, on the author source before stripping `<controller>`.
    if let Err(e) = parse::preprocess::check_sfc_block_order(source) {
        return vec![Diagnostic::from_compile_error(&e, source)];
    }
    // SFC: strip the inline `<controller>` block (else it parses as a template)
    // and resolve the controller (inline body wins, else paired file) for the
    // scope-aware passes — mirrors `compile()`.
    let (stripped, inline_controller_src) = parse::preprocess::extract_controller_block(source);
    // ONE controller: the inline `<controller>` block. A sibling
    // `<stem>.controller.ts` is a compile error (`sail-paired-controller`), not
    // a fallback.
    let controller = inline_controller_src.and_then(|body| {
        check::controller_parse::parse(
            source_path.map(|p| p.to_path_buf()).unwrap_or_default(),
            body,
        )
    });
    match parse::parse(&stripped) {
        Ok(program) => check::collect(
            &program,
            &stripped,
            source, // author source — type gate maps controller spans here
            source_path,
            controller.as_ref(),
            overrides,
            route_params,
            tier,
        ),
        Err(e) => {
            // Fatal parse/preprocess error — still surface confluent off-scale
            // spacing rewrites from raw author tags on the full (OnSave) path.
            // OnType stays parse-only for this fatal branch (no compose).
            let mut diags = vec![Diagnostic::from_compile_error(&e, &stripped)];
            if tier.includes_layout() {
                diags.extend(check::compose::collect_off_scale_for_check(
                    &stripped,
                    source_path,
                    overrides,
                ));
            }
            // Type gate still runs OnSave when parse failed after a controller
            // was present — DOM law does not depend on a successful template
            // parse.
            if matches!(tier, DiagnosticTier::OnSave) {
                diags.extend(check::tscheck::collect_sail_type_gate(source, source_path));
            }
            diags
        }
    }
}

/// Compile a `.sail` source string.
/// Validate a custom-element tag name — the subset whose violation makes
/// `customElements.define` throw: non-empty, contains a hyphen, lowercase ASCII
/// start, no uppercase, only `[a-z0-9._-]`. (The full spec's reserved-name and
/// PCEN rules are stricter, but these are the ones authors actually trip.)
fn validate_custom_element_name(name: &str) -> Result<(), &'static str> {
    if name.is_empty() {
        return Err("it is empty");
    }
    if !name.contains('-') {
        return Err("it has no hyphen");
    }
    if !name.as_bytes()[0].is_ascii_lowercase() {
        return Err("it must start with a lowercase letter");
    }
    if name.bytes().any(|b| b.is_ascii_uppercase()) {
        return Err("it must not contain uppercase letters");
    }
    if name
        .bytes()
        .any(|b| !(b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'-' | b'.' | b'_')))
    {
        return Err("it contains an invalid character");
    }
    Ok(())
}

/// ELEMENT MODE — compile a `.sail` source as an L1 language element (one
/// authoring technology per layer; see `emit::element`). Same parse pipeline
/// as `compile()` — preprocess then parse — but the outputs are the element
/// artifacts: a static structure snippet for use-site lowering, framework-sheet
/// skin CSS keyed on `.sail-<name>`, and the element manifest. The mode enforces
/// the seam test on the *template*: control flow or dynamic attributes are
/// compile errors naming the source a component. A `<controller>` block is an
/// L2-component facility — element mode REJECTS it: JS enhancement rides
/// stacked `use:<behavior>` stamps instead.
pub fn compile_element(
    source: &str,
    name: &str,
) -> Result<emit::element::ElementOutput, CompileError> {
    // An L1 element name is lowercase, and it may be hyphenated — `tab-list`,
    // `menu-item` and `context-menu` are all language elements. What it may NOT be
    // is `sail-`-prefixed: that is the legacy COMPONENT namespace (the custom
    // elements L1 replaces), and the compiler stamps its own `sail-` prefix onto the
    // root class. Passing one through yields `.sail-sail-spinner` — nonsense CSS for a
    // source that should never have compiled as an element. (Caught by a test the
    // aborting suite had been hiding.)
    if let Some(rest) = name.strip_prefix("sail-") {
        return Err(CompileError::new(format!(
            "element name `{name}` is in the `sail-` component namespace — an L1 element is a              bare lowercase name (`{rest}`), and the compiler adds the `sail-` prefix to its              root class itself. A `sail-`-prefixed tag is a custom element, i.e. a component."
        ))
        .with_category("sail-element-name"));
    }
    if name.is_empty()
        || !name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return Err(CompileError::new(format!(
            "element name `{name}` must be a bare lowercase name (hyphens allowed: `tab-list`)."
        ))
        .with_category("sail-element-name"));
    }
    // A `<controller>` block is the L2-component behavior channel; L1
    // elements carry behavior as stacked `use:<behavior>` stamps on their
    // static structure (lowered to `runAction` at the use site), so the
    // block is a targeted compile error here, not a captured module.
    let (stripped, controller_src) = parse::preprocess::extract_controller_block(source);
    if controller_src.is_some() {
        let at = source.find("<controller").unwrap_or(0);
        return Err(CompileError::new(
            "element mode: `<controller>` is not supported — attach behavior with `use:<behavior>`.",
        )
        .with_span(crate::diagnostics::Span::point(at))
        .with_category("sail-element-mode"));
    }
    // parse_element_source: the pipeline minus the stdlib-element pass — an
    // element source may not use stdlib elements (no recursion).
    // SPEC-D1: host known by construction → surface vocabulary for this host.
    // One regime: closed `bind(name)` resolution runs on stdlib/skins too.
    sail_lint::style::with_host_surface(Some(name), || {
        let program = parse::parse_element_source(&stripped)?;
        let bindings = check::binding::resolve(&stripped, &program, None)?;
        emit::element::emit_element_with_bindings(&program, &stripped, name, &bindings)
    })
}

pub fn compile(source: &str, opts: CompileOpts) -> Result<CompileOutput, CompileError> {
    // Light DOM is the architecture (invariant 9): components render into
    // their own element, so native events from composed L1 controls bubble
    // to the host with no boundary.
    // A tagged component registers via `customElements.define(name, …)`, which
    // throws at runtime unless `name` is a valid custom-element name. Reject it
    // at compile time (fail-loud) rather than emit code that explodes on load.
    if let Some(meta) = &opts.component_meta
        && let Err(why) = validate_custom_element_name(&meta.name)
    {
        return Err(CompileError::new(format!(
            "invalid component name `{}`: {why}. A custom-element tag must be lowercase, \
             start with a letter, and contain a hyphen — e.g. `dash-panel`, not `dashboard`.",
            meta.name
        ))
        .with_category("sail-component-name"));
    }
    // SPEC-PROJECT-STRUCTURE-1 §4: `<scope>` before `<controller>` is grammar.
    // Check on the author source before stripping the controller block.
    parse::preprocess::check_sfc_block_order(source).map_err(|e| attach_path(e, &opts))?;
    // SFC: strip an inline `<controller>` block before parsing so its body
    // isn't misread as a template expression. This makes SFC first-class in
    // core `compile()` — every caller (bin, deck, tests) handles it uniformly,
    // not just the CLI. The controller module itself is materialized by the
    // file-emitting callers (the bin writes `<stem>.controller.ts`); source
    // that was already stripped (the bin pre-strips) is a no-op here.
    let (stripped_source, inline_controller_src) = compile_span("sfc", || {
        parse::preprocess::extract_controller_block(source)
    });
    let effective_source: &str = &stripped_source;
    // Resolve the controller for the scope-aware checks in-memory: the inline
    // `<controller>` block (SFC) wins; else the paired `.controller.ts` on disk
    // (route modules / not-yet-migrated). No filesystem round-trip for the inline case.
    let controller = inline_controller_src.and_then(|body| {
        check::controller_parse::parse(opts.source_path.clone().unwrap_or_default(), body)
    });
    let effective_overrides = opts.lint_overrides.clone();

    let program = compile_span("parse-total", || parse::parse(effective_source))
        .map_err(|e| attach_path(e, &opts))?;
    // `<scope>` member sets are POSITION-SCOPED: the same input that names a
    // file (a tag = component; tagless = route/page) decides which members
    // are addable. A route receives params from the router, never props from
    // a parent — `props:` there is a modeling error, told now.
    if opts.component_meta.is_none() && !program.props.is_empty() {
        return Err(attach_path(
            CompileError::new(
                "`props:` declared in a route/page `<scope>` — a route has no \
                 parent to fill props; it receives route params from the router. \
                 Props are component members."
                    .to_string(),
            )
            .with_category("sail-route-props"),
            &opts,
        ));
    }
    compile_span("check-run", || {
        check::run(
            &program,
            effective_source,
            opts.source_path.as_deref(),
            controller.as_ref(),
            &effective_overrides,
            opts.route_params.as_deref(),
        )
    })
    .map_err(|e| attach_path(e, &opts))?;
    // One contract per boundary (#255): the Error-severity lint + layout +
    // type-gate stream gates `compile()` itself, so the daemon + napi harness
    // reject exactly what the CLI does. Compose is Fix by default and stays
    // on `check_all` / LSP OnSave / `sail_edit` — `gate` does not run it.
    let diagnostics = compile_span("gate", || {
        check::gate(
            &program,
            effective_source,
            source, // author source — type gate (JOB-E1) maps controller spans here
            opts.source_path.as_deref(),
            controller.as_ref(),
            &effective_overrides,
            opts.route_params.as_deref(),
        )
    })
    .map_err(|e| attach_path(e, &opts))?;
    // `run`/`gate` above already validated the `(--x)` resolution law
    // (fail-fast); re-resolving here is cheap (pure text scan) and gets us
    // the matched prop/signal bindings the emitter wires `@property` +
    // host reflection for.
    let sx_var_bindings = check::binding::resolve(effective_source, &program, controller.as_ref())
        .map_err(|e| attach_path(e, &opts))?;
    let bindings = bindings::build(&program);
    // `is_route` is the master route/component switch in lowering (which runtime
    // factory to import, direct-emit hydration, route detection). A source is a
    // route iff it is tagless (no `component_meta`); a tag makes it a component.
    let lowered = compile_span("lower", || {
        lower::lower(&program, opts.component_meta.is_none())
    })
    .map_err(|e| attach_path(e, &opts))?;

    let js = compile_span("emit", || {
        emit::js::emit_module(&program, &lowered, &bindings, &opts, &sx_var_bindings)
    });
    // CEM (custom-elements-manifest) describes a custom element, so it's emitted
    // only for tagged sources (components) — a tagless page has no element to
    // describe. Keyed on the tag, like the emit split itself.
    let cem = if opts.component_meta.is_some() {
        Some(emit::cem::emit_cem(&program, &opts))
    } else {
        None
    };
    let schema = Some(emit::schema::emit_schema(&program, &opts));
    let dts = if opts.emit_dts {
        // Reuse the inline controller resolved above — one source, no re-read,
        // no paired-file fallback.
        Some(emit::dts::emit_dts(
            &program,
            controller.as_ref(),
            &opts.scope_capabilities,
        ))
    } else {
        None
    };
    let meta = Some(emit::meta::emit_meta(&program, &bindings));
    Ok(CompileOutput {
        js,
        css: None,
        cem,
        schema,
        dts,
        meta,
        used_icons: lowered.used_icons,
        diagnostics,
    })
}

fn attach_path(err: CompileError, opts: &CompileOpts) -> CompileError {
    match &opts.source_path {
        Some(p) => err.with_path(p.clone()),
        None => err,
    }
}

fn compile_span<R>(phase: &'static str, f: impl FnOnce() -> R) -> R {
    #[cfg(not(target_arch = "wasm32"))]
    {
        crate::profile::span(phase, f)
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = phase;
        f()
    }
}

#[cfg(test)]
mod name_validation_tests {
    use super::*;

    #[test]
    fn rejects_names_that_would_throw_at_define() {
        assert_eq!(
            validate_custom_element_name("dashboard"),
            Err("it has no hyphen")
        );
        assert_eq!(validate_custom_element_name(""), Err("it is empty"));
        assert_eq!(
            validate_custom_element_name("Dash-Panel"),
            Err("it must start with a lowercase letter")
        );
        assert_eq!(
            validate_custom_element_name("dash-Panel"),
            Err("it must not contain uppercase letters")
        );
        assert_eq!(
            validate_custom_element_name("1-panel"),
            Err("it must start with a lowercase letter")
        );
        assert_eq!(
            validate_custom_element_name("dash-pan!el"),
            Err("it contains an invalid character")
        );
    }

    #[test]
    fn accepts_valid_tags() {
        assert!(validate_custom_element_name("dash-panel").is_ok());
        assert!(validate_custom_element_name("sail-trading-journal").is_ok());
        assert!(validate_custom_element_name("x-1").is_ok());
    }

    #[test]
    fn compile_rejects_a_hyphenless_component_name() {
        let opts = CompileOpts {
            component_meta: Some(ComponentMeta {
                name: "dashboard".into(),
            }),
            ..CompileOpts::default()
        };
        let err = compile("<column><p>x</p></column>", opts).unwrap_err();
        assert!(format!("{err:?}").contains("no hyphen"), "got: {err:?}");
    }
}
