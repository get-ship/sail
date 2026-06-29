//! sui-compiler integration for the Rust bundler — IN-PROCESS (#255).
//!
//! The bundler calls `sui_compiler::compile()` directly from its Rolldown
//! `load` hook. One toolchain: the compiler is the crate the bundler links, so
//! there is no separate `sui-compiler` binary to locate, no installed-vs-debug-
//! vs-release drift, and a flag the bundler passes can never be rejected by a
//! stale binary. `just build` produces a bundler carrying *the* current
//! compiler — build = works, no gate, no version dance.

use std::path::{Path, PathBuf};

use sui_compiler::emit::synthetic::{emit_synthetic_client_module, AppManifest, AuthoredController};
use sui_compiler::parse::parse;
use sui_compiler::{compile, CompileMode as SuiMode, CompileOpts, ComponentMeta};

/// A `.sui` compiles as a custom-element (`component`) or a view. Component mode
/// passes a tag name so the element is registered; view mode emits a route module.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompileMode {
    Component,
    View,
}

#[derive(Debug, thiserror::Error)]
pub enum CompileError {
    #[error("read {0}: {1}")]
    Read(PathBuf, #[source] std::io::Error),
    /// A rendered compiler diagnostic (with source location). Consumers
    /// stringify it (the HMR overlay, the bundle error); no structured fields.
    #[error("{0}")]
    Compile(String),
}

/// Compile one `.sui` to JS via `sui_compiler::compile` (in-process). Auto-wires
/// a sibling controller (`page.sui`→`controller.ts`, `<stem>.sui`→
/// `<stem>.controller.ts`) as `import setup from "./<controller>"`.
/// `emit_component_imports` opts the view into the per-view resolver — it emits
/// `@sui-tag/<tag>` registration imports the bundler resolves to source.
pub fn compile_sui(
    sui_file: &Path,
    mode: CompileMode,
    stem: &str,
    emit_component_imports: bool,
) -> Result<String, CompileError> {
    // Gated, scoped synthetic CLIENT emit (slice 1). When `SUI_CLIENT_EMIT=
    // synthetic` AND this is a VIEW with an `app.manifest.jsonc` above it, emit
    // the synthetic client contract (`default.setup` → `__mount` + `hydrate`)
    // instead of the production module. Default OFF and scoped to manifested
    // views, so every other target — and the flag-off case — is byte-for-byte
    // the production emit below.
    if mode == CompileMode::View && synthetic_client_enabled() {
        if let Some(js) = try_synthetic_client(sui_file)? {
            return Ok(js);
        }
    }
    let source = std::fs::read_to_string(sui_file)
        .map_err(|e| CompileError::Read(sui_file.to_path_buf(), e))?;
    let opts = CompileOpts {
        setup_url: paired_controller(sui_file, stem),
        mode: match mode {
            CompileMode::Component => SuiMode::Component,
            CompileMode::View => SuiMode::Component,
        },
        source_path: Some(sui_file.to_path_buf()),
        styles: None,
        component_meta: (mode == CompileMode::Component)
            .then(|| ComponentMeta { name: stem.to_string(), shadow: true }),
        // The bundler reads only the JS — dts/ssr are the CLI bake's job, and
        // the CEM is walked from disk by the catalog, not returned here.
        emit_dts: false,
        emit_component_imports,
    };
    match compile(&source, opts) {
        Ok(out) => {
            // deck build only BLOCKS on Error severity (`compile()` runs
            // `check::gate` over the full Error stream). The Warning-level
            // `sui-lint` findings (`check::collect`) are advisory — surface
            // them to stderr during the build so authors see them, without
            // failing. A compile that reached here already passed the Error
            // gate, so only Warnings remain. Re-runs the collect() pass (a
            // parse + lint over this one file); cheap next to bundling and
            // only on the build path.
            emit_build_warnings(sui_file, &source);
            Ok(out.js)
        }
        Err(e) => Err(CompileError::Compile(e.render(&source))),
    }
}

/// Print Warning-severity diagnostics for `source` to stderr (the build
/// surfacing seam for #2). Errors never appear — they fail `compile()` before
/// this is reached. Mirrors the CLI/MCP rendering: `<path>:<line>:<col>
/// [<rule>] <message>`.
fn emit_build_warnings(sui_file: &Path, source: &str) {
    for d in sui_compiler::check_all(source, Some(sui_file)) {
        if d.severity != sui_compiler::DiagSeverity::Warning {
            continue;
        }
        let rule = d.rule.as_deref().unwrap_or("sui-lint");
        eprintln!(
            "deck warn {}:{}:{} [{}] {}",
            sui_file.display(),
            d.line,
            d.col,
            rule,
            d.message
        );
    }
}

/// `import setup from "./<controller>"` when a sibling controller exists next to
/// the `.sui`. Mirrors the CLI's `paired_setup` for the bundler (`--out`/
/// in-process) case: relative `./` import that Rolldown resolves to the sibling.
fn paired_controller(sui_file: &Path, stem: &str) -> Option<String> {
    let parent = sui_file.parent()?;
    let filename = if stem == "page" {
        "controller.ts".to_string()
    } else {
        format!("{stem}.controller.ts")
    };
    parent.join(&filename).exists().then(|| format!("./{filename}"))
}

/// The gate for the synthetic CLIENT emit. Default OFF — the bundler emits the
/// production module. ON only when `SUI_CLIENT_EMIT=synthetic`, and even then
/// only for views that opt in by having an `app.manifest.jsonc` above them
/// (see [`try_synthetic_client`]). This is the slice-1 proof seam: additive,
/// scoped, never a global default flip.
pub fn synthetic_client_enabled() -> bool {
    std::env::var("SUI_CLIENT_EMIT").map(|v| v == "synthetic").unwrap_or(false)
}

/// Emit the synthetic CLIENT module for `sui_file` when it opts into the
/// synthetic path: an `app.manifest.jsonc` exists at or above its directory.
/// Returns `Ok(None)` (→ production emit) when no manifest is found, so a
/// non-manifested view is untouched even with the flag set. The merged module
/// composes a sibling `controller.ts` (absolute path so Rolldown resolves it)
/// and resolves the manifest's free names as real ESM imports.
fn try_synthetic_client(sui_file: &Path) -> Result<Option<String>, CompileError> {
    let Some(manifest_path) = find_app_manifest(sui_file) else {
        return Ok(None);
    };
    let source = std::fs::read_to_string(sui_file)
        .map_err(|e| CompileError::Read(sui_file.to_path_buf(), e))?;
    let manifest_src = std::fs::read_to_string(&manifest_path)
        .map_err(|e| CompileError::Read(manifest_path.clone(), e))?;
    let program = parse(&source).map_err(|e| CompileError::Compile(e.render(&source)))?;
    let mut manifest =
        AppManifest::parse(&manifest_src).map_err(|e| CompileError::Compile(e.render(&manifest_src)))?;
    // Deck declares its host context (route/navigate -> @ship/deck/router) into
    // the manifest seam; the app's own entries win. SUI names no host.
    for (name, client) in deck_core::host_context::host_context_clients() {
        manifest.clients.entry(name).or_insert(client);
    }
    let authored = sibling_controller_abs(sui_file)
        .map(|specifier| AuthoredController { specifier });
    let js = emit_synthetic_client_module(&program, &manifest, authored.as_ref(), SuiMode::Component, None)
        .map_err(|e| CompileError::Compile(e.render(&source)))?;
    Ok(Some(js))
}

/// Walk up from `sui_file`'s directory to the nearest `app.manifest.jsonc`.
fn find_app_manifest(sui_file: &Path) -> Option<PathBuf> {
    let mut dir = sui_file.parent();
    while let Some(d) = dir {
        let candidate = d.join("app.manifest.jsonc");
        if candidate.is_file() {
            return Some(candidate);
        }
        dir = d.parent();
    }
    None
}

/// The sibling authored controller next to a `page.sui`, as an absolute path
/// string — the synthetic controller imports it directly so Rolldown resolves
/// it to the on-disk `.ts`.
fn sibling_controller_abs(sui_file: &Path) -> Option<String> {
    let c = sui_file.parent()?.join("controller.ts");
    c.is_file().then(|| c.to_string_lossy().into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    /// Serializes the tests that mutate the process-global `SUI_CLIENT_EMIT`
    /// env var. Cargo runs tests in parallel threads and env is process-wide,
    /// so without this the two env-mutating tests race — one's set/remove
    /// clobbers the other's assertions (latent before; exposed when sibling
    /// crates added more `compile()`-calling tests that shifted scheduling).
    /// Read-only consumers (`compile()` elsewhere) are unaffected — their
    /// assertions don't depend on the synthetic flag.
    static EMIT_ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn compiles_view_to_js_in_process() {
        let src = tempdir().unwrap();
        let sui = src.path().join("page.sui");
        std::fs::write(&sui, "<div class=\"hello\">hi {1 + 1}</div>\n").unwrap();
        let js = compile_sui(&sui, CompileMode::View, "page", false).expect("view compiles");
        assert!(js.contains("@ship/sui-runtime"), "emits a runtime import; got:\n{js}");
    }

    #[test]
    fn view_inline_style_is_bundled_not_dropped() {
        // The bundler must BUNDLE a route `page.sui`'s inline `<style>` as the
        // route's stylesheet (installed at mount), not silently drop it.
        let src = tempdir().unwrap();
        let sui = src.path().join("page.sui");
        std::fs::write(
            &sui,
            "<style>.hero { color: var(--foreground); }</style>\n<div class=\"hero\">hi</div>\n",
        )
        .unwrap();
        let js = compile_sui(&sui, CompileMode::View, "page", false).expect("view compiles");
        assert!(
            js.contains(".hero { color: var(--foreground); }"),
            "inline <style> must be bundled into the view JS, not dropped:\n{js}"
        );
        assert!(
            js.contains("data-sui-style"),
            "the runtime style install must be present:\n{js}"
        );
    }

    #[test]
    fn surfaces_compile_errors_with_rendered_diagnostic() {
        let src = tempdir().unwrap();
        let sui = src.path().join("broken.sui");
        std::fs::write(&sui, "<div>{#if cond()}oops</div>\n").unwrap();
        let err = compile_sui(&sui, CompileMode::View, "broken", false).unwrap_err();
        assert!(matches!(err, CompileError::Compile(_)));
    }

    #[test]
    fn component_imports_flag_emits_tag_registration() {
        let src = tempdir().unwrap();
        let sui = src.path().join("page.sui");
        std::fs::write(&sui, "<div><sui-table></sui-table></div>\n").unwrap();
        let on = compile_sui(&sui, CompileMode::View, "page", true).expect("compiles");
        assert!(on.contains("@sui-tag/sui-table"), "flag emits the registration import");
        let off = compile_sui(&sui, CompileMode::View, "page", false).expect("compiles");
        assert!(!off.contains("@sui-tag/"), "off: no component imports");
    }

    /// The synthetic gate is OFF by default and ON only for manifested views.
    /// Serialized via the env var; restore it so sibling tests are unaffected.
    #[test]
    fn synthetic_gate_is_scoped_and_additive() {
        let _env = EMIT_ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let src = tempdir().unwrap();
        let dir = src.path();
        std::fs::write(dir.join("page.sui"), "<h1>{route.params.id()}</h1>\n").unwrap();
        let sui = dir.join("page.sui");

        // Flag OFF (default): production emit regardless of manifest presence.
        let prev = std::env::var("SUI_CLIENT_EMIT").ok();
        unsafe { std::env::remove_var("SUI_CLIENT_EMIT") };
        let off = compile_sui(&sui, CompileMode::View, "page", false).expect("compiles");
        assert!(
            off.contains("component((ctx)") && !off.contains("export default component(() =>"),
            "flag-off must be the production emit:\n{off}"
        );

        // Flag ON but NO manifest above → still production (scoped opt-in).
        unsafe { std::env::set_var("SUI_CLIENT_EMIT", "synthetic") };
        let on_no_manifest = compile_sui(&sui, CompileMode::View, "page", false).expect("compiles");
        assert!(
            on_no_manifest.contains("component((ctx)"),
            "flag-on without a manifest stays production:\n{on_no_manifest}"
        );

        // Flag ON + an app manifest above → synthetic client contract.
        std::fs::write(
            dir.join("app.manifest.jsonc"),
            r#"{ "clients": { "route": { "from": "@ship/deck/router", "import": "route", "construct": true } } }"#,
        )
        .unwrap();
        let on = compile_sui(&sui, CompileMode::View, "page", false).expect("compiles");
        assert!(
            on.contains("export default component(() =>") && on.contains("__mount(container"),
            "flag-on + manifest emits the synthetic client contract:\n{on}"
        );

        match prev {
            Some(v) => unsafe { std::env::set_var("SUI_CLIENT_EMIT", v) },
            None => unsafe { std::env::remove_var("SUI_CLIENT_EMIT") },
        }
    }

    /// `route` and `navigate` are the view-host handles every `.sui` view may
    /// reference; they resolve through the synthetic emit's convention
    /// (`@ship/deck/router` import, `construct: true` for `route`) so an
    /// app needs no `app.manifest.jsonc` entry for them. The synthetic
    /// CLIENT contract still gates on the manifest (it scopes which views
    /// opt in), but the route handle is now convention-driven and reaches
    /// the bundled module the moment the flag turns on.
    #[test]
    fn convention_resolves_route_navigate_without_manifest() {
        let _env = EMIT_ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let src = tempdir().unwrap();
        let dir = src.path();
        std::fs::write(
            dir.join("page.sui"),
            "<h1>id={route.params.id()}</h1>\n",
        )
        .unwrap();
        let sui = dir.join("page.sui");

        // Flag ON, NO manifest above → convention still kicks in and the
        // synthetic client contract is emitted (the manifest here is the
        // "any view that opts in" trigger, not the source of `route`).
        let prev = std::env::var("SUI_CLIENT_EMIT").ok();
        unsafe { std::env::set_var("SUI_CLIENT_EMIT", "synthetic") };
        std::fs::write(
            dir.join("app.manifest.jsonc"),
            // The manifest is now ONLY used to opt this view in. The
            // view-host handles resolve by convention.
            r#"{ "clients": {} }"#,
        )
        .unwrap();
        let on = compile_sui(&sui, CompileMode::View, "page", false).expect("compiles");
        assert!(
            on.contains("export default component(() =>") && on.contains("__mount(container"),
            "convention must drive the synthetic client contract:\n{on}"
        );
        // `route` is a construct: the convention emits
        // `import { route as __make_route } from "@ship/deck/router"`
        // and `const route = __make_route()`.
        assert!(
            on.contains("__make_route") && on.contains("@ship/deck/router"),
            "convention must import `route` from `@ship/deck/router`:\n{on}"
        );

        // A view that does NOT reference `route` must not pull in
        // `@ship/deck/router` — the convention is per-view, not global.
        std::fs::write(
            dir.join("page.sui"),
            "<p>just a paragraph, no route reads</p>\n",
        )
        .unwrap();
        let on_no_route = compile_sui(&sui, CompileMode::View, "page", false).expect("compiles");
        assert!(
            !on_no_route.contains("@ship/deck/router"),
            "a view that never references `route` must not import `@ship/deck/router`:\n{on_no_route}"
        );

        match prev {
            Some(v) => unsafe { std::env::set_var("SUI_CLIENT_EMIT", v) },
            None => unsafe { std::env::remove_var("SUI_CLIENT_EMIT") },
        }
    }
}
