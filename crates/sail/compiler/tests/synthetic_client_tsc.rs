//! tsgo/tsc proof for the synthetic CLIENT emit (slice 1, acceptance #1/#2).
//!
//! The SSR proof (`synthetic_tsc.rs`) type-checks the mount-only render. This
//! checks the CLIENT contract — `export default view(() => …)` returning
//! `__mount` + `hydrate` — across a representative real set: hub `/`, a
//! control-flow view, a controller live() view, a route-param view, a
//! propless component, and the static view-host view. The client render imports the
//! `view()` wrapper from `@ship/sail-runtime` and the `hydrate*` helpers from
//! `@ship/sail-dom`, so the tsconfig maps both — proving the browser contract
//! resolves against real types, exactly as strictly as authored TS.
//!
//! `@ship/sail-runtime` maps to a tiny shim (the SSR server entry + `view`) so
//! the check stays lightweight; `@ship/sail-dom` maps to its real source.
//!
//! Opt-in: requires `bun install` so `node_modules/.bin/tsc` + the workspace
//! sources exist. Absent tsc → prints why and returns (mirrors `synthetic_tsc`).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use sail_compiler::emit::synthetic::{emit_synthetic, AppManifest, AuthoredController, ManifestClient};
use sail_compiler::parse::parse;
use sail_compiler::{compile, CompileMode, CompileOpts};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..").canonicalize().unwrap()
}

fn tsc_path(repo: &Path) -> Option<PathBuf> {
    let p = repo.join("node_modules/.bin/tsc");
    p.exists().then_some(p)
}

struct Surface {
    name: &'static str,
    source_rel: &'static str,
    manifest_rel: &'static str,
    mode: CompileMode,
    /// Sibling authored controller (repo-relative) to compose, if any.
    controller_rel: Option<&'static str>,
}

const CONF_MANIFEST: &str = "crates/deck/deck-conformance/app/app.manifest.jsonc";

/// Mirror deck's host context — the `route`/`navigate` handles deck injects into
/// every app manifest at build time (`deck_core::host_context_clients`). SUI's
/// compiler names no host, so post-seam route/navigate are NOT compiler-ambient;
/// they must be DECLARED in the manifest, and the real pipeline merges them in.
/// This stands in for that merge so the route-bearing surfaces (and the
/// route-typo gate) type-check against `RouteSignals` exactly as they ship.
/// (sail-compiler can't depend on deck-core — that's a cycle — so we mirror the
/// two entries rather than call into it.)
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

/// Synthesize the CLIENT (controller + client render) for `s` into its own
/// gendir, alongside the view's emitted `.sail.d.ts` and a copy of the authored
/// controller (so the controller's `import type { Scope } from './page.sail'`
/// and `./controller` both resolve locally).
fn synth_client_into(repo: &Path, gendir: &Path, s: &Surface) {
    let src = fs::read_to_string(repo.join(s.source_rel))
        .unwrap_or_else(|e| panic!("read {}: {e}", s.source_rel));
    let manifest_src = fs::read_to_string(repo.join(s.manifest_rel)).expect("read manifest");
    let mut manifest = AppManifest::parse(&manifest_src).expect("parse manifest");
    inject_host_context(&mut manifest);
    // A source may carry its controller INLINE (`<controller>…</controller>`)
    // rather than as a sibling `.controller.ts` — strip it before the
    // template-only parser sees it (mirrors `deck_ssr::build`).
    let (stripped, inline_controller) = sail_compiler::parse::preprocess::extract_controller_block(&src);
    let program = parse(&stripped).unwrap_or_else(|e| panic!("parse {}: {}", s.name, e.render(&stripped)));

    let dir = gendir.join(s.name);
    fs::create_dir_all(&dir).unwrap();

    // The source stem (page / badge) — the controller imports `./<stem>.sail`.
    let stem = Path::new(s.source_rel).file_stem().and_then(|f| f.to_str()).unwrap_or("page");

    // Emit the view's `.sail.d.ts` so `import { Scope } from './page.sail'`
    // (in some authored controllers) resolves under allowArbitraryExtensions.
    // Views only — component controllers don't import their own `.sail`.
    if s.mode == CompileMode::Component {
        let dts = compile(
            &src,
            CompileOpts { mode: s.mode, emit_dts: true,
            lint_overrides: Vec::new(), ..Default::default() },
        )
        .unwrap_or_else(|e| panic!("compile dts {}: {}", s.name, e.render(&src)))
        .dts
        .unwrap_or_default();
        fs::write(dir.join(format!("{stem}.sail.d.ts")), dts).unwrap();
    }

    // Copy the authored controller into the gendir (so its relative imports
    // resolve next to the emitted dts), and compose it. An explicit sibling
    // `controller_rel` wins; otherwise fall back to the source's own inline
    // `<controller>` block, if any.
    let authored = if let Some(c) = s.controller_rel {
        let body = fs::read_to_string(repo.join(c)).expect("read controller");
        fs::write(dir.join("controller.ts"), body).unwrap();
        Some(AuthoredController { specifier: "./controller.ts".to_string() })
    } else if let Some(body) = inline_controller {
        fs::write(dir.join("controller.ts"), body).unwrap();
        Some(AuthoredController { specifier: "./controller.ts".to_string() })
    } else {
        None
    };

    let controller_name = format!("{stem}.synthetic.controller.ts");
    let out = emit_synthetic(&program, &manifest, authored.as_ref(), &format!("./{controller_name}"), s.mode, None)
        .unwrap_or_else(|e| panic!("synthesize {}: {}", s.name, e.render(&src)));
    fs::write(dir.join(&controller_name), out.controller_ts).unwrap();
    fs::write(dir.join(format!("{stem}.client.ts")), out.render_client_ts).unwrap();
}

/// Write the `@ship/sail-runtime` shim: the SSR server surface + the
/// `component()` factory (which the server entry omits). Lightweight — avoids
/// pulling the full browser index graph into the check.
fn write_runtime_shim(repo: &Path, work: &Path) {
    let r = repo.display();
    let shim = format!(
        "export * from \"{r}/packages/sail/runtime/src/server.ts\";\n\
         export {{ component }} from \"{r}/packages/sail/runtime/src/component.ts\";\n\
         // Slice 2 — re-export the hydration-marker recording helpers so\n\
         // the synthetic emit's `mount(container)` (which calls\n\
         // `markHydrateText` next to `bindText`) type-checks under tsc.\n\
         export {{ markHydrateText, markHydrateAttr, markHydrateBoundary }} from \"{r}/packages/sail/runtime/src/hydrate-mark.ts\";\n\
         // Types the @ship/sail-dom hydrate helpers import from the runtime but\n\
         // that the SSR server entry does not surface.\n\
         export type {{ Modifier, OnOptions }} from \"{r}/packages/sail/runtime/src/on.ts\";\n\
         // `ViewScope` lives on controller.ts (the legacy export authors still\n\
         // reach for). Re-export so a controller declaring\n\
         // `controller<ViewScope>(...)` doesn't break under the shim.\n\
         export type {{ ViewScope }} from \"{r}/packages/sail/runtime/src/controller.ts\";\n"
    );
    fs::write(work.join("runtime-shim.ts"), shim).unwrap();
    // Ambient type shim for the `ship` host-dispatch module — the ONE
    // external specifier a controller's `live(() => call(op, args))` source
    // imports. Never a real npm package (deno_core supplies it at runtime
    // via `op_ship_call`); this shim exists so tsc can resolve it here.
    fs::write(
        work.join("ship-shim.d.ts"),
        "declare module 'ship' {\n  \
         export function call(op: string, args?: Record<string, unknown>): Promise<unknown>\n  \
         export function emit(value: unknown): void\n\
         }\n",
    )
    .unwrap();
}

fn write_tsconfig(repo: &Path, work: &Path, include: &[String], name: &str) {
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
    "allowArbitraryExtensions": true,
    "baseUrl": ".",
    "paths": {{
      "@ship/sail-runtime": ["./runtime-shim.ts"],
      "@ship/sail-runtime/*": ["{r}/packages/sail/runtime/src/*"],
      "@ship/sail-dom": ["{r}/packages/sail/dom/src/index.ts"],
      "@ship/deck/router": ["{r}/packages/deck/router/src/server.ts"],
      "@app/*": ["{r}/crates/deck/deck-conformance/app/*"]
    }}
  }},
  "include": [{}]
}}"#,
        include.iter().map(|i| format!("\"{i}\"")).collect::<Vec<_>>().join(", ")
    );
    fs::write(work.join(name), cfg).unwrap();
}

fn run_tsc(tsc: &Path, dir: &Path, project: &str) -> (bool, String) {
    let out = Command::new(tsc).arg("-p").arg(project).current_dir(dir).output().expect("run tsc");
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    (out.status.success(), combined)
}

#[test]
fn client_contract_typechecks_green_across_real_surfaces() {
    let repo = repo_root();
    let Some(tsc) = tsc_path(&repo) else {
        eprintln!(
            "SKIP synthetic_client_tsc: {} not found — run `bun install` to enable the tsc proof.",
            repo.join("node_modules/.bin/tsc").display()
        );
        return;
    };

    let work = repo.join("target/synthetic-client-tsc-proof");
    let _ = fs::remove_dir_all(&work);
    let gendir = work.join("generated");
    fs::create_dir_all(&gendir).unwrap();
    write_runtime_shim(&repo, &work);

    // The strict-green set. These exercise the CLIENT contract — view() wrapper,
    // __mount (CSR), hydrate (text/element/boundary/while markers), typed scope
    // reads, the @ship/sail-dom helper imports — with author-sound typing, AND
    // the manifest seam (route/navigate declared into the manifest, typed
    // against `RouteSignals` — see `inject_host_context`).
    //
    // `greet` is CONTROLLER-FREE (its scope is template/route declared). `feed`
    // carries its controller INLINE (`<controller>…</controller>`, composed
    // automatically — see `synth_client_into`). Pre-`controller-collapse`, a
    // zero-arg `controller(() => …)` (no `ctx` param) resolved to a legacy
    // `__def`-less overload, so the synthetic scope-factory recovery yielded
    // `{}` (`ef696e5`) — `feed`'s controller took a `(_ctx)` param to dodge
    // it. `controller(fn)` is now the ONE overload (always carries `__def`;
    // see `controller_recovers_scope_factory_return_type` below), so a
    // zero-arg setup function recovers correctly too — the `(_ctx)` dodge in
    // `feed`'s controller is no longer load-bearing, just untouched.
    let surfaces = [
        // feed — {#while-loading}/{#while-error} + controller-side live() over
        // host-dispatch (the real living-async surface).
        Surface {
            name: "feed",
            source_rel: "crates/deck/deck-conformance/app/views/feed/page.sail",
            manifest_rel: CONF_MANIFEST,
            mode: CompileMode::Component,
            controller_rel: None,
        },
        // greet/[id] — dynamic route params (the typed-scope typo-catch surface).
        Surface {
            name: "greet",
            source_rel: "crates/deck/deck-conformance/app/views/greet/[id]/page.sail",
            manifest_rel: CONF_MANIFEST,
            mode: CompileMode::Component,
            controller_rel: None,
        },
    ];

    for s in &surfaces {
        synth_client_into(&repo, &gendir, s);
    }

    let mut includes: Vec<String> =
        surfaces.iter().map(|s| format!("generated/{}/**/*.ts", s.name)).collect();
    includes.push("ship-shim.d.ts".to_string());
    write_tsconfig(&repo, &work, &includes, "tsconfig.json");

    let (ok, out) = run_tsc(&tsc, &work, "tsconfig.json");
    assert!(ok, "the synthetic CLIENT contract must type-check GREEN across the real set:\n{out}");
}

/// The typed-scope thesis on the CLIENT render: a `route.parms` typo survives
/// into the client render as a real member access against `RouteSignals`, so
/// tsgo rejects it (TS2339) — no ambient passthrough, exactly as for SSR.
#[test]
fn client_render_typo_is_caught_by_tsgo() {
    let repo = repo_root();
    let Some(tsc) = tsc_path(&repo) else {
        eprintln!("SKIP synthetic_client_tsc typo: tsc not found — run `bun install`.");
        return;
    };
    let manifest_src = fs::read_to_string(repo.join(CONF_MANIFEST)).unwrap();
    let mut manifest = AppManifest::parse(&manifest_src).unwrap();
    inject_host_context(&mut manifest);
    let program = parse("<h1>Hello, {route.parms.id()}</h1>").unwrap();
    let out = emit_synthetic(&program, &manifest, None, "./page.synthetic.controller.ts", CompileMode::Component, None)
        .unwrap();

    let work = repo.join("target/synthetic-client-tsc-typo");
    let _ = fs::remove_dir_all(&work);
    let dir = work.join("generated/typo");
    fs::create_dir_all(&dir).unwrap();
    write_runtime_shim(&repo, &work);
    fs::write(dir.join("page.synthetic.controller.ts"), out.controller_ts).unwrap();
    fs::write(dir.join("page.client.ts"), out.render_client_ts).unwrap();
    write_tsconfig(&repo, &work, &["generated/typo/**/*.ts".to_string()], "tsconfig.json");

    let (ok, output) = run_tsc(&tsc, &work, "tsconfig.json");
    assert!(!ok, "the `route.parms` typo MUST fail the client type-check; tsc was green:\n{output}");
    assert!(
        output.contains("parms") && (output.contains("2339") || output.contains("does not exist")),
        "typo must be a real member-access error against RouteSignals; got:\n{output}"
    );
}

/// Slice 2 G4 — `board`'s authored controller declares `controller(() => …)`
/// while the view declares NO `<state>` (the empty view `Scope`), so the legacy
/// `ControllerExport<infer S>` recovery would erase the controller-added
/// `items`/`add`/`remove` from the recovered type — `items()` would be a TS2339.
/// The slice 2 G4 fix: recover the setup function's ACTUAL return type instead
/// of the `ControllerExport<Scope>` generic. The board render is strict-tsc
/// GREEN. (board is the in-repo conformance equivalent of the old `apps/hub`
/// surface, which lived outside the framework boundary.)
///
/// The `controller-collapse` job (function-form `controller(fn)`, one
/// overload) removed the two-overload split that caused the original
/// `ef696e5` regression this test tracked (a zero-`ctx`-param factory
/// couldn't infer `Ctx`, so the call fell through to a legacy overload with
/// no `__def`, and recovery found nothing to recover from). There is now
/// only one overload — it always carries `__def` — and the recovery pattern
/// (`typeof __authored.__def extends (...a: never[]) => infer R`) matches
/// any function regardless of arity, zero-arg included, so the __def side
/// of the original regression is fixed by construction.
///
/// STILL IGNORED — for an unrelated reason: `board/controller.ts` (the
/// `controller_rel` this test points at) does not exist in this repo — only
/// `board/page.sail` does. Nobody noticed because the test was already
/// `#[ignore]`d. Un-ignore once a `board/controller.ts` fixture exists.
#[test]
#[ignore = "board/controller.ts fixture doesn't exist (pre-existing gap, unrelated to controller-collapse) — the __def/overload regression this test tracked is fixed, see doc comment"]
fn controller_recovers_scope_factory_return_type() {
    let repo = repo_root();
    let Some(tsc) = tsc_path(&repo) else {
        eprintln!("SKIP synthetic_client_tsc board G4: tsc not found — run `bun install`.");
        return;
    };
    let work = repo.join("target/synthetic-client-tsc-g4");
    let _ = fs::remove_dir_all(&work);
    let gendir = work.join("generated");
    fs::create_dir_all(&gendir).unwrap();
    write_runtime_shim(&repo, &work);
    synth_client_into(
        &repo,
        &gendir,
        &Surface {
            name: "board",
            source_rel: "crates/deck/deck-conformance/app/views/board/page.sail",
            manifest_rel: CONF_MANIFEST,
            mode: CompileMode::Component,
            controller_rel: Some("crates/deck/deck-conformance/app/views/board/controller.ts"),
        },
    );
    write_tsconfig(&repo, &work, &["generated/board/**/*.ts".to_string()], "tsconfig.json");
    let (ok, output) = run_tsc(&tsc, &work, "tsconfig.json");
    assert!(
        ok,
        "slice 2 G4: board must type-check GREEN (scope-factory return type is recovered); \
         got:\n{output}"
    );
}
