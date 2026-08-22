//! tsc proof for flat host inject (no ship, no clients bag).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use sail_compiler::emit::dts::ScopeCapability;
use sail_compiler::{CompileMode, CompileOpts, compile};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .unwrap()
}

fn require_tsc(repo: &Path) -> PathBuf {
    let p = repo.join("node_modules/typescript/bin/tsc");
    assert!(
        p.is_file(),
        "repo TypeScript wrapper missing at {} — bun install",
        p.display()
    );
    p
}

fn generated_scope_dts() -> String {
    let opts = CompileOpts {
        element_prefix: "sail".into(),
        host_data_attr: "data-sail-host".into(),
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
    compile(r#"<button on:click={noop}>x</button>"#, opts)
        .expect("compile")
        .dts
        .expect("dts emitted")
}

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
    let out = Command::new(require_tsc(repo))
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

const PRELUDE: &str = r#"
import { controller } from '@ship/sail-runtime'
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './scope'

interface DemoApi {
  list(): Promise<string[]>
}

declare module '@ship/sail-runtime' {
  interface Host {
    api: DemoApi
  }
}
"#;

#[test]
fn granted_capabilities_reach_the_controller_precisely() {
    let repo = repo_root();
    let body = format!(
        "{PRELUDE}
export default controller((ctx: ControllerScope<Scope>) => {{
  const rows: Promise<string[]> = ctx.api.list()
  const path: string = ctx.route.pathname()
  const go: (p: string) => void = ctx.navigate
  return {{ rows, path, go }}
}})
"
    );
    let (ok, out) = check(
        &repo,
        &repo.join("target/controller-scope-port-tsc/positive"),
        &body,
    );
    assert!(
        ok,
        "the dissolved controller scope MUST typecheck green:\n{out}"
    );
}

#[test]
fn controller_scope_has_no_ship_member() {
    let repo = repo_root();
    let body = format!(
        "{PRELUDE}
export default controller((ctx: ControllerScope<Scope>) => {{
  return {{ host: ctx.ship }}
}})
"
    );
    let (ok, out) = check(
        &repo,
        &repo.join("target/controller-scope-port-tsc/no-ship"),
        &body,
    );
    assert!(
        !ok,
        "`ctx.ship` MUST be a type error; tsc was green:\n{out}"
    );
    assert!(
        out.contains("ship"),
        "expected failure to name `ship`; got:\n{out}"
    );
}

#[test]
fn controller_scope_has_no_clients_bag() {
    let repo = repo_root();
    let body = format!(
        "{PRELUDE}
export default controller((ctx: ControllerScope<Scope>) => {{
  return {{ bag: ctx.clients }}
}})
"
    );
    let (ok, out) = check(
        &repo,
        &repo.join("target/controller-scope-port-tsc/no-clients"),
        &body,
    );
    assert!(
        !ok,
        "`ctx.clients` MUST be a type error; tsc was green:\n{out}"
    );
    assert!(
        out.contains("clients"),
        "expected failure to name `clients`; got:\n{out}"
    );
}

#[test]
fn typo_on_a_declared_client_is_caught() {
    let repo = repo_root();
    let body = format!(
        "{PRELUDE}
export default controller((ctx: ControllerScope<Scope>) => {{
  return {{ rows: ctx.api.lst() }}
}})
"
    );
    let (ok, out) = check(
        &repo,
        &repo.join("target/controller-scope-port-tsc/typo"),
        &body,
    );
    assert!(!ok, "typo MUST fail; tsc was green:\n{out}");
}

#[test]
fn an_undeclared_ambient_name_is_a_type_error() {
    let repo = repo_root();
    let body = format!(
        "{PRELUDE}
export default controller((ctx: ControllerScope<Scope>) => {{
  return {{ hit: ctx.analytics }}
}})
"
    );
    let (ok, out) = check(
        &repo,
        &repo.join("target/controller-scope-port-tsc/undeclared"),
        &body,
    );
    assert!(!ok, "undeclared MUST fail; tsc was green:\n{out}");
    assert!(
        out.contains("analytics"),
        "expected failure to name `analytics`; got:\n{out}"
    );
}

#[test]
fn scope_mutable_state_and_the_runtime_context_agree() {
    let repo = repo_root();
    let body = format!(
        "{PRELUDE}
import type {{ MutableState }} from '@ship/sail-runtime'
import type {{ ResolverContext }} from '@ship/sail-runtime/internal'

type Ambient = 'route' | 'navigate'

type FromScope = {{ [K in Ambient]: Scope[K] }}
type FromState = {{ [K in Ambient]: MutableState<Scope>[K] }}
declare const fromScope: FromScope
declare const fromState: FromState
const a: FromState = fromScope
const b: FromScope = fromState
void a; void b

declare const ctx: Required<Pick<ResolverContext, Ambient>>
const d: Scope['navigate'] = ctx.navigate
void d
const api: Scope['api'] = null as unknown as Scope['api']
void api
"
    );
    let (ok, out) = check(
        &repo,
        &repo.join("target/controller-scope-port-tsc/agreement"),
        &body,
    );
    assert!(
        ok,
        "Scope/MutableState/ResolverContext MUST agree on route/navigate:\n{out}"
    );
}
