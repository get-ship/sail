//! tsgo/tsc proof for the synthetic CLIENT emit (slice 1, acceptance #1/#2).
//!
//! The SSR proof (`synthetic_tsc.rs`) type-checks the mount-only render. This
//! checks the CLIENT contract — `export default view(() => …)` returning
//! `__mount` + `hydrate` — across a representative real set: hub `/`, a
//! control-flow view, a `<data live>` view, a route-param view, a propless
//! component, and the static view-host view. The client render imports the
//! `view()` wrapper from `@ship/sui-runtime` and the `hydrate*` helpers from
//! `@ship/sui-dom`, so the tsconfig maps both — proving the browser contract
//! resolves against real types, exactly as strictly as authored TS.
//!
//! `@ship/sui-runtime` maps to a tiny shim (the SSR server entry + `view`) so
//! the check stays lightweight; `@ship/sui-dom` maps to its real source.
//!
//! Opt-in: requires `bun install` so `node_modules/.bin/tsc` + the workspace
//! sources exist. Absent tsc → prints why and returns (mirrors `synthetic_tsc`).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use sui_compiler::emit::synthetic::{emit_synthetic, AppManifest, AuthoredController};
use sui_compiler::parse::parse;
use sui_compiler::{compile, CompileMode, CompileOpts};

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
const HUB_MANIFEST: &str = "apps/hub/app.manifest.jsonc";
const VIEWS_MANIFEST: &str = "views/app.manifest.jsonc";

/// Synthesize the CLIENT (controller + client render) for `s` into its own
/// gendir, alongside the view's emitted `.sui.d.ts` and a copy of the authored
/// controller (so the controller's `import type { Scope } from './page.sui'`
/// and `./controller` both resolve locally).
fn synth_client_into(repo: &Path, gendir: &Path, s: &Surface) {
    let src = fs::read_to_string(repo.join(s.source_rel))
        .unwrap_or_else(|e| panic!("read {}: {e}", s.source_rel));
    let manifest_src = fs::read_to_string(repo.join(s.manifest_rel)).expect("read manifest");
    let manifest = AppManifest::parse(&manifest_src).expect("parse manifest");
    let program = parse(&src).unwrap_or_else(|e| panic!("parse {}: {}", s.name, e.render(&src)));

    let dir = gendir.join(s.name);
    fs::create_dir_all(&dir).unwrap();

    // The source stem (page / badge) — the controller imports `./<stem>.sui`.
    let stem = Path::new(s.source_rel).file_stem().and_then(|f| f.to_str()).unwrap_or("page");

    // Emit the view's `.sui.d.ts` so `import { Scope } from './page.sui'`
    // (in some authored controllers) resolves under allowArbitraryExtensions.
    // Views only — component controllers don't import their own `.sui`.
    if s.mode == CompileMode::View {
        let dts = compile(
            &src,
            CompileOpts { mode: s.mode, emit_dts: true, ..Default::default() },
        )
        .unwrap_or_else(|e| panic!("compile dts {}: {}", s.name, e.render(&src)))
        .dts
        .unwrap_or_default();
        fs::write(dir.join(format!("{stem}.sui.d.ts")), dts).unwrap();
    }

    // Copy the authored controller into the gendir (so its relative imports
    // resolve next to the emitted dts), and compose it.
    let authored = if let Some(c) = s.controller_rel {
        let body = fs::read_to_string(repo.join(c)).expect("read controller");
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

/// Write the `@ship/sui-runtime` shim: the SSR server surface + the `view()`
/// wrapper (which the server entry omits). Lightweight — avoids pulling the
/// full browser index graph into the check.
fn write_runtime_shim(repo: &Path, work: &Path) {
    let r = repo.display();
    let shim = format!(
        "export * from \"{r}/packages/sui/runtime/src/server.ts\";\n\
         export {{ view }} from \"{r}/packages/sui/runtime/src/view.ts\";\n\
         // Slice 2 — re-export the hydration-marker recording helpers so\n\
         // the synthetic emit's `mount(container)` (which calls\n\
         // `markHydrateText` next to `bindText`) type-checks under tsc.\n\
         export {{ markHydrateText, markHydrateAttr, markHydrateBoundary }} from \"{r}/packages/sui/runtime/src/hydrate-mark.ts\";\n\
         // Types the @ship/sui-dom hydrate helpers import from the runtime but\n\
         // that the SSR server entry does not surface.\n\
         export type {{ Modifier, OnOptions }} from \"{r}/packages/sui/runtime/src/on.ts\";\n\
         // `ViewScope` lives on controller.ts (the legacy export authors still\n\
         // reach for). Re-export so a controller declaring\n\
         // `controller<ViewScope>(...)` doesn't break under the shim.\n\
         export type {{ ViewScope }} from \"{r}/packages/sui/runtime/src/controller.ts\";\n"
    );
    fs::write(work.join("runtime-shim.ts"), shim).unwrap();
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
      "@ship/sui-runtime": ["./runtime-shim.ts"],
      "@ship/sui-runtime/*": ["{r}/packages/sui/runtime/src/*"],
      "@ship/sui-dom": ["{r}/packages/sui/dom/src/index.ts"],
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

    // The strict-green set. These exercise the full CLIENT contract — view()
    // wrapper, __mount (CSR), hydrate (text/element/boundary/while markers),
    // typed scope reads, the @ship/sui-dom helper imports — with author-sound
    // typing. (hub/counter over-constrain `controller<Scope>` with the empty
    // view Scope, erasing the controller-added member from the recovered type;
    // todos relies on LiveState narrowing across an `{#if kind==='ok'}` the SSR
    // render shares — both are catalogued separately, not client-contract bugs.)
    let surfaces = [
        // board — control flow: {#if}/{#for}/class:/on:/nested component + composed controller.
        Surface {
            name: "board",
            source_rel: "crates/deck/deck-conformance/app/views/board/page.sui",
            manifest_rel: CONF_MANIFEST,
            mode: CompileMode::View,
            controller_rel: Some("crates/deck/deck-conformance/app/views/board/controller.ts"),
        },
        // feed — {#while-loading}/{#while-error} + <data live> + <client>.
        Surface {
            name: "feed",
            source_rel: "crates/deck/deck-conformance/app/views/feed/page.sui",
            manifest_rel: CONF_MANIFEST,
            mode: CompileMode::View,
            controller_rel: None,
        },
        // greet/[id] — dynamic route params (the typed-scope typo-catch surface).
        Surface {
            name: "greet",
            source_rel: "crates/deck/deck-conformance/app/views/greet/[id]/page.sui",
            manifest_rel: CONF_MANIFEST,
            mode: CompileMode::View,
            controller_rel: None,
        },
        // badge — a propless component, with controller (the "real component" CSR target).
        Surface {
            name: "badge",
            source_rel: "crates/deck/deck-conformance/app/components/badge/badge.sui",
            manifest_rel: CONF_MANIFEST,
            mode: CompileMode::Component,
            controller_rel: Some("crates/deck/deck-conformance/app/components/badge/badge.controller.ts"),
        },
        // showcase — the static view-host view.
        Surface {
            name: "showcase",
            source_rel: "views/showcase/page.sui",
            manifest_rel: VIEWS_MANIFEST,
            mode: CompileMode::View,
            controller_rel: None,
        },
    ];

    for s in &surfaces {
        synth_client_into(&repo, &gendir, s);
    }

    let includes: Vec<String> =
        surfaces.iter().map(|s| format!("generated/{}/**/*.ts", s.name)).collect();
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
    let manifest = AppManifest::parse(&manifest_src).unwrap();
    let program = parse("<h1>Hello, {route.parms.id()}</h1>").unwrap();
    let out = emit_synthetic(&program, &manifest, None, "./page.synthetic.controller.ts", CompileMode::View, None)
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

/// Slice 2 G4 — hub's authored controller declares
/// `controller<Scope>({...})` with the EMPTY view `Scope` (the view
/// declares no `<state>`), so the legacy `ControllerExport<infer S>`
/// recovery would erase the controller-added `name` from the recovered
/// type — `scope.name()` would be a TS2339. The slice 2 G4 fix:
/// recover the scope-factory's ACTUAL return type instead of the
/// `ControllerExport<Scope>` generic. The hub render is now strict-tsc
/// GREEN. (Slice 1 catalogued the opposite — this is the slice-2
/// acceptance proof.)
#[test]
fn hub_controller_recovers_scope_factory_return_type() {
    let repo = repo_root();
    let Some(tsc) = tsc_path(&repo) else {
        eprintln!("SKIP synthetic_client_tsc hub: tsc not found — run `bun install`.");
        return;
    };
    let work = repo.join("target/synthetic-client-tsc-hub");
    let _ = fs::remove_dir_all(&work);
    let gendir = work.join("generated");
    fs::create_dir_all(&gendir).unwrap();
    write_runtime_shim(&repo, &work);
    synth_client_into(
        &repo,
        &gendir,
        &Surface {
            name: "hub",
            source_rel: "apps/hub/routes/page.sui",
            manifest_rel: HUB_MANIFEST,
            mode: CompileMode::View,
            controller_rel: Some("apps/hub/routes/controller.ts"),
        },
    );
    write_tsconfig(&repo, &work, &["generated/hub/**/*.ts".to_string()], "tsconfig.json");
    let (ok, output) = run_tsc(&tsc, &work, "tsconfig.json");
    assert!(
        ok,
        "slice 2 G4: hub must type-check GREEN (scope-factory return type is recovered); \
         got:\n{output}"
    );
}
