//! tsc proof for the dissolved controller scope (#105, #109).
//!
//! Every claim is checked against the REAL runtime source (no shim of the types
//! under test) and, since #109, against the REAL emitted `.sail.d.ts` — the
//! `Scope` here is `emit_dts`' own output for a file with a host grant, not a
//! hand-written stand-in. If the emitter and the runtime disagree, tsc says so.
//!
//!  1. POSITIVE — the generated `Scope` carries the host grant: the `clients`
//!     map, each declared client by name, `route`, and `navigate`. A host's
//!     concrete client map (declared the provider way,
//!     `declare module '@ship/sail-runtime' { interface Clients { … } }`) is
//!     still assignable to the scope's `clients` slot and reachable with
//!     precise member types.
//!  2. NEGATIVE — no `ship` member. `ctx.ship` must be a type ERROR; if the key
//!     ever gets re-injected this test goes red.
//!  3. NEGATIVE — the port is still typed, not `any`: a typo on a declared
//!     client (`clients.api.lst()`) is caught.
//!  4. NEGATIVE — an ambient name the app never DECLARED (`ctx.analytics`) is a
//!     type error. This is the point of dissolving the wrapper: the capability
//!     list is the contract, not a hand-maintained set of framework members.
//!  5. AGREEMENT — the three places the ambient members are spelled (the
//!     emitted `Scope`, `MutableState`'s ambient block, and the runtime
//!     `baseScope` via the `ResolverContext` it is built from) type-check as
//!     mutually compatible, `navigate` included. `navigate` was the member the
//!     hand-maintained `ControllerScope` wrapper never had; nothing forced the
//!     three to agree, so nobody noticed. Now something does.
//!
//! Opt-in like the other tsc proofs: needs `bun install` for
//! `node_modules/.bin/tsc`. Absent tsc → print why and return.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use sail_compiler::emit::dts::ScopeCapability;
use sail_compiler::{compile, CompileMode, CompileOpts};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..").canonicalize().unwrap()
}

fn tsc_path(repo: &Path) -> Option<PathBuf> {
    let p = repo.join("node_modules/.bin/tsc");
    p.exists().then_some(p)
}

/// The emitted `.sail.d.ts` for a view whose host grant is deck's context
/// (`route`/`navigate`, typed by the SAIL-local ports) plus one app-declared
/// data client. This is the compiler's real output — the probes below import
/// `Scope` from it, so these are end-to-end proofs, not restatements.
fn generated_scope_dts() -> String {
    let opts = CompileOpts {
        route_params: None,
        setup_url: None,
        mode: CompileMode::Component,
        source_path: None,
        component_meta: None,
        emit_dts: true,
        scope_capabilities: vec![
            ScopeCapability::client("api"),
            ScopeCapability {
                name: "navigate".into(),
                ts_type: "(path: string) => void".into(),
                ts_import: None,
            },
            ScopeCapability {
                name: "route".into(),
                ts_type: "RouteSignals".into(),
                ts_import: Some(("RouteSignals".into(), "@ship/sail-runtime".into())),
            },
        ],
        emit_component_imports: false,
        lint_overrides: Vec::new(),
    };
    // A template-declared member (`bind:this`) rides along, so the probes can
    // check that the grant is ADDITIVE to the file's own declarations.
    compile(r#"<button bind:this={btnEl}>x</button>"#, opts)
        .expect("compile")
        .dts
        .expect("dts emitted")
}

/// Write a TS project mapping `@ship/sail-runtime` at the real runtime index —
/// `probe.ts` plus the GENERATED `scope.d.ts` it imports `Scope` from — and run
/// tsc over it. Returns (green, output).
fn check(repo: &Path, work: &Path, body: &str) -> (bool, String) {
    let _ = fs::remove_dir_all(work);
    fs::create_dir_all(work).unwrap();
    fs::write(work.join("probe.ts"), body).unwrap();
    fs::write(work.join("scope.d.ts"), generated_scope_dts()).unwrap();
    let r = repo.display();
    fs::write(
        work.join("tsconfig.json"),
        format!(
            r#"{{
  "compilerOptions": {{
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "lib": ["ES2022", "DOM", "DOM.Iterable"],
    "types": [],
    "strict": true,
    "noEmit": true,
    "skipLibCheck": true,
    "allowImportingTsExtensions": true,
    "baseUrl": ".",
    "paths": {{
      "@ship/sail-runtime": ["{r}/packages/sail/runtime/src/index.ts"],
      "@ship/sail-runtime/internal": ["{r}/packages/sail/runtime/src/internal.ts"]
    }}
  }},
  "include": ["probe.ts", "scope.d.ts"]
}}"#
        ),
    )
    .unwrap();
    let out = Command::new(tsc_path(repo).unwrap())
        .arg("-p")
        .arg("tsconfig.json")
        .current_dir(work)
        .output()
        .expect("run tsc");
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    (out.status.success(), combined)
}

/// The provider-side augmentation every host does, plus the generated `Scope`.
/// Authored controllers still annotate `ControllerScope<Scope>` — an identity
/// alias since #109 — so this prelude is also the zero-edit migration proof.
const PRELUDE: &str = r#"
import { controller } from '@ship/sail-runtime'
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './scope'

interface DemoApi {
  list(): Promise<string[]>
}

// The provider augments its OWN key — the framework names no host shape.
declare module '@ship/sail-runtime' {
  interface Clients {
    api: DemoApi
  }
}
"#;

#[test]
fn granted_capabilities_reach_the_controller_precisely() {
    let repo = repo_root();
    if tsc_path(&repo).is_none() {
        eprintln!("SKIP controller_scope_port_tsc: tsc not found — run `bun install`.");
        return;
    }
    let body = format!(
        "{PRELUDE}
// (1) Every granted capability is reachable and PRECISE, through BOTH paths the
// runtime provides: the `clients` map and the per-name members spread from it.
export default controller((ctx: ControllerScope<Scope>) => {{
  const rows: Promise<string[]> = ctx.clients.api.list()
  const spread: Promise<string[]> = ctx.api.list()
  const path: string = ctx.route.pathname()
  const go: (p: string) => void = ctx.navigate
  return {{ rows, spread, path, go, btnEl: ctx.btnEl }}
}})

// (1b) A host's CONCRETE map is assignable to the scope's `clients` slot from
// the outside — this is the mount-seam assignment, typed.
const concrete = {{ api: {{ list: async () => ['a'] }} }}
const seam: Scope['clients'] = concrete
void seam
"
    );
    let (ok, out) = check(&repo, &repo.join("target/controller-scope-port-tsc/positive"), &body);
    assert!(ok, "the dissolved controller scope MUST typecheck green:\n{out}");
}

#[test]
fn controller_scope_has_no_ship_member() {
    let repo = repo_root();
    if tsc_path(&repo).is_none() {
        eprintln!("SKIP controller_scope_port_tsc: tsc not found — run `bun install`.");
        return;
    }
    let body = format!(
        "{PRELUDE}
export default controller((ctx: ControllerScope<Scope>) => {{
  // #105 — `ship` is NOT a member of the view scope.
  return {{ host: ctx.ship }}
}})
"
    );
    let (ok, out) = check(&repo, &repo.join("target/controller-scope-port-tsc/no-ship"), &body);
    assert!(
        !ok,
        "`ctx.ship` MUST be a type error — the view scope may not carry a \
         host-product member (#105); tsc was green:\n{out}"
    );
    assert!(
        out.contains("ship"),
        "expected the failure to name `ship`; got:\n{out}"
    );
}

#[test]
fn typo_on_a_declared_client_is_caught() {
    let repo = repo_root();
    if tsc_path(&repo).is_none() {
        eprintln!("SKIP controller_scope_port_tsc: tsc not found — run `bun install`.");
        return;
    }
    let body = format!(
        "{PRELUDE}
export default controller((ctx: ControllerScope<Scope>) => {{
  return {{ rows: ctx.clients.api.lst() }}
}})
"
    );
    let (ok, out) = check(&repo, &repo.join("target/controller-scope-port-tsc/typo"), &body);
    assert!(
        !ok,
        "a typo on a declared client MUST fail — the port is typed, not `any`; \
         tsc was green:\n{out}"
    );
}

#[test]
fn an_undeclared_ambient_name_is_a_type_error() {
    let repo = repo_root();
    if tsc_path(&repo).is_none() {
        eprintln!("SKIP controller_scope_port_tsc: tsc not found — run `bun install`.");
        return;
    }
    let body = format!(
        "{PRELUDE}
export default controller((ctx: ControllerScope<Scope>) => {{
  // #109 — `analytics` is in no capability list this view was granted. Under
  // the old hand-maintained wrapper the framework decided which ambient members
  // existed; now the DECLARED list does, and an undeclared read is caught.
  return {{ hit: ctx.analytics }}
}})
"
    );
    let (ok, out) =
        check(&repo, &repo.join("target/controller-scope-port-tsc/undeclared"), &body);
    assert!(
        !ok,
        "reading a capability the app never declared MUST be a type error \
         (#109); tsc was green:\n{out}"
    );
    assert!(out.contains("analytics"), "expected the failure to name `analytics`; got:\n{out}");
}

#[test]
fn scope_mutable_state_and_the_runtime_context_agree() {
    let repo = repo_root();
    if tsc_path(&repo).is_none() {
        eprintln!("SKIP controller_scope_port_tsc: tsc not found — run `bun install`.");
        return;
    }
    let body = format!(
        "{PRELUDE}
import type {{ MutableState }} from '@ship/sail-runtime'
import type {{ ResolverContext }} from '@ship/sail-runtime/internal'

// (5) The three spellings of the ambient members must agree. `navigate` is the
// one that used to drift: it rode the runtime `baseScope` and `MutableState`
// but was missing from the hand-maintained `ControllerScope` wrapper, and
// nothing forced a comparison. These assignments are that comparison.
type Ambient = 'clients' | 'route' | 'navigate'

// Scope (the compiler's emit) ⟷ MutableState's ambient block.
type FromScope = {{ [K in Ambient]: Scope[K] }}
type FromState = {{ [K in Ambient]: MutableState<Scope>[K] }}
declare const fromScope: FromScope
declare const fromState: FromState
const a: FromState = fromScope
const b: FromScope = fromState
void a; void b

// Scope ⟷ the runtime `baseScope`, via the `ResolverContext` it is built from
// (`{{ clients, ...clients, route, navigate, ...scope }}`). The context's members
// are optional — the resolver defaults them — so compare the defined halves.
declare const ctx: Required<Pick<ResolverContext, Ambient>>
const c: Scope['clients'] = ctx.clients
const d: Scope['navigate'] = ctx.navigate
void c; void d
"
    );
    let (ok, out) = check(&repo, &repo.join("target/controller-scope-port-tsc/agreement"), &body);
    assert!(
        ok,
        "the emitted Scope, MutableState's ambient block and the runtime \
         resolver context MUST agree on `clients`/`route`/`navigate`:\n{out}"
    );
}
