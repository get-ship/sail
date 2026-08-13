//! sail-compiler integration for the Rust bundler — IN-PROCESS (#255).
//!
//! The bundler calls `sail_compiler::compile()` directly from its Rolldown
//! `load` hook. One toolchain: the compiler is the crate the bundler links, so
//! there is no separate `sail-compiler` binary to locate, no installed-vs-debug-
//! vs-release drift, and a flag the bundler passes can never be rejected by a
//! stale binary. `just build` produces a bundler carrying *the* current
//! compiler — build = works, no gate, no version dance.

use std::path::{Path, PathBuf};

use sail_compiler::emit::synthetic::{
    AppManifest, AuthoredController, emit_synthetic_client_module,
};
use sail_compiler::parse::parse;
use sail_compiler::parse::preprocess::extract_controller_block;
use sail_compiler::{CompileMode as SailCompileMode, CompileOpts, ComponentMeta, Policy, compile};

/// A `.sail` compiles as a custom-element (`component`) or a route. Component mode
/// passes a tag name so the element is registered; route mode emits a route module.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompileMode {
    Component,
    Route,
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

/// Compile one `.sail` to JS via `sail_compiler::compile` (in-process). Auto-wires
/// a sibling controller (`page.sail`→`controller.ts`, `<stem>.sail`→
/// `<stem>.controller.ts`) as `import setup from "./<controller>"`.
/// `emit_component_imports` opts the route into the per-route resolver — it emits
/// `@sail-tag/<tag>` registration imports the bundler resolves to source.
pub fn compile_sail(
    sail_file: &Path,
    mode: CompileMode,
    stem: &str,
    emit_component_imports: bool,
) -> Result<String, CompileError> {
    // Gated, scoped synthetic CLIENT emit (slice 1). When `SAIL_CLIENT_EMIT=
    // synthetic` AND this is a ROUTE with an `app.manifest.jsonc` above it, emit
    // the synthetic client contract (`default.setup` → `__mount` + `hydrate`)
    // instead of the production module. Default OFF and scoped to manifested
    // routes, so every other target — and the flag-off case — is byte-for-byte
    // the production emit below.
    if mode == CompileMode::Route && synthetic_client_enabled() {
        if let Some(js) = try_synthetic_client(sail_file)? {
            return Ok(js);
        }
    }
    let source = std::fs::read_to_string(sail_file)
        .map_err(|e| CompileError::Read(sail_file.to_path_buf(), e))?;
    // The FLOOR is unconditional — no <style>, no style=, in any SAIL file, with
    // or without a config. A deck project does not opt into the language. The
    // capability keys (sx / variants / raw containers) still come from the
    // nearest `sail.config.json` walking up, which is how a project dials the
    // surface ABOVE the floor.
    let policy_overrides = Policy::overrides_for(sail_file);
    // POSITION GRANT, computed once: deck routes by file location, so deck
    // (not the author, not the sail compiler) decides whether this file may
    // read `route` — and with which params.
    let route_params = (mode == CompileMode::Route)
        .then(|| deck_core::routes::route_params_for(sail_file))
        .flatten();
    let opts = CompileOpts {
        setup_url: controller_specifier(sail_file, stem, &source),
        mode: match mode {
            CompileMode::Component => SailCompileMode::Component,
            CompileMode::Route => SailCompileMode::Component,
        },
        source_path: Some(sail_file.to_path_buf()),
        component_meta: (mode == CompileMode::Component).then(|| ComponentMeta {
            name: stem.to_string(),
        }),
        route_params: route_params.clone(),
        // The bundler reads only the JS — dts/ssr are the CLI bake's job, and
        // the CEM is walked from disk by the catalog, not returned here.
        emit_dts: false,
        scope_capabilities: Vec::new(),
        emit_component_imports,
        // The resolved per-package floor (empty for unconfigured packages). The
        // bundler never softens the design-system lints beyond a package's own
        // declared policy.
        lint_overrides: policy_overrides.clone(),
    };
    match compile(&source, opts) {
        Ok(out) => {
            // Warning-level findings are advisory — print them without
            // failing. `compile()` already ran `collect()` inside `gate`;
            // a second `check_all` would walk the file again.
            emit_build_warnings(sail_file, &out.diagnostics);
            Ok(out.js)
        }
        Err(e) => Err(CompileError::Compile(e.render(&source))),
    }
}

/// Print Warning-severity diagnostics to stderr. Errors never appear —
/// they fail `compile()` before this is reached. Mirrors the CLI/MCP
/// rendering: `<path>:<line>:<col> [<rule>] <message>`.
fn emit_build_warnings(sail_file: &Path, diagnostics: &[sail_compiler::Diagnostic]) {
    for line in warning_lines(sail_file, diagnostics) {
        eprintln!("{line}");
    }
}

fn warning_lines(sail_file: &Path, diagnostics: &[sail_compiler::Diagnostic]) -> Vec<String> {
    diagnostics
        .iter()
        .filter(|d| d.severity == sail_compiler::DiagSeverity::Warning)
        .map(|d| {
            let rule = d.rule.as_deref().unwrap_or("sail-lint");
            format!(
                "deck warn {}:{}:{} [{}] {}",
                sail_file.display(),
                d.line,
                d.col,
                rule,
                d.message
            )
        })
        .collect()
}

/// The setup-import specifier for a `.sail`'s controller — SFC first. An inline
/// `<controller>` block IS the controller: the specifier is the VIRTUAL
/// `./<file>.sail.controller.ts` sibling the plugin serves by extracting the
/// block (nothing is materialized to disk — one authored file). Only a source
/// without a block falls back to the paired-file model (a real sibling
/// `controller.ts` / `<stem>.controller.ts` on disk). Inline-wins matches
/// `compile()`'s own controller resolution for the scope-aware checks, so what
/// the checks validate is exactly what the bundle runs.
fn controller_specifier(sail_file: &Path, stem: &str, source: &str) -> Option<String> {
    if extract_controller_block(source).1.is_some() {
        let file = sail_file.file_name()?.to_string_lossy();
        return Some(format!("./{file}.controller.ts"));
    }
    paired_controller(sail_file, stem)
}

/// `import setup from "./<controller>"` when a sibling controller exists next to
/// the `.sail`. Mirrors the CLI's `paired_setup` for the bundler (`--out`/
/// in-process) case: relative `./` import that Rolldown resolves to the sibling.
fn paired_controller(sail_file: &Path, stem: &str) -> Option<String> {
    let parent = sail_file.parent()?;
    let filename = if stem == "page" {
        "controller.ts".to_string()
    } else {
        format!("{stem}.controller.ts")
    };
    parent
        .join(&filename)
        .exists()
        .then(|| format!("./{filename}"))
}

/// The gate for the synthetic CLIENT emit. Default OFF — the bundler emits the
/// production module. ON only when `SAIL_CLIENT_EMIT=synthetic`, and even then
/// only for routes that opt in by having an `app.manifest.jsonc` above them
/// (see [`try_synthetic_client`]). This is the slice-1 proof seam: additive,
/// scoped, never a global default flip.
pub fn synthetic_client_enabled() -> bool {
    std::env::var("SAIL_CLIENT_EMIT")
        .map(|v| v == "synthetic")
        .unwrap_or(false)
}

/// Emit the synthetic CLIENT module for `sail_file` when it opts into the
/// synthetic path: an `app.manifest.jsonc` exists at or above its directory.
/// Returns `Ok(None)` (→ production emit) when no manifest is found, so a
/// non-manifested route is untouched even with the flag set. The merged module
/// composes a sibling `controller.ts` (absolute path so Rolldown resolves it)
/// and resolves the manifest's free names as real ESM imports.
fn try_synthetic_client(sail_file: &Path) -> Result<Option<String>, CompileError> {
    let Some(manifest_path) = find_app_manifest(sail_file) else {
        return Ok(None);
    };
    let source = std::fs::read_to_string(sail_file)
        .map_err(|e| CompileError::Read(sail_file.to_path_buf(), e))?;
    let manifest_src = std::fs::read_to_string(&manifest_path)
        .map_err(|e| CompileError::Read(manifest_path.clone(), e))?;
    let program = parse(&source).map_err(|e| CompileError::Compile(e.render(&source)))?;
    let mut manifest = AppManifest::parse(&manifest_src)
        .map_err(|e| CompileError::Compile(e.render(&manifest_src)))?;
    // Deck declares its host context (route/navigate -> @ship/deck/router) into
    // the manifest seam; the app's own entries win. SAIL names no host.
    for (name, client) in deck_core::host_context::host_context_clients() {
        manifest.clients.entry(name).or_insert(client);
    }
    // Inline `<controller>` wins here too: the authored controller is the
    // virtual `.sail.controller.ts` module the plugin extracts from the source.
    let authored = extract_controller_block(&source)
        .1
        .map(|_| format!("{}.controller.ts", sail_file.display()))
        .or_else(|| sibling_controller_abs(sail_file))
        .map(|specifier| AuthoredController { specifier });
    let js = emit_synthetic_client_module(
        &program,
        &manifest,
        authored.as_ref(),
        SailCompileMode::Component,
        None,
    )
    .map_err(|e| CompileError::Compile(e.render(&source)))?;
    Ok(Some(js))
}

/// Walk up from `sail_file`'s directory to the nearest `app.manifest.jsonc`.
fn find_app_manifest(sail_file: &Path) -> Option<PathBuf> {
    let mut dir = sail_file.parent();
    while let Some(d) = dir {
        let candidate = d.join("app.manifest.jsonc");
        if candidate.is_file() {
            return Some(candidate);
        }
        dir = d.parent();
    }
    None
}

/// The sibling authored controller next to a `page.sail`, as an absolute path
/// string — the synthetic controller imports it directly so Rolldown resolves
/// it to the on-disk `.ts`.
fn sibling_controller_abs(sail_file: &Path) -> Option<String> {
    let c = sail_file.parent()?.join("controller.ts");
    c.is_file().then(|| c.to_string_lossy().into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    /// Serializes the tests that mutate the process-global `SAIL_CLIENT_EMIT`
    /// env var. Cargo runs tests in parallel threads and env is process-wide,
    /// so without this the two env-mutating tests race — one's set/remove
    /// clobbers the other's assertions (latent before; exposed when sibling
    /// crates added more `compile()`-calling tests that shifted scheduling).
    /// Read-only consumers (`compile()` elsewhere) are unaffected — their
    /// assertions don't depend on the synthetic flag.
    static EMIT_ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn compiles_route_to_js_in_process() {
        let src = tempdir().unwrap();
        let sail = src.path().join("page.sail");
        std::fs::write(&sail, "<div>hi {1 + 1}</div>\n").unwrap();
        let js = compile_sail(&sail, CompileMode::Route, "page", false).expect("view compiles");
        assert!(
            js.contains("@ship/sail-runtime"),
            "emits a runtime import; got:\n{js}"
        );
    }

    #[test]
    fn route_raw_style_block_is_rejected_by_the_floor() {
        // The FLOOR: authored <style> is a compile error in every SAIL file,
        // config or not — deck enforces it for route modules like everything
        // else. (This test used to pin the pre-floor behavior — inline
        // <style> bundled into the route JS. That surface is gone.)
        let src = tempdir().unwrap();
        let sail = src.path().join("page.sail");
        std::fs::write(
            &sail,
            "<style>.hero { color: var(--foreground); }</style>\n<p>hi</p>\n",
        )
        .unwrap();
        let err = compile_sail(&sail, CompileMode::Route, "page", false)
            .expect_err("raw <style> in a view must not compile");
        assert!(
            format!("{err}").contains("sail-no-style-block"),
            "the floor rule names itself: {err}"
        );
    }

    #[test]
    fn surfaces_compile_errors_with_rendered_diagnostic() {
        let src = tempdir().unwrap();
        let sail = src.path().join("broken.sail");
        std::fs::write(&sail, "<div><if {cond()}>oops</if></div>\n").unwrap();
        let err = compile_sail(&sail, CompileMode::Route, "broken", false).unwrap_err();
        assert!(matches!(err, CompileError::Compile(_)));
    }

    #[test]
    fn component_imports_flag_emits_tag_registration() {
        let src = tempdir().unwrap();
        let sail = src.path().join("page.sail");
        std::fs::write(&sail, "<div><sail-table></sail-table></div>\n").unwrap();
        let on = compile_sail(&sail, CompileMode::Route, "page", true).expect("compiles");
        assert!(
            on.contains("@sail-tag/sail-table"),
            "flag emits the registration import"
        );
        let off = compile_sail(&sail, CompileMode::Route, "page", false).expect("compiles");
        assert!(!off.contains("@sail-tag/"), "off: no component imports");
    }

    /// The synthetic gate is OFF by default and ON only for manifested routes.
    /// Serialized via the env var; restore it so sibling tests are unaffected.
    #[test]
    fn synthetic_gate_is_scoped_and_additive() {
        let _env = EMIT_ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let src = tempdir().unwrap();
        // Route-ness is POSITION: the page must live under a `routes/` (or
        // `routes/`) ancestor for deck to grant `route` at compile time.
        let dir = src.path().join("routes/greet/[id]");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("page.sail"), "<h1>{route.params.id()}</h1>\n").unwrap();
        let sail = dir.join("page.sail");

        // Flag OFF (default): production emit regardless of manifest presence.
        let prev = std::env::var("SAIL_CLIENT_EMIT").ok();
        unsafe { std::env::remove_var("SAIL_CLIENT_EMIT") };
        let off = compile_sail(&sail, CompileMode::Route, "page", false).expect("compiles");
        assert!(
            off.contains("component((ctx)") && !off.contains("export default component(() =>"),
            "flag-off must be the production emit:\n{off}"
        );

        // Flag ON but NO manifest above → still production (scoped opt-in).
        unsafe { std::env::set_var("SAIL_CLIENT_EMIT", "synthetic") };
        let on_no_manifest =
            compile_sail(&sail, CompileMode::Route, "page", false).expect("compiles");
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
        let on = compile_sail(&sail, CompileMode::Route, "page", false).expect("compiles");
        assert!(
            on.contains("export default component(() =>") && on.contains("__mount(container"),
            "flag-on + manifest emits the synthetic client contract:\n{on}"
        );

        match prev {
            Some(v) => unsafe { std::env::set_var("SAIL_CLIENT_EMIT", v) },
            None => unsafe { std::env::remove_var("SAIL_CLIENT_EMIT") },
        }
    }

    /// `route` and `navigate` are the route-host handles every `.sail` route may
    /// reference; they resolve through the synthetic emit's convention
    /// (`@ship/deck/router` import, `construct: true` for `route`) so an
    /// app needs no `app.manifest.jsonc` entry for them. The synthetic
    /// CLIENT contract still gates on the manifest (it scopes which routes
    /// opt in), but the route handle is now convention-driven and reaches
    /// the bundled module the moment the flag turns on.
    #[test]
    fn convention_resolves_route_navigate_without_manifest() {
        let _env = EMIT_ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let src = tempdir().unwrap();
        let dir = src.path();
        std::fs::write(dir.join("page.sail"), "<h1>id={route.params.id()}</h1>\n").unwrap();
        let sail = dir.join("page.sail");

        // Flag ON, NO manifest above → convention still kicks in and the
        // synthetic client contract is emitted (the manifest here is the
        // "any route that opts in" trigger, not the source of `route`).
        let prev = std::env::var("SAIL_CLIENT_EMIT").ok();
        unsafe { std::env::set_var("SAIL_CLIENT_EMIT", "synthetic") };
        std::fs::write(
            dir.join("app.manifest.jsonc"),
            // The manifest is now ONLY used to opt this route in. The
            // route-host handles resolve by convention.
            r#"{ "clients": {} }"#,
        )
        .unwrap();
        let on = compile_sail(&sail, CompileMode::Route, "page", false).expect("compiles");
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

        // A route that does NOT reference `route` must not pull in
        // `@ship/deck/router` — the convention is per-route, not global.
        std::fs::write(
            dir.join("page.sail"),
            "<p>just a paragraph, no route reads</p>\n",
        )
        .unwrap();
        let on_no_route = compile_sail(&sail, CompileMode::Route, "page", false).expect("compiles");
        assert!(
            !on_no_route.contains("@ship/deck/router"),
            "a view that never references `route` must not import `@ship/deck/router`:\n{on_no_route}"
        );

        match prev {
            Some(v) => unsafe { std::env::set_var("SAIL_CLIENT_EMIT", v) },
            None => unsafe { std::env::remove_var("SAIL_CLIENT_EMIT") },
        }
    }

    #[test]
    fn warning_severity_lint_surfaces_from_compile_diagnostics() {
        let src = tempdir().unwrap();
        let sail = src.path().join("warn-chip.sail");
        std::fs::write(
            &sail,
            r#"
<scope>
props: {
  variant: 'a' | 'b' | 'c' = 'a'
}
</scope>
<div style({ match: { variant: { a: { paint: bg(primary) }, b: { paint: bg(muted) } } } })>x</div>
"#,
        )
        .unwrap();
        let js = compile_sail(&sail, CompileMode::Component, "warn-chip", false)
            .expect("a Warning must not fail compile");
        assert!(
            js.contains("@ship/sail-runtime"),
            "component still emits: {js}"
        );

        let source = std::fs::read_to_string(&sail).unwrap();
        let out = compile(
            &source,
            CompileOpts {
                setup_url: None,
                mode: SailCompileMode::Component,
                source_path: Some(sail.clone()),
                component_meta: Some(ComponentMeta {
                    name: "warn-chip".into(),
                }),
                route_params: None,
                emit_dts: false,
                scope_capabilities: Vec::new(),
                emit_component_imports: false,
                lint_overrides: Policy::overrides_for(&sail),
            },
        )
        .expect("same source compiles");
        let lines = warning_lines(&sail, &out.diagnostics);
        assert!(
            lines
                .iter()
                .any(|l| l.contains("sail-style-match-exhaustive")),
            "warning-severity lint must still print: {lines:?}"
        );
    }

    #[test]
    fn type_gate_still_fails_compile() {
        let src = tempdir().unwrap();
        let sail = src.path().join("page.sail");
        std::fs::write(
            &sail,
            r#"
<controller>
export function setup() {
  const n: number = "nope";
  return {};
}
</controller>
<div>hi</div>
"#,
        )
        .unwrap();
        let err = compile_sail(&sail, CompileMode::Route, "page", false)
            .expect_err("type-gate must still fail compile");
        assert!(matches!(err, CompileError::Compile(_)), "{err}");
    }

    #[test]
    fn compile_sail_does_not_double_collect_on_showcase() {
        let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        for _ in 0..3 {
            root.pop();
        }
        let sail = root.join("apps/sail-site/routes/showcase/page.sail");
        assert!(
            sail.is_file(),
            "showcase fixture missing: {}",
            sail.display()
        );

        let source = std::fs::read_to_string(&sail).unwrap();
        let opts = CompileOpts {
            setup_url: controller_specifier(&sail, "page", &source),
            mode: SailCompileMode::Component,
            source_path: Some(sail.clone()),
            component_meta: None,
            route_params: deck_core::routes::route_params_for(&sail),
            emit_dts: false,
            scope_capabilities: Vec::new(),
            emit_component_imports: false,
            lint_overrides: Policy::overrides_for(&sail),
        };

        // Warm caches so the timed pass matches an HMR miss, not process start.
        let _ = compile(&source, opts.clone());
        let _ = compile_sail(&sail, CompileMode::Route, "page", false);

        let t = std::time::Instant::now();
        let out = compile(&source, opts).expect("showcase compiles");
        let compile_ms = t.elapsed().as_secs_f64() * 1000.0;

        let t = std::time::Instant::now();
        let lines = warning_lines(&sail, &out.diagnostics);
        let warn_ms = t.elapsed().as_secs_f64() * 1000.0;
        let _ = lines.len();

        let t = std::time::Instant::now();
        compile_sail(&sail, CompileMode::Route, "page", false).expect("showcase compiles");
        let sail_ms = t.elapsed().as_secs_f64() * 1000.0;
        eprintln!(
            "showcase HMR probe: compile={compile_ms:.1}ms warn={warn_ms:.3}ms compile_sail={sail_ms:.1}ms"
        );

        assert!(
            warn_ms < 5.0,
            "reusing compile() diagnostics must be cheap, got {warn_ms:.3}ms"
        );
        assert!(
            sail_ms < compile_ms * 1.25 + 5.0,
            "compile_sail must not pay a second collect(): compile={compile_ms:.1}ms sail={sail_ms:.1}ms"
        );
        assert!(
            compile_ms + warn_ms < 1300.0,
            "showcase compile+warn {compile_ms:.1}+{warn_ms:.3}ms exceeds 1.3s"
        );
    }
}
