//! tsc proof for the de-Shipped controller scope (#105).
//!
//! Three claims, all checked against the REAL runtime source (no shim of the
//! types under test):
//!
//!  1. POSITIVE — a host's concrete client map, declared the provider way
//!     (`declare module '@ship/sail-runtime' { interface Clients { … } }`),
//!     still satisfies `ControllerScope<Scope>['clients']` and is reachable
//!     with precise member types from a controller. The `clients` seam kept
//!     working when its TYPE moved from `./ship` to the SAIL-local `./clients`
//!     port.
//!  2. NEGATIVE — `ControllerScope` no longer carries a `ship` member.
//!     `ctx.ship` must be a type ERROR; if the key ever gets re-injected into
//!     the generic scope this test goes red.
//!  3. NEGATIVE — the port is still typed, not `any`: a typo on a declared
//!     client (`clients.api.lst()`) is caught.
//!
//! Opt-in like the other tsc proofs: needs `bun install` for
//! `node_modules/.bin/tsc`. Absent tsc → print why and return.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..").canonicalize().unwrap()
}

fn tsc_path(repo: &Path) -> Option<PathBuf> {
    let p = repo.join("node_modules/.bin/tsc");
    p.exists().then_some(p)
}

/// Write a one-file TS project mapping `@ship/sail-runtime` at the real
/// runtime index, and run tsc over it. Returns (green, output).
fn check(repo: &Path, work: &Path, body: &str) -> (bool, String) {
    let _ = fs::remove_dir_all(work);
    fs::create_dir_all(work).unwrap();
    fs::write(work.join("probe.ts"), body).unwrap();
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
      "@ship/sail-runtime": ["{r}/packages/sail/runtime/src/index.ts"]
    }}
  }},
  "include": ["probe.ts"]
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

/// The provider-side augmentation every host does, plus a controller reading
/// its declared client through the scope.
const PRELUDE: &str = r#"
import { controller } from '@ship/sail-runtime'
import type { ControllerScope } from '@ship/sail-runtime'

interface DemoApi {
  list(): Promise<string[]>
}

// The provider augments its OWN key — the framework names no host shape.
declare module '@ship/sail-runtime' {
  interface Clients {
    api: DemoApi
  }
}

interface Scope {
  title: string
}
"#;

#[test]
fn host_client_map_still_satisfies_the_sail_local_port() {
    let repo = repo_root();
    if tsc_path(&repo).is_none() {
        eprintln!("SKIP controller_scope_port_tsc: tsc not found — run `bun install`.");
        return;
    }
    let body = format!(
        "{PRELUDE}
// (1) The port is reachable and PRECISE: `list()` resolves to Promise<string[]>.
export default controller((ctx: ControllerScope<Scope>) => {{
  const rows: Promise<string[]> = ctx.clients.api.list()
  const path: string = ctx.route.pathname()
  return {{ rows, path, title: ctx.title }}
}})

// (1b) A host's CONCRETE map is assignable to the scope's `clients` slot from
// the outside — this is the mount-seam assignment, typed.
const concrete = {{ api: {{ list: async () => ['a'] }} }}
const seam: ControllerScope<Scope>['clients'] = concrete
void seam
"
    );
    let (ok, out) = check(&repo, &repo.join("target/controller-scope-port-tsc/positive"), &body);
    assert!(ok, "the de-Shipped controller scope MUST typecheck green:\n{out}");
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
  // #105 — `ship` is NOT a member of the generic controller scope.
  return {{ host: ctx.ship }}
}})
"
    );
    let (ok, out) = check(&repo, &repo.join("target/controller-scope-port-tsc/no-ship"), &body);
    assert!(
        !ok,
        "`ctx.ship` MUST be a type error — the generic controller scope may not \
         carry a host-product member (#105); tsc was green:\n{out}"
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
