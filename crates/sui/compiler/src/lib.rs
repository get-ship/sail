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
//!     mode: CompileMode::View,
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
pub mod render;
pub(crate) mod scan;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

pub use diagnostics::{CompileError, DiagSeverity, Diagnostic, Span};
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

/// Compile mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompileMode {
    /// Auto-detect: picks `Component` when `component_meta` is set, otherwise `View`.
    Auto,
    View,
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
    /// Emit a `.ssr.html` static SSR snapshot alongside the JS module
    /// (view mode only). Defaults to `true`. Pass `false` (via `--no-ssr`)
    /// to suppress, e.g. for island bundles that have no static skeleton.
    pub emit_ssr: bool,
    /// Emit a side-effect `import "@sui-tag/<tag>";` for every custom-element
    /// tag the template uses, so components enter the module's import graph and
    /// the bundler can tree-shake exactly what a view needs — the per-view
    /// resolver that replaces the umbrella/islands registry. Pure: the
    /// specifier is a function of the tag; the bundler's `resolve_id` maps
    /// `@sui-tag/<tag>` to the component source. Defaults to `false` — the
    /// registry path is unaffected until the bundler opts in.
    pub emit_component_imports: bool,
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
            emit_ssr: true,
            emit_component_imports: false,
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
    /// Static SSR HTML body fragment, present when `CompileOpts::emit_ssr` is
    /// true and mode is `View`. Written to the project mirror tree as
    /// `<repo-root>/.ship/sui-out/<src-rel>/<stem>.ssr.html` by the CLI.
    pub ssr_html: Option<String>,
}

/// Check a `.sui` source string and return **every** diagnostic, not just
/// the first. This is the entry point for the LSP + agent tooling — it
/// parses, runs all check passes collecting (rather than short-circuiting
/// like [`compile`]), and merges the full `sui-lint` findings.
///
/// A fatal parse error returns a single diagnostic (nothing else can run
/// without an AST). `source_path` locates the paired `.controller.ts` for
/// the controller-aware passes; pass `None` for an unsaved/synthetic buffer.
pub fn check_all(source: &str, source_path: Option<&std::path::Path>) -> Vec<Diagnostic> {
    match parse::parse(source) {
        Ok(program) => check::collect(&program, source, source_path),
        Err(e) => vec![Diagnostic::from_compile_error(&e, source)],
    }
}

/// Output of [`render_view`]: the split body/scope plus the declared
/// `<client>` names. Bundled together so a host (deck-core's view-payload
/// serve path) gets everything one request needs from a single call instead
/// of reaching into the compiler's `parse` / `bindings` / `render`
/// sub-modules itself.
#[derive(Debug, Clone)]
pub struct RenderView {
    /// Body HTML + initial scope, returned separately (see
    /// [`render::RenderSplit`]). `__subscribe__` is NOT injected — the host
    /// reads `split.has_live_data` and folds the served URL in with its own
    /// route builders.
    pub split: render::RenderSplit,
    /// Declared `<client>` names, in source order.
    pub clients: Vec<String>,
}

/// Render a `.sui` view source to a [`RenderView`] — the
/// `parse::parse → bindings::build → render::render_split` chain packaged
/// behind one public entry point.
///
/// This is the serve-path counterpart to [`compile`] (which emits the ESM
/// module): it produces the first-paint split + the client list a host ships
/// in its view payload. Hosts call this instead of touching the internal
/// sub-modules. Serve concerns stay on the host side — `__subscribe__`
/// injection (via [`render::RenderSplit::has_live_data`]) and any
/// mtime-gated caching are the host's, not the compiler's.
pub fn render_view(
    source: &str,
    opts: &render::RenderOpts,
) -> Result<RenderView, CompileError> {
    let program = parse::parse(source)?;
    let registry = bindings::build(&program);
    let split = render::render_split(&program, &registry, opts)?;
    let clients = program.clients.iter().map(|c| c.name.clone()).collect();
    Ok(RenderView { split, clients })
}

/// Compile a `.sui` source string.
pub fn compile(source: &str, opts: CompileOpts) -> Result<CompileOutput, CompileError> {
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

    let mode = match opts.mode {
        CompileMode::Auto => {
            if opts.component_meta.is_some() {
                CompileMode::Component
            } else {
                CompileMode::View
            }
        }
        m => m,
    };

    let program = parse::parse(effective_source).map_err(|e| attach_path(e, &opts))?;
    check::run(&program, effective_source, opts.source_path.as_deref())
        .map_err(|e| attach_path(e, &opts))?;
    // One contract per boundary (#255): the full Error-severity lint stream
    // (sui-prop-value, sui-attr-bool-prop, sui-css-content-text, controller-*)
    // gates `compile()` itself, so the daemon + napi harness reject exactly
    // what the CLI does. The CLI's post-compile `fatal_diagnostics_gate` is now
    // redundant-but-harmless.
    check::gate(&program, effective_source, opts.source_path.as_deref())
        .map_err(|e| attach_path(e, &opts))?;
    let bindings = bindings::build(&program);
    let lowered = lower::lower(&program, mode);

    let js = emit::js::emit_module(&program, &lowered, &bindings, &opts, mode);
    let cem = if mode == CompileMode::Component {
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
    let ssr_html = if opts.emit_ssr && mode == CompileMode::View {
        let view_name = opts.source_path
            .as_ref()
            .and_then(|p| p.file_stem())
            .and_then(|s| s.to_str())
            .map(|s| s.trim_end_matches(".html").to_string())
            .unwrap_or_else(|| "view".to_string());
        Some(emit::ssr::emit_ssr_html(&program, &bindings, &view_name))
    } else {
        None
    };
    Ok(CompileOutput { js, css: None, cem, schema, dts, meta, ssr_html })
}

fn attach_path(err: CompileError, opts: &CompileOpts) -> CompileError {
    match &opts.source_path {
        Some(p) => err.with_path(p.clone()),
        None => err,
    }
}
