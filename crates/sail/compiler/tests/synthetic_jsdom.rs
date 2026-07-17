//! jsdom client-mount proof for the synthetic emitter (acceptance #3).
//!
//! The SAME synthesized render that server-renders through ship-js must also
//! client-mount and stay reactive in a browser DOM. This synthesizes the
//! control-flow `board` surface, then drives the workspace `vitest` in a jsdom
//! environment to `mount()` it and assert: the {#if}/{#for} DOM builds, the
//! nested component + class: directive render, and an on:click handler mutates
//! the controller signal so the list re-renders (proving live reactivity, not
//! just first paint).
//!
//! Opt-in by environment: requires `bun install` so `node_modules/.bin/vitest`
//! exists. When it is absent the test prints why and returns (mirrors
//! `parity.rs` / `synthetic_tsc.rs`).

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
fn synthesized_board_client_mounts_and_stays_reactive() {
    let repo = repo_root();
    let vitest = repo.join("node_modules/.bin/vitest");
    if !vitest.exists() {
        eprintln!(
            "SKIP synthetic_jsdom: {} not found — run `bun install` to enable the jsdom proof.",
            vitest.display()
        );
        return;
    }

    let manifest_src =
        fs::read_to_string(repo.join("crates/deck/deck-conformance/app/app.manifest.jsonc")).unwrap();
    let manifest = AppManifest::parse(&manifest_src).unwrap();
    let raw =
        fs::read_to_string(repo.join("crates/deck/deck-conformance/app/views/board/page.sui")).unwrap();
    // SFC: strip the `<controller>` block before parsing and materialize its
    // body here (deck serves it as a virtual sibling; this harness has no
    // bundler). The name must not collide with the SYNTHETIC controller below.
    let (board_src, inline) = sail_compiler::parse::preprocess::extract_controller_block(&raw);
    let program = parse(&board_src).unwrap();

    let work = repo.join("target/synthetic-jsdom-proof");
    let _ = fs::remove_dir_all(&work);
    fs::create_dir_all(&work).unwrap();
    let authored_path = work.join("board.authored.ts");
    fs::write(&authored_path, inline.expect("board declares a <controller> block")).unwrap();
    let authored = AuthoredController {
        specifier: authored_path.to_string_lossy().into_owned(),
    };

    let out = emit_synthetic(
        &program,
        &manifest,
        Some(&authored),
        "./board.synthetic.controller.ts",
        CompileMode::Component,
        None,
    )
    .unwrap();

    fs::write(work.join("board.synthetic.controller.ts"), &out.controller_ts).unwrap();
    fs::write(work.join("board.render.ts"), &out.render_ts).unwrap();
    write_harness(&repo, &work);

    let status = Command::new(&vitest)
        .args(["run", "-c", "vitest.config.ts"])
        .current_dir(&work)
        .status()
        .expect("run vitest");
    assert!(status.success(), "the synthesized board render must client-mount + stay reactive in jsdom");
}

fn write_harness(repo: &Path, work: &Path) {
    let r = repo.display();
    // Slice 2 — use a shim that re-exports the SSR server surface + the
    // hydration-marker recording helpers (markHydrateText et al.) so the
    // synthetic render's `markHydrateText(...)` calls type-check and resolve
    // at runtime under vitest. The server.ts surface alone lacks the marker
    // helpers; the shim pulls them in from `hydrate-mark.ts`. This matches
    // `synthetic_client_jsdom`'s harness.
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
    let route_server = repo.join("packages/deck/router/src/server.ts");
    let cfg = format!(
        r#"import {{ defineConfig }} from 'vitest/config'
export default defineConfig({{
  test: {{ environment: 'jsdom', include: ['mount.test.ts'] }},
  resolve: {{ alias: {{
    '@ship/sail-runtime/internal': '{rt}',
    '@ship/sail-runtime': '{rt}',
    '@ship/deck/router': '{route}',
  }} }},
}})
"#,
        rt = shim.to_string_lossy(),
        route = route_server.to_string_lossy(),
    );
    fs::write(work.join("vitest.config.ts"), cfg).unwrap();
    let test = r#"import { describe, it, expect } from 'vitest'
import { mount } from './board.render.ts'

describe('synthesized board — client mount + reactivity', () => {
  it('builds the control-flow DOM and re-renders on a signal write', () => {
    const container = document.createElement('div')
    document.body.appendChild(container)
    const dispose = mount(container)

    // {#if items().length > 0} → true → the list renders both {#for} rows.
    // The canonical fixture authors <row>, which lowers to div.sail-row —
    // the old <ul>/<li> markup died with the corpus codemod.
    expect(container.querySelectorAll('.sail-row').length).toBe(2)
    expect(container.textContent).toContain('Alpha')
    expect(container.textContent).toContain('Bravo')
    // Nested component tag renders with its slotted label.
    expect(container.querySelector('demo-chip')).toBeTruthy()
    // aria-pressed={item.flagged} → exactly one pressed row (false removes
    // the attribute; true sets it).
    expect(container.querySelectorAll('.sail-row[aria-pressed]').length).toBe(1)

    // on:click={() => add()} mutates the controller signal → the {#for}
    // re-renders with a third row. Proves live reactivity, not just paint.
    const buttons = [...container.querySelectorAll('button')] as HTMLButtonElement[]
    const addBtn = buttons.find((b) => b.textContent === 'add')
    expect(addBtn).toBeTruthy()
    addBtn!.click()
    expect(container.querySelectorAll('.sail-row').length).toBe(3)

    // on:click={() => remove(item.id)} removes a row.
    const removeBtn = [...container.querySelectorAll('button')].find(
      (b) => b.textContent === 'remove',
    ) as HTMLButtonElement
    removeBtn.click()
    expect(container.querySelectorAll('.sail-row').length).toBe(2)

    dispose()
  })
})
"#;
    fs::write(work.join("mount.test.ts"), test).unwrap();
}
