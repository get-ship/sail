//! Gate tests for SPEC-AMBIENT-SURFACE-1 §1 — ambient table completeness.
//!
//! Spike S5.1 red gate. Calls product surfaces only:
//! - `sail_compiler::runtime_ambient::RUNTIME_AMBIENT` (inject SSOT)
//! - `parse::preprocess::wrap_controller_body` / `extract_controller_block`
//! - `ambients::emit_types_dts` + tsc (when `node_modules/typescript/bin/tsc` present)
//!
//! Implementer must not modify, skip, or weaken these tests.
//!
//! Ambient NEW (this spike — must inject + typecheck with zero authored imports):
//! - `interval`, `debounce`
//! - `theme` (preview / commit / applyScope / current surface)
//! - store ops: `defineStore`, `provideStore`, `useStore`
//! - `query`, `mutation`
//!
//! Behavioral law: a controller that references these names without importing
//! them gets a self-contained emit module (ambient inject from
//! `@ship/sail-runtime/internal`). Types (`Signal`, `Theme`, `StoreDef`, …)
//! are free globals via the ambient pack — no `import type` from `@ship/*`.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use sail_compiler::ambients;
use sail_compiler::parse::preprocess::{extract_controller_block, wrap_controller_body};
use sail_compiler::runtime_ambient::{RUNTIME_AMBIENT, RUNTIME_SPECIFIER};

/// SPEC §1 ambient additions this spike must land in the inject table.
const REQUIRED_AMBIENT: &[&str] = &[
    "interval",
    "debounce",
    "theme",
    "defineStore",
    "provideStore",
    "useStore",
    "query",
    "mutation",
];

fn ambient_authored_names() -> HashSet<&'static str> {
    RUNTIME_AMBIENT
        .iter()
        .map(|(authored, _, _)| *authored)
        .collect()
}

fn ambient_import_line(body: &str) -> Option<&str> {
    body.lines().find(|l| {
        let t = l.trim_start();
        t.starts_with("import") && t.contains(RUNTIME_SPECIFIER)
    })
}

fn asserts_ambient_inject(body: &str, idents: &[&str]) {
    let line = ambient_import_line(body).unwrap_or_else(|| {
        panic!("expected ambient inject from `{RUNTIME_SPECIFIER}`, got module:\n{body}")
    });
    for ident in idents {
        assert!(
            line.contains(ident),
            "ambient `{ident}` missing from inject line `{line}`\nmodule:\n{body}"
        );
    }
}

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

// ── 1. Inject table SSOT ────────────────────────────────────────────────────

#[test]
fn runtime_ambient_table_covers_s5_surface() {
    let names = ambient_authored_names();
    let mut missing = Vec::new();
    for need in REQUIRED_AMBIENT {
        if !names.contains(need) {
            missing.push(*need);
        }
    }
    assert!(
        missing.is_empty(),
        "RUNTIME_AMBIENT must include SPEC-AMBIENT-SURFACE-1 §1 names \
         (interval, debounce, theme, store ops, query, mutation). missing: {missing:?}\n\
         table: {:?}",
        {
            let mut v: Vec<_> = names.into_iter().collect();
            v.sort_unstable();
            v
        }
    );
}

// ── 2. Emit inject (no authored import) ─────────────────────────────────────

#[test]
fn wrap_injects_interval_and_debounce() {
    let body = wrap_controller_body(
        r#"
export default controller(() => {
  const ticks = signal(0)
  interval(() => ticks.set(ticks() + 1), 1000)
  const save = debounce((text: string) => { void text }, 300)
  return { ticks, save }
})
"#,
    );
    asserts_ambient_inject(&body, &["controller", "signal", "interval", "debounce"]);
}

#[test]
fn wrap_injects_theme() {
    let body = wrap_controller_body(
        r#"
export default controller(() => {
  const peek = () => theme.current()
  const live = (patch: Record<string, string>) => theme.preview(patch)
  const persist = (patch: Record<string, string>) => theme.commit(patch)
  return { peek, live, persist }
})
"#,
    );
    asserts_ambient_inject(&body, &["controller", "theme"]);
}

#[test]
fn wrap_injects_store_ops() {
    let body = wrap_controller_body(
        r#"
const tally = defineStore('tally', () => ({ n: signal(0) }))
export default controller(() => {
  provideStore(tally)
  const store = useStore(tally)
  return { store }
})
"#,
    );
    asserts_ambient_inject(
        &body,
        &[
            "controller",
            "signal",
            "defineStore",
            "provideStore",
            "useStore",
        ],
    );
}

#[test]
fn wrap_injects_query_and_mutation() {
    let body = wrap_controller_body(
        r#"
export default controller(() => {
  const items = query(async () => [] as string[])
  const save = mutation(async () => undefined as void)
  return { items, save }
})
"#,
    );
    asserts_ambient_inject(&body, &["controller", "query", "mutation"]);
}

#[test]
fn inline_controller_block_injects_new_ambient_names() {
    // Same path ambient_component_controller uses: extract → wrap.
    let sfc = r#"
<controller>
export default controller(() => {
  interval(() => {}, 500)
  const d = debounce(() => {}, 100)
  const items = query(async () => [1])
  const write = mutation(async () => 0)
  const t = theme.current()
  const store = useStore(defineStore('x', () => ({})))
  return { d, items, write, t, store }
})
</controller>
<div></div>
"#;
    let (_, body) = extract_controller_block(sfc);
    let body = body.expect("inline controller");
    // extract already wraps; re-wrap is idempotent for inject.
    let wrapped = wrap_controller_body(&body);
    asserts_ambient_inject(
        &wrapped,
        &[
            "controller",
            "interval",
            "debounce",
            "query",
            "mutation",
            "theme",
            "useStore",
            "defineStore",
        ],
    );
}

// ── 3. Ambient pack types + free value names (tsc) ──────────────────────────

#[test]
fn ambient_pack_declares_s5_value_names() {
    // Pack text is the typecheck surface; inject table is the emit surface.
    // Both must list the §1 names (pack may already declare some — lock it).
    let dts = ambients::emit_types_dts();
    for name in REQUIRED_AMBIENT {
        // theme is `const theme`; functions are `function name` or `const name`.
        let as_fn = format!("function {name}");
        let as_const = format!("const {name}");
        assert!(
            dts.contains(&as_fn) || dts.contains(&as_const),
            "emit_types_dts must declare ambient `{name}` (function or const). \
             pack length={}",
            dts.len()
        );
    }
}

#[test]
fn bare_s5_ambient_values_typecheck_without_import() {
    let repo = repo_root();
    let tsc = require_tsc(&repo);

    let work = repo.join("target/ambient-table-complete-tsc");
    let _ = fs::remove_dir_all(&work);
    fs::create_dir_all(&work).unwrap();

    let ambient = ambients::emit_types_dts();
    fs::write(work.join("sail-types.d.ts"), ambient).unwrap();

    let runtime = repo
        .join("packages/sail/runtime/src/index.ts")
        .display()
        .to_string()
        .replace('\\', "/");
    let runtime_parent = repo
        .join("packages/sail/runtime/src")
        .display()
        .to_string()
        .replace('\\', "/");
    let cfg = format!(
        r#"{{
  "compilerOptions": {{
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "lib": ["ES2022", "DOM", "DOM.Iterable"],
    "strict": true,
    "noImplicitAny": true,
    "skipLibCheck": true,
    "noEmit": true,
    "paths": {{
      "@ship/sail-runtime": ["{runtime}"],
      "@ship/sail-runtime/*": ["{runtime_parent}/*"]
    }}
  }},
  "include": ["./snippet.ts", "./sail-types.d.ts"]
}}"#
    );
    fs::write(work.join("tsconfig.json"), cfg).unwrap();

    // Zero imports. Full §1 surface used bare.
    fs::write(
        work.join("snippet.ts"),
        r#"
export default controller(() => {
  const ticks = signal(0)
  interval(() => ticks.set(ticks() + 1), 1000)
  const save = debounce((s: string) => { void s }, 200)
  const items = query(async () => [] as number[])
  const write = mutation(async () => 1 as number)
  const _peek = theme.current()
  const _prev = theme.preview
  const _commit = theme.commit
  const _scope = theme.applyScope
  const tally = defineStore('tally', () => ({ n: signal(0) }))
  provideStore(tally)
  const store = useStore(tally)
  return { ticks, save, items, write, store, _peek, _prev, _commit, _scope }
})
"#,
    )
    .unwrap();

    let out = Command::new(&tsc)
        .args(["-p", "tsconfig.json", "--noEmit"])
        .current_dir(&work)
        .output()
        .expect("run tsc ambient values");
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        out.status.success(),
        "bare interval/debounce/theme/store/query/mutation must typecheck via ambient pack \
         with zero imports. tsc:\n{combined}"
    );
}

#[test]
fn public_types_are_global_without_ship_import() {
    // SPEC §1: Signal, Theme, Store (StoreDef), … via declare global — no import type.
    let repo = repo_root();
    let tsc = require_tsc(&repo);

    let work = repo.join("target/ambient-table-complete-types-tsc");
    let _ = fs::remove_dir_all(&work);
    fs::create_dir_all(&work).unwrap();

    fs::write(work.join("sail-types.d.ts"), ambients::emit_types_dts()).unwrap();
    let runtime = repo
        .join("packages/sail/runtime/src/index.ts")
        .display()
        .to_string()
        .replace('\\', "/");
    let runtime_parent = repo
        .join("packages/sail/runtime/src")
        .display()
        .to_string()
        .replace('\\', "/");
    fs::write(
        work.join("tsconfig.json"),
        format!(
            r#"{{
  "compilerOptions": {{
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "lib": ["ES2022", "DOM", "DOM.Iterable"],
    "strict": true,
    "skipLibCheck": true,
    "noEmit": true,
    "paths": {{
      "@ship/sail-runtime": ["{runtime}"],
      "@ship/sail-runtime/*": ["{runtime_parent}/*"]
    }}
  }},
  "include": ["./snippet.ts", "./sail-types.d.ts"]
}}"#
        ),
    )
    .unwrap();

    // Free type names — no `import type` from @ship/*.
    fs::write(
        work.join("snippet.ts"),
        r#"
const n: Signal<number> = signal(0)
const th: Theme = theme
const def: StoreDef<{ n: Signal<number> }> = defineStore('n', () => ({ n: signal(0) }))
void n
void th
void def
"#,
    )
    .unwrap();

    let out = Command::new(&tsc)
        .args(["-p", "tsconfig.json", "--noEmit"])
        .current_dir(&work)
        .output()
        .expect("run tsc ambient types");
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        out.status.success(),
        "Signal / Theme / StoreDef must be free global types (no @ship import type). tsc:\n{combined}"
    );
}
