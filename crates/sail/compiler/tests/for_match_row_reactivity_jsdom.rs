//! Task #34 regression (RED→GREEN pinned): a keyed `{#for}` row reuse must
//! re-select a nested `{#match}` branch when the row's underlying field
//! changes via a signal write — not just when the row's identity/index
//! changes.
//!
//! Root cause: `emit_if`/`emit_match` (crates/sail/compiler/src/emit/
//! template_emit/control_emit.rs) build their `bindIf` `test()` discriminant
//! via plain `self.rw(...)` calls, never routing through
//! `for_version_comma`/`for_version_stmt` (template_emit.rs) the way every
//! other binding kind inside a `{#for}` row does. Keyed row reuse
//! (`packages/sail/runtime/src/for.ts` `bindFor`) reassigns `$scope.<var>` and
//! bumps the row's `__v_<field>` signals — it does NOT re-run the row's
//! render — so a `test()` closure that never reads a `__v_<field>` signal
//! never resubscribes and never re-fires, leaving the stale branch mounted.
//!
//! Opt-in: requires `bun install` so `node_modules/.bin/vitest` exists
//! (mirrors `synthetic_jsdom.rs`).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use sail_compiler::emit::synthetic::{emit_synthetic, AppManifest, AuthoredController};
use sail_compiler::parse::parse;
use sail_compiler::CompileMode;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..").canonicalize().unwrap()
}

#[test]
fn keyed_for_row_reuse_reselects_nested_match_branch() {
    let repo = repo_root();
    let vitest = repo.join("node_modules/.bin/vitest");
    if !vitest.exists() {
        eprintln!(
            "SKIP for_match_row_reactivity_jsdom: {} not found — run `bun install` to enable the jsdom proof.",
            vitest.display()
        );
        return;
    }

    let manifest = AppManifest::parse("{}").unwrap();
    let raw = fs::read_to_string(
        repo.join("crates/sail/compiler/tests/fixtures/for-match-row-reactivity.sui"),
    )
    .unwrap();
    let (src, inline) = sail_compiler::parse::preprocess::extract_controller_block(&raw);
    let program = parse(&src).unwrap();

    let work = repo.join("target/for-match-row-reactivity-jsdom-proof");
    let _ = fs::remove_dir_all(&work);
    fs::create_dir_all(&work).unwrap();
    let authored_path = work.join("fm.authored.ts");
    fs::write(
        &authored_path,
        inline.expect("fixture declares a <controller> block"),
    )
    .unwrap();
    let authored = AuthoredController {
        specifier: authored_path.to_string_lossy().into_owned(),
    };

    let out = emit_synthetic(
        &program,
        &manifest,
        Some(&authored),
        "./fm.synthetic.controller.ts",
        CompileMode::Component,
        None,
    )
    .unwrap();

    fs::write(work.join("fm.synthetic.controller.ts"), &out.controller_ts).unwrap();
    fs::write(work.join("fm.render.ts"), &out.render_ts).unwrap();
    write_harness(&repo, &work);

    let status = Command::new(&vitest)
        .args(["run", "-c", "vitest.config.ts"])
        .current_dir(&work)
        .status()
        .expect("run vitest");
    assert!(
        status.success(),
        "a keyed {{#for}} row reuse must re-select the nested {{#match}} branch on a row-field signal write"
    );
}

fn write_harness(repo: &Path, work: &Path) {
    let r = repo.display();
    let shim = work.join("runtime-shim.ts");
    fs::write(
        &shim,
        format!(
            "export * from \"{r}/packages/sail/runtime/src/server.ts\";\n\
             export {{ markHydrateText, markHydrateAttr, markHydrateBoundary }} from \"{r}/packages/sail/runtime/src/hydrate-mark.ts\";\n\
             export {{ __tpl }} from \"{r}/packages/sail/runtime/src/tpl.ts\";\n\
             export {{ delegateEvents }} from \"{r}/packages/sail/runtime/src/delegate.ts\";\n"
        ),
    )
    .unwrap();
    let cfg = format!(
        r#"import {{ defineConfig }} from 'vitest/config'
export default defineConfig({{
  test: {{ environment: 'jsdom', include: ['mount.test.ts'] }},
  resolve: {{ alias: {{
    '@ship/sail-runtime/internal': '{rt}',
    '@ship/sail-runtime': '{rt}',
  }} }},
}})
"#,
        rt = shim.to_string_lossy(),
    );
    fs::write(work.join("vitest.config.ts"), cfg).unwrap();
    let test = r#"import { describe, it, expect } from 'vitest'
import { mount } from './fm.render.ts'

describe('task #34 — keyed {#for} row reuse re-tests nested {#match}', () => {
  it('toggling a row field (same key) flips the matched branch, not just on reorder', () => {
    const container = document.createElement('div')
    document.body.appendChild(container)
    const dispose = mount(container)

    const rows = () => [...container.querySelectorAll('[data-test="row"]')] as HTMLElement[]
    expect(rows().length).toBe(2)

    const rowA = rows().find((r) => r.getAttribute('data-id') === 'a')!
    const rowB = rows().find((r) => r.getAttribute('data-id') === 'b')!
    expect(rowA.querySelector('[data-test="status"]')?.textContent).toBe('todo')
    expect(rowB.querySelector('[data-test="status"]')?.textContent).toBe('done')

    // Toggle row "a" — the row's KEY does not change, only its `status`
    // field, via an immutable signal write (bindFor's keyed-reuse path).
    const toggleA = rowA.querySelector('[data-test="toggle"]') as HTMLButtonElement
    toggleA.click()

    // The DOM node identity for row "a" must be preserved (keyed reuse —
    // no remount), but its nested {#match} must re-select the 'done' arm.
    const rowsAfter = rows()
    expect(rowsAfter.length).toBe(2)
    const rowAAfter = rowsAfter.find((r) => r.getAttribute('data-id') === 'a')!
    expect(rowAAfter).toBe(rowA) // same DOM node — keyed reuse, not remount
    expect(rowAAfter.querySelector('[data-test="status"]')?.textContent).toBe('done')

    // Row "b" is untouched.
    const rowBAfter = rowsAfter.find((r) => r.getAttribute('data-id') === 'b')!
    expect(rowBAfter.querySelector('[data-test="status"]')?.textContent).toBe('done')

    dispose()
  })
})
"#;
    fs::write(work.join("mount.test.ts"), test).unwrap();
}
