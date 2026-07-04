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
//!     styles: None,
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
pub mod check;
pub mod diagnostics;
pub mod emit;
pub mod ir;
pub mod islands;
pub mod keys;
pub mod lower;
pub mod out_layout;
pub mod parse;
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
    /// External CSS to prepend as a `<style>` block before parsing. Only
    /// valid for component sources (`component_meta` must be set or `mode`
    /// must be `Component`); otherwise `compile()` errors.
    pub styles: Option<String>,
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
            styles: None,
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
    match parse::parse(source) {
        Ok(program) => check::collect(&program, source, source_path, overrides),
        Err(e) => vec![Diagnostic::from_compile_error(&e, source)],
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

pub fn compile(source: &str, opts: CompileOpts) -> Result<CompileOutput, CompileError> {
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
    let merged_owned;
    let effective_source: &str = if let Some(css) = opts.styles.as_deref() {
        let is_component =
            opts.component_meta.is_some() || opts.mode == CompileMode::Component;
        if !is_component {
            return Err(CompileError::new(
                "--styles is only valid for component sources",
            ));
        }
        merged_owned = format!("<style>{css}</style>\n{source}");
        &merged_owned
    } else {
        source
    };

    // The --styles mechanism prepends a COMPILER-INJECTED <style> block (a
    // component's external CSS). That block is not author-written template
    // content, so it must not trip the `sui-no-style-block` floor rule — which
    // governs authored <style>. Suppress the rule for this compile when we
    // injected styles. (--styles is itself a pre-floor component-CSS path the
    // floor will eventually retire; the rule still fires on authored blocks.)
    let effective_overrides: Vec<check::lint_overrides::LintOverride> =
        if opts.styles.is_some() {
            let mut o = opts.lint_overrides.clone();
            // Prepend, not append: `apply_overrides` is first-match-wins, and
            // the resolved policy may already carry a `no-style-block → Error`
            // promotion. The injected-styles exemption must take precedence.
            o.insert(
                0,
                check::lint_overrides::LintOverride {
                    rule: "sui-no-style-block".to_string(),
                    severity: check::lint_overrides::OverrideSeverity::Off,
                },
            );
            o
        } else {
            opts.lint_overrides.clone()
        };

    let program = parse::parse(effective_source).map_err(|e| attach_path(e, &opts))?;
    check::run(
        &program,
        effective_source,
        opts.source_path.as_deref(),
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
        &effective_overrides,
    )
    .map_err(|e| attach_path(e, &opts))?;
    let bindings = bindings::build(&program);
    // `is_view` is the master view/component switch in lowering (which runtime
    // factory to import, direct-emit hydration, route detection). A source is a
    // view iff it is tagless (no `component_meta`); a tag makes it a component.
    let lowered = lower::lower(&program, opts.component_meta.is_none());

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
        // Re-read the paired controller for d.ts emission. Cheap — it's
        // a small file read; the parsed shape is identical to the one
        // `check::run` consumed above, so behaviour stays consistent.
        let controller = check::controller_parse::read_paired_controller(opts.source_path.as_deref());
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
