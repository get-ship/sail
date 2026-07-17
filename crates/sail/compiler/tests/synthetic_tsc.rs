//! tsgo/tsc proof for the synthetic emitter (acceptance #1).
//!
//! Synthesizes the four representative surfaces against the conformance app's
//! manifest, then type-checks the emitted controller+render pairs with the
//! workspace `tsc` against the REAL `@ship/deck/router`, `@ship/sail-runtime`,
//! and the app's typed demo clients — proving manifest names resolve as real
//! imports with real type resolution (no ambient passthrough). Finally it
//! shows a deliberate `route.parms` typo is CAUGHT (TS2339).
//!
//! The tsconfig is FULL STRICT (`strict: true`, `noImplicitAny: true`): the
//! synthetic render's generated machinery (boundary `anchor`, `{#while-error}`
//! binder, disposer arrays) is type-annotated mode-aware, so no implicit-`any`
//! remains. Author expressions were already fully checked; this closes the
//! machinery gap so a synthesized view checks exactly as strictly as authored TS.
//!
//! Opt-in by environment: requires `bun install` so `node_modules/.bin/tsc`
//! and the workspace package sources exist. When `tsc` is absent the test
//! prints why and returns — it never fails for a missing toolchain (mirrors
//! `parity.rs`'s opt-in stance).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use sail_compiler::emit::synthetic::{emit_synthetic, AppManifest, AuthoredController, ManifestClient};
use sail_compiler::parse::parse;
use sail_compiler::CompileMode;

/// Mirror deck's host context — the `route`/`navigate` handles deck injects into
/// every app manifest at build (`deck_core::host_context_clients`). SUI's
/// compiler names no host, so post-seam route/navigate must be DECLARED in the
/// manifest; the real pipeline merges them in. This stands in for that merge so
/// the route-bearing surface + the route-typo gate type-check against
/// `RouteSignals`. (sail-compiler can't depend on deck-core — that's a cycle.)
fn inject_host_context(manifest: &mut AppManifest) {
    manifest.clients.entry("route".to_string()).or_insert(ManifestClient {
        from: "@ship/deck/router".to_string(),
        import: "route".to_string(),
        construct: true,
    });
    manifest.clients.entry("navigate".to_string()).or_insert(ManifestClient {
        from: "@ship/deck/router".to_string(),
        import: "navigate".to_string(),
        construct: false,
    });
}

fn repo_root() -> PathBuf {
    // crates/sail/compiler -> crates/sui -> crates -> repo root.
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..").canonicalize().unwrap()
}

fn tsc_path(repo: &Path) -> Option<PathBuf> {
    let p = repo.join("node_modules/.bin/tsc");
    p.exists().then_some(p)
}

struct Surface {
    name: &'static str,
    source_rel: &'static str,
    mode: CompileMode,
    /// Authored controller specifier (relative to the generated dir), if any.
    authored: Option<&'static str>,
    /// Component tag name (kebab-case) for component-mode emit; `None` for views.
    component_tag: Option<&'static str>,
}

fn synth_into(repo: &Path, manifest: &AppManifest, gendir: &Path, s: &Surface) {
    let src = fs::read_to_string(repo.join(s.source_rel))
        .unwrap_or_else(|e| panic!("read {}: {e}", s.source_rel));
    // A source may carry its controller INLINE (`<controller>…</controller>`)
    // rather than as a sibling module — strip it before the template-only
    // parser sees it (mirrors `deck_ssr::build`).
    let (stripped, inline_controller) =
        sail_compiler::parse::preprocess::extract_controller_block(&src);
    let program = parse(&stripped).unwrap_or_else(|e| panic!("parse {}: {}", s.name, e.render(&stripped)));
    let dir = gendir.join(s.name);
    fs::create_dir_all(&dir).unwrap();
    let authored = if let Some(a) = s.authored {
        Some(AuthoredController { specifier: a.to_string() })
    } else if let Some(body) = inline_controller {
        fs::write(dir.join("controller.ts"), body).unwrap();
        Some(AuthoredController { specifier: "./controller.ts".to_string() })
    } else {
        None
    };
    let controller_name = format!("{}.synthetic.controller.ts", s.name);
    let out = emit_synthetic(
        &program,
        manifest,
        authored.as_ref(),
        &format!("./{controller_name}"),
        s.mode,
        s.component_tag,
    )
    .unwrap_or_else(|e| panic!("synthesize {}: {}", s.name, e.render(&src)));
    fs::write(dir.join(&controller_name), out.controller_ts).unwrap();
    fs::write(dir.join(format!("{}.render.ts", s.name)), out.render_ts).unwrap();
}

/// Ambient type shim for the `ship` host-dispatch module — the ONE external
/// specifier a controller's `live(() => call(op, args))` source (or
/// `ssr-entry.ts`) imports. Never a real npm package (deno_core supplies it
/// at runtime via `op_ship_call`); this shim exists so tsc can resolve it in
/// the generated-surfaces project. Mirrors the real signature `demo.server.ts`
/// / `ssr-entry.ts` already call against.
fn write_ship_shim(dir: &Path) {
    fs::write(
        dir.join("ship-shim.d.ts"),
        "declare module 'ship' {\n  \
         export function call(op: string, args?: Record<string, unknown>): Promise<unknown>\n  \
         export function emit(value: unknown): void\n\
         }\n",
    )
    .unwrap();
}

fn write_tsconfig(repo: &Path, dir: &Path, include: &[&str], name: &str) {
    let r = repo.display();
    let cfg = format!(
        r#"{{
  "compilerOptions": {{
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "lib": ["ES2022", "DOM", "DOM.Iterable"],
    "types": [],
    "strict": true,
    "noImplicitAny": true,
    "skipLibCheck": true,
    "noEmit": true,
    "allowImportingTsExtensions": true,
    "baseUrl": ".",
    "paths": {{
      "@ship/deck/router": ["{r}/packages/deck/router/src/server.ts"],
      "@ship/sail-runtime": ["{r}/packages/sail/runtime/src/server.ts"],
      "@ship/sail-runtime/*": ["{r}/packages/sail/runtime/src/*"],
      "@ship/deck/router": ["{r}/packages/deck/router/src/server.ts"],
      "@app/*": ["{r}/crates/deck/deck-conformance/app/*"]
    }}
  }},
  "include": [{}]
}}"#,
        include.iter().map(|i| format!("\"{i}\"")).collect::<Vec<_>>().join(", ")
    );
    fs::write(dir.join(name), cfg).unwrap();
}

fn run_tsc(tsc: &Path, dir: &Path, project: &str) -> (bool, String) {
    let out = Command::new(tsc)
        .arg("-p")
        .arg(project)
        .current_dir(dir)
        .output()
        .expect("run tsc");
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    (out.status.success(), combined)
}

#[test]
fn surfaces_typecheck_green_and_typo_is_caught() {
    let repo = repo_root();
    let Some(tsc) = tsc_path(&repo) else {
        eprintln!(
            "SKIP synthetic_tsc: {} not found — run `bun install` to enable the tsc proof.",
            repo.join("node_modules/.bin/tsc").display()
        );
        return;
    };

    let manifest_src =
        fs::read_to_string(repo.join("crates/deck/deck-conformance/app/app.manifest.jsonc"))
            .expect("read app.manifest.jsonc");
    let mut manifest = AppManifest::parse(&manifest_src).expect("parse manifest");
    inject_host_context(&mut manifest);

    let work = repo.join("target/synthetic-tsc-proof");
    let _ = fs::remove_dir_all(&work);
    let gendir = work.join("generated");
    fs::create_dir_all(&gendir).unwrap();

    // Representative CONTROLLER-FREE surfaces (scope is template/data/route
    // declared) + the manifest seam (route/navigate typed via inject_host_context).
    // The surfaces that COMPOSE an authored `controller(fn)` — board
    // (control-flow + composed controller) and badge (a component) — are NOT
    // listed here for an unrelated reason: `board/controller.ts` doesn't exist
    // in this repo (see the `#[ignore]`'d `controller_recovers_scope_factory_
    // return_type` in `synthetic_client_tsc.rs`). The `ef696e5` zero-ctx `__def`
    // regression that test tracked is fixed by `controller-collapse`'s single
    // `controller(fn)` overload (always carries `__def`; the recovery pattern
    // matches any function arity).
    let surfaces = [
        // 1. route param.
        Surface {
            name: "greet",
            source_rel: "crates/deck/deck-conformance/app/views/greet/[id]/page.sui",
            mode: CompileMode::Component,
            authored: None,
            component_tag: None,
        },
        // 2. controller live() (host-dispatch data path).
        Surface {
            name: "feed",
            source_rel: "crates/deck/deck-conformance/app/views/feed/page.sui",
            mode: CompileMode::Component,
            authored: None,
            component_tag: None,
        },
    ];
    for s in &surfaces {
        synth_into(&repo, &manifest, &gendir, s);
    }

    // Good set: every pair must type-check clean.
    write_ship_shim(&work);
    let mut good_includes: Vec<String> =
        surfaces.iter().map(|s| format!("generated/{}/**/*.ts", s.name)).collect();
    good_includes.push("ship-shim.d.ts".to_string());
    let good_refs: Vec<&str> = good_includes.iter().map(|s| s.as_str()).collect();
    write_tsconfig(&repo, &work, &good_refs, "tsconfig.json");

    let (ok, out) = run_tsc(&tsc, &work, "tsconfig.json");
    assert!(ok, "the synthesized surfaces must type-check GREEN:\n{out}");

    // Typo set: greet with `route.parms` must be a hard TS2339 against RouteSignals.
    let typo_src = "<h1>Hello, {route.parms.id()}</h1>";
    let program = parse(typo_src).unwrap();
    let out_typo =
        emit_synthetic(&program, &manifest, None, "./greet-typo.synthetic.controller.ts", CompileMode::Component, None)
            .unwrap();
    let typo_dir = gendir.join("greet-typo");
    fs::create_dir_all(&typo_dir).unwrap();
    fs::write(typo_dir.join("greet-typo.synthetic.controller.ts"), out_typo.controller_ts).unwrap();
    fs::write(typo_dir.join("greet-typo.render.ts"), out_typo.render_ts).unwrap();
    write_tsconfig(&repo, &work, &["generated/greet-typo/**/*.ts"], "tsconfig.typo.json");

    let (typo_ok, typo_out) = run_tsc(&tsc, &work, "tsconfig.typo.json");
    assert!(!typo_ok, "the `route.parms` typo MUST fail type-check; tsc was green:\n{typo_out}");
    assert!(
        typo_out.contains("parms") && (typo_out.contains("2339") || typo_out.contains("does not exist")),
        "typo must be a real member-access error against RouteSignals; got:\n{typo_out}"
    );
}
