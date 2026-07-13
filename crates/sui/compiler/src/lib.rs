//! `sui-compiler` — ahead-of-time compiler for `.sui` files.
//!
//! Same semantics as the Phase-2 interpreter
//! (`packages/sui/dom`); everything that can be resolved at build
//! time is. The emitted ESM module imports from `@ship/sui-runtime`
//! and constructs the rendered tree directly, with no parser code in
//! the runtime payload.
//!
//! # Public API
//!
//! ```ignore
//! use sui_compiler::{compile, CompileOpts, CompileMode, ComponentMeta};
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
//! See `crates/sui/compiler/src/parse/` for the parse pipeline,
//! `src/lower/` for IR lowering, and `src/emit/` for JS / CEM
//! generation. Diagnostics carry byte spans pointing at the user's
//! source — see `src/diagnostics.rs`.

use std::path::PathBuf;

pub mod bindings;
pub mod analysis;
pub mod animation;
pub mod check;
pub mod diagnostics;
pub mod emit;
pub mod ir;
pub mod islands;
pub mod keys;
pub mod lower;
pub mod out_layout;
pub mod parse;
pub mod stdlib;
pub mod escape;
pub mod view_init;
pub use view_init::{view_init, ViewInit};
pub(crate) mod scan;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

pub use diagnostics::{CompileError, DiagSeverity, Diagnostic, Span};
pub use check::lint_overrides::{LintOverride, OverrideSeverity};
pub use check::policy::Policy;
pub use analysis::{analyze, completions_at, hover_at, Analysis, Hover, Symbol};

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

/// Compile mode. Every `.sui` compiles to a component; `Auto` resolves to
/// `Component`. The retired `View` mode (static SSR HTML + a hydration island)
/// is gone — SSR is the host's job, run on the same component via ship-js.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CompileMode {
    #[default]
    Auto,
    Component,
}

/// Tag name and shadow-root flag for a component file. Passed in from the
/// build system (read from the per-component `.sui.json` metadata file) so
/// the template source no longer needs a `<component>` wrapper element.
#[derive(Debug, Clone)]
pub struct ComponentMeta {
    pub name: String,
    pub shadow: bool,
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
    /// Emit a `.sui.d.ts` declaration file alongside the JS module.
    /// Defaults to `true`. Pass `false` (via `--no-dts`) to suppress.
    pub emit_dts: bool,
    /// Emit a side-effect `import "@sui-tag/<tag>";` for every custom-element
    /// tag the template uses, so components enter the module's import graph and
    /// the bundler can tree-shake exactly what a view needs — the per-view
    /// resolver that replaces the umbrella/islands registry. Pure: the
    /// specifier is a function of the tag; the bundler's `resolve_id` maps
    /// `@sui-tag/<tag>` to the component source. Defaults to `false` — the
    /// registry path is unaffected until the bundler opts in.
    pub emit_component_imports: bool,
    /// Per-rule severity overrides applied at the build gate. The
    /// lint engine emits findings at the rule's natural severity
    /// (Error for the typo-catching rules, Warn for surfaced-but-
    /// non-blocking rules); this list downgrades specific rules'
    /// Error findings to Warning (or drops them) so an external
    /// SUI consumer can opt out of a strict rule without editing
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
    ///     .with_lint_override("sui-css-token", OverrideSeverity::Off);
    /// ```
    pub fn with_lint_override(
        mut self,
        rule: impl Into<String>,
        severity: check::lint_overrides::OverrideSeverity,
    ) -> Self {
        self.lint_overrides.push(check::lint_overrides::LintOverride {
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
            emit_dts: true,
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
    /// Emitted `.sui.d.ts` text, present when `CompileOpts::emit_dts` is true.
    pub dts: Option<String>,
    /// `.sui.meta.json` sidecar — binding info (kinds/exprs/deps/renderKey)
    /// for LSP/IDE consumption. The runtime never loads this; it's purely
    /// developer-tooling fuel.
    pub meta: Option<serde_json::Value>,
}

/// Check a `.sui` source string and return **every** diagnostic, not just
/// the first. This is the entry point for the LSP + agent tooling — it
/// parses, runs all check passes collecting (rather than short-circuiting
/// like [`compile`]), and merges the full `sui-lint` findings.
///
/// A fatal parse error returns a single diagnostic (nothing else can run
/// without an AST). `source_path` locates the paired `.controller.ts` for
/// the controller-aware passes; pass `None` for an unsaved/synthetic buffer.
pub fn check_all(
    source: &str,
    source_path: Option<&std::path::Path>,
    overrides: &[LintOverride],
) -> Vec<Diagnostic> {
    // SFC: strip the inline `<controller>` block (else it parses as a template)
    // and resolve the controller (inline body wins, else paired file) for the
    // scope-aware passes — mirrors `compile()`.
    let (stripped, inline_controller_src) = parse::preprocess::extract_controller_block(source);
    // ONE controller: the inline `<controller>` block. A sibling
    // `<stem>.controller.ts` is a compile error (`sui-paired-controller`), not
    // a fallback.
    let controller = inline_controller_src.and_then(|body| {
        check::controller_parse::parse(
            source_path.map(|p| p.to_path_buf()).unwrap_or_default(),
            body,
        )
    });
    match parse::parse(&stripped) {
        Ok(program) => {
            check::collect(&program, &stripped, source_path, controller.as_ref(), overrides)
        }
        Err(e) => vec![Diagnostic::from_compile_error(&e, &stripped)],
    }
}

/// Compile a `.sui` source string.
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

/// ELEMENT MODE — compile a `.sui` source as an L1 language element (one
/// authoring technology per layer; see `emit::element`). Same parse pipeline
/// as `compile()` — preprocess (sx/variants/component-block lowering) then
/// parse — but the outputs are the element artifacts: a static structure
/// snippet for use-site lowering, framework-sheet skin CSS keyed on
/// `.sui-<name>`, and the element manifest. The mode enforces the seam test:
/// control flow, dynamic attributes, `<data>`/`<client>`, or a controller
/// are compile errors naming the source a component.
pub fn compile_element(
    source: &str,
    name: &str,
) -> Result<emit::element::ElementOutput, CompileError> {
    // An L1 element name is lowercase, and it may be hyphenated — `tab-list`,
    // `menu-item` and `context-menu` are all language elements. What it may NOT be
    // is `sui-`-prefixed: that is the legacy COMPONENT namespace (the custom
    // elements L1 replaces), and the compiler stamps its own `sui-` prefix onto the
    // root class. Passing one through yields `.sui-sui-spinner` — nonsense CSS for a
    // source that should never have compiled as an element. (Caught by a test the
    // aborting suite had been hiding.)
    if let Some(rest) = name.strip_prefix("sui-") {
        return Err(CompileError::new(format!(
            "element name `{name}` is in the `sui-` component namespace — an L1 element is a              bare lowercase name (`{rest}`), and the compiler adds the `sui-` prefix to its              root class itself. A `sui-`-prefixed tag is a custom element, i.e. a component."
        )));
    }
    if name.is_empty() || !name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        return Err(CompileError::new(format!(
            "element name `{name}` must be a bare lowercase name (hyphens allowed: `tab-list`)."
        )));
    }
    // parse_element_source: the pipeline minus the stdlib-element pass — an
    // element source may not use stdlib elements (no recursion).
    let program = parse::parse_element_source(source)?;
    emit::element::emit_element(&program, name)
}

pub fn compile(source: &str, mut opts: CompileOpts) -> Result<CompileOutput, CompileError> {
    // Light DOM is the default architecture (invariant 9): components render
    // into their own element, so native events from composed L1 controls bubble
    // to the host with no boundary. Shadow DOM is a per-component containment
    // FEATURE declared in the component's own manifest (`<component>{ shadow:
    // true }`) for wrapping foreign content (tiptap-class editors, FFI). The
    // SOURCE is authoritative for every caller — bundler, napi, CLI.
    if let Some(meta) = opts.component_meta.as_mut() {
        meta.shadow = parse::preprocess::declares_shadow_dom(source);
    }
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
        )));
    }
    // (--styles is gone — complete cutover: the pre-floor external-CSS
    // injection path had zero producers. Element skins are the sanctioned
    // toolchain-CSS home now.)
    //
    // SFC: strip an inline `<controller>` block before parsing so its body
    // isn't misread as a template expression. This makes SFC first-class in
    // core `compile()` — every caller (bin, deck, tests) handles it uniformly,
    // not just the CLI. The controller module itself is materialized by the
    // file-emitting callers (the bin writes `<stem>.controller.ts`); source
    // that was already stripped (the bin pre-strips) is a no-op here.
    let (stripped_source, inline_controller_src) =
        parse::preprocess::extract_controller_block(source);
    let effective_source: &str = &stripped_source;
    // Resolve the controller for the scope-aware checks in-memory: the inline
    // `<controller>` block (SFC) wins; else the paired `.controller.ts` on disk
    // (views / not-yet-migrated). No filesystem round-trip for the inline case.
    let controller = inline_controller_src.and_then(|body| {
        check::controller_parse::parse(opts.source_path.clone().unwrap_or_default(), body)
    });
    let effective_overrides = opts.lint_overrides.clone();

    let program = parse::parse(effective_source).map_err(|e| attach_path(e, &opts))?;
    check::run(
        &program,
        effective_source,
        opts.source_path.as_deref(),
        controller.as_ref(),
        &effective_overrides,
    )
    .map_err(|e| attach_path(e, &opts))?;
    // One contract per boundary (#255): the full Error-severity lint stream
    // (sui-prop-value, sui-attr-bool-prop, sui-css-content-text, controller-*)
    // gates `compile()` itself, so the daemon + napi harness reject exactly
    // what the CLI does. The CLI's post-compile `fatal_diagnostics_gate` is now
    // redundant-but-harmless.
    check::gate(
        &program,
        effective_source,
        opts.source_path.as_deref(),
        controller.as_ref(),
        &effective_overrides,
    )
    .map_err(|e| attach_path(e, &opts))?;
    let bindings = bindings::build(&program);
    // `is_view` is the master view/component switch in lowering (which runtime
    // factory to import, direct-emit hydration, route detection). A source is a
    // view iff it is tagless (no `component_meta`); a tag makes it a component.
    let lowered = lower::lower(&program, opts.component_meta.is_none()).map_err(|e| attach_path(e, &opts))?;

    let js = emit::js::emit_module(&program, &lowered, &bindings, &opts);
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
        Some(emit::dts::emit_dts(&program, controller.as_ref()))
    } else {
        None
    };
    let meta = Some(emit::meta::emit_meta(&program, &bindings));
    Ok(CompileOutput { js, css: None, cem, schema, dts, meta })
}

fn attach_path(err: CompileError, opts: &CompileOpts) -> CompileError {
    match &opts.source_path {
        Some(p) => err.with_path(p.clone()),
        None => err,
    }
}

#[cfg(test)]
mod name_validation_tests {
    use super::*;

    #[test]
    fn rejects_names_that_would_throw_at_define() {
        assert_eq!(validate_custom_element_name("dashboard"), Err("it has no hyphen"));
        assert_eq!(validate_custom_element_name(""), Err("it is empty"));
        assert_eq!(validate_custom_element_name("Dash-Panel"), Err("it must start with a lowercase letter"));
        assert_eq!(validate_custom_element_name("dash-Panel"), Err("it must not contain uppercase letters"));
        assert_eq!(validate_custom_element_name("1-panel"), Err("it must start with a lowercase letter"));
        assert_eq!(validate_custom_element_name("dash-pan!el"), Err("it contains an invalid character"));
    }

    #[test]
    fn accepts_valid_tags() {
        assert!(validate_custom_element_name("dash-panel").is_ok());
        assert!(validate_custom_element_name("sui-trading-journal").is_ok());
        assert!(validate_custom_element_name("x-1").is_ok());
    }

    #[test]
    fn compile_rejects_a_hyphenless_component_name() {
        let opts = CompileOpts {
            component_meta: Some(ComponentMeta { name: "dashboard".into(), shadow: true }),
            ..CompileOpts::default()
        };
        let err = compile("<column><p>x</p></column>", opts).unwrap_err();
        assert!(format!("{err:?}").contains("no hyphen"), "got: {err:?}");
    }
}
