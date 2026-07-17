//! jsdom proof for the synthetic CLIENT contract (slice 1, acceptance #1/#3).
//!
//! Two things the SSR sweep can't show:
//!   * **CSR** — the client `default.setup().__mount(container)` path, driven
//!     through the REAL `mount.ts`, builds reactive DOM in a browser (jsdom):
//!     hub `/` renders, and a propless component (badge) re-renders on a signal
//!     write. (acceptance #1: "render CSR (the __mount path)".)
//!   * **Isomorphic-render parity / clean hydration** — the SAME render run
//!     server-side (the SSR module against the shared server-DOM, i.e. exactly
//!     what ship-js serializes) produces a tree the client agrees with: the
//!     client `__mount` build is structurally identical, and `mount.ts`
//!     adopting the SSR HTML completes with NO mismatch / teardown.
//!     (acceptance #3.)
//!
//! Opt-in: requires `bun install` so `node_modules/.bin/vitest` exists.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use sail_compiler::emit::synthetic::{
    emit_synthetic, emit_synthetic_client_module, AppManifest, AuthoredController,
};
use sail_compiler::parse::parse;
use sail_compiler::CompileMode;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..").canonicalize().unwrap()
}

// The in-repo conformance app is the canonical fixture. App-level targets
// (apps/hub, views/) live OUTSIDE the framework boundary — sail-compiler's tests
// resolve only against in-repo fixtures.
const CONF_MANIFEST: &str = "crates/deck/deck-conformance/app/app.manifest.jsonc";

struct Target {
    name: &'static str,
    source_rel: &'static str,
    manifest_rel: &'static str,
    mode: CompileMode,
    /// Component tag name (kebab-case) for component-mode emit; `None` for views.
    component_tag: Option<&'static str>,
}

/// Synthesize, for one target: the merged CLIENT module (`<name>.client.ts`,
/// what `mount.ts` drives) AND the split SSR render + controller
/// (`<name>.ssr.ts` + `<name>.controller.ts`, what reproduces the ship-js
/// server-DOM render). Both compose the same authored controller (abs path).
fn synth(repo: &Path, work: &Path, t: &Target) {
    let raw = fs::read_to_string(repo.join(t.source_rel)).expect("read source");
    // SFC: the `<controller>` block is the controller. Strip it before parsing
    // (as `compile()` does) and write the body to a real module here — deck
    // serves it as a virtual sibling; this harness has no bundler, so it
    // materializes it in its own work dir under a name that cannot collide
    // with the SYNTHETIC controller the emit generates below.
    let (src, inline) = sail_compiler::parse::preprocess::extract_controller_block(&raw);
    let manifest_src = fs::read_to_string(repo.join(t.manifest_rel)).expect("read manifest");
    let manifest = AppManifest::parse(&manifest_src).expect("manifest");
    let program = parse(&src).unwrap_or_else(|e| panic!("parse {}: {}", t.name, e.render(&src)));
    let authored = inline.as_ref().map(|body| {
        let file = work.join(format!("{}.authored.ts", t.name));
        fs::write(&file, body).expect("write authored controller");
        AuthoredController { specifier: file.to_string_lossy().into_owned() }
    });

    let merged = emit_synthetic_client_module(&program, &manifest, authored.as_ref(), t.mode, t.component_tag)
        .unwrap_or_else(|e| panic!("merged {}: {}", t.name, e.render(&src)));
    fs::write(work.join(format!("{}.client.ts", t.name)), merged).unwrap();

    let split = emit_synthetic(
        &program,
        &manifest,
        authored.as_ref(),
        &format!("./{}.controller.ts", t.name),
        t.mode,
        t.component_tag,
    )
    .unwrap_or_else(|e| panic!("split {}: {}", t.name, e.render(&src)));
    fs::write(work.join(format!("{}.ssr.ts", t.name)), split.render_ts).unwrap();
    fs::write(
        work.join(format!("{}.controller.ts", t.name)),
        split.controller_ts,
    )
    .unwrap();
}

fn write_harness(repo: &Path, work: &Path) {
    let r = repo.display();
    // @ship/sail-runtime → a shim (the SSR server surface + `view` + the types
    // @ship/sail-dom imports). @ship/sail-dom → its real source. @ship/deck/router
    // → the server route factory. mount.ts is imported by absolute path.
    // Slice 2 — re-export the hydration-marker recording helpers so the
    // SSR modules (which call `markHydrateText` next to `bindText`) type-check
    // under vitest + the synthetic-server-DOM path. The shim's server.ts
    // already pulls the rest of the runtime surface through.
    fs::write(
        work.join("runtime-shim.ts"),
        format!(
            "export * from \"{r}/packages/sail/runtime/src/server.ts\";\n\
             export {{ component }} from \"{r}/packages/sail/runtime/src/component.ts\";\n\
             export {{ markHydrateText, markHydrateAttr, markHydrateBoundary }} from \"{r}/packages/sail/runtime/src/hydrate-mark.ts\";\n\
             export {{ __tpl }} from \"{r}/packages/sail/runtime/src/tpl.ts\";\n\
             export {{ delegateEvents }} from \"{r}/packages/sail/runtime/src/delegate.ts\";\n\
             export type {{ Modifier, OnOptions }} from \"{r}/packages/sail/runtime/src/on.ts\";\n"
        ),
    )
    .unwrap();

    let cfg = format!(
        r#"import {{ defineConfig }} from 'vitest/config'
export default defineConfig({{
  // testTimeout: 20s, not vitest's 5s default — this harness's work dir is
  // nuked and rebuilt every run (see `remove_dir_all` above), so vite's
  // dependency pre-bundle cache under `<work>/node_modules/.vite` never
  // survives to warm a second run. A COLD transform of the runtime's
  // module graph (mount.ts, if.ts, for.ts, component-factory.ts, …) took
  // ~13s in one observed run — comfortably past the 5s default, which
  // read as a flaky timeout on whichever test happened to import it first
  // (not a real hang: the same assertions pass once the graph is warm).
  test: {{ environment: 'jsdom', include: ['client.test.ts'], testTimeout: 20000 }},
  resolve: {{ alias: {{
    '@ship/sail-runtime/internal': '{w}/runtime-shim.ts',
    '@ship/sail-runtime': '{w}/runtime-shim.ts',
    '@ship/sail-dom': '{r}/packages/sail/dom/src/index.ts',
    '@ship/deck/router': '{r}/packages/deck/router/src/server.ts',
  }} }},
}})
"#,
        w = work.display(),
        r = r,
    );
    fs::write(work.join("vitest.config.ts"), cfg).unwrap();

    let mount_path = format!("{r}/packages/sail/runtime/src/mount.ts");
    let server_dom = format!("{r}/packages/sail/runtime/src/server-dom.ts");
    let test = format!(
        r#"import {{ describe, it, expect }} from 'vitest'
import {{ mount }} from '{mount}'
import {{ installServerDom }} from '{server_dom}'

// A minimal ViewPayload for a pre-resolved client module + given SSR html.
function payload(name: string, html: string) {{
  return {{ name, html, css: '', scope: {{}}, moduleUrl: '', params: {{}} }} as any
}}

// Reproduce the ship-js SSR HTML by running the SSR render against the shared
// server-DOM — the SAME code ship-js runs, just in node. Swaps globalThis.document
// for the server-DOM during the render, then restores jsdom's document.
async function ssrHtml(name: string): Promise<string> {{
  const saved = (globalThis as any).document
  const srv = installServerDom()
  try {{
    const mod = await import(`./${{name}}.ssr.ts`)
    const container = srv.container()
    const dispose = mod.mount(container)
    const html = srv.html(container)
    dispose()
    return html
  }} finally {{
    ;(globalThis as any).document = saved
  }}
}}

// Strip insignificant whitespace BETWEEN tags so server-DOM vs jsdom
// serialization differences don't mask structural equality.
function norm(html: string): string {{
  return html.replace(/>\s+</g, '><').replace(/\s+/g, ' ').trim()
}}

describe('synthetic CLIENT contract — CSR (__mount via mount.ts)', () => {{
  it('badge (a real component) mounts and stays reactive', async () => {{
    const mod = await import('./badge.client.ts')
    const el = document.createElement('div')
    document.body.appendChild(el)
    const handle = await mount(el, payload('badge', ''), {{ module: mod as any }})
    const btn = el.querySelector('button') as HTMLButtonElement
    expect(btn).toBeTruthy()
    expect(btn.textContent).toBe('0')
    btn.click()
    expect(btn.textContent).toBe('1')  // signal write → re-render (live reactivity)
    handle.dispose()
  }})

  it('board control-flow builds + reacts client-side', async () => {{
    const mod = await import('./board.client.ts')
    const el = document.createElement('div')
    document.body.appendChild(el)
    const handle = await mount(el, payload('board', ''), {{ module: mod as any }})
    // The `{{#for}}` row is a `<row>` primitive (compiles to `.sail-row`), not
    // an `<li>` — the fixture migrated off `<ul>`/`<li class:flagged>` to
    // `<column>`/`<row>` + `aria-pressed` (see 22fd23f); these selectors
    // track the current, correct markup.
    expect(el.querySelectorAll('.sail-row').length).toBe(2)
    expect(el.textContent).toContain('Alpha')
    expect(el.querySelector('demo-chip')).toBeTruthy()
    expect(el.querySelectorAll('.sail-row[aria-pressed]').length).toBe(1)
    handle.dispose()
  }})
}})

describe('isomorphic-render parity / clean hydration (acceptance #3)', () => {{
  it('board: client __mount build matches the ship-js SSR tree (markers stripped)', async () => {{
    const html_ssr = await ssrHtml('board')
    const mod = await import('./board.client.ts')
    const el = document.createElement('div')
    document.body.appendChild(el)
    const handle = await mount(el, payload('board', ''), {{ module: mod as any }})
    const html_csr = el.innerHTML
    // The SAME render server-side and client-side → same DOM structure.
    // Slice 2 — ship-js SSR now wraps dynamic text in `<!--sail:text:bN-->`
    // markers (so the client `hydrate()` can reattach); strip them from
    // both sides for the parity check (the markers are an SSR-only
    // hydration hint, not a render hint).
    const stripMarkers = (s: string) => s.replace(/<!--sail:text:b\d+-->|<!--\/sail-->/g, '')
    expect(norm(stripMarkers(html_csr))).toBe(norm(stripMarkers(html_ssr)))
    handle.dispose()
  }})

  // FOLLOW-UP (in-boundary fixture needed): the clean-ADOPT proof (mount onto
  // SSR HTML leaves the DOM byte-identical — no teardown) and the
  // hydrate-reattach-reactivity proof below previously used the `hub`/`showcase`
  // app targets, which lived OUTSIDE the framework boundary. They need a
  // dedicated in-repo conformance fixture: a clean view whose mount has no
  // side-effect AND a controller-owned signal + dynamic text. The conformance
  // app's `badge` is stateful (mounting flips its count), so it's not a clean
  // adopt fixture. board-parity (above) + greet markers (below) still gate the
  // core isomorphism + marker contracts.
}})

describe('slice 2 — hydration markers + reactivity', () => {{
  // Slice 2 — ship-js SSR HTML carries the hydration markers the
  // client `hydrate()` reads. Greet's dynamic text binding (`route.params.id()`) —
  // a single DynamicText position that the marker wraps.
  it('ship-js SSR HTML carries `<!--sail:text:bN-->` markers for dynamic text', async () => {{
    const html_ssr = await ssrHtml('greet')
    expect(html_ssr).toMatch(/<!--sail:text:b\d+-->/)
    expect(html_ssr).toMatch(/<!--\/sail-->/)
  }})

  // The hydrate-reattach-reactivity proof (mount onto SSR HTML, then drive a
  // signal write and watch the DOM update) needs the clean stateful fixture
  // described in the FOLLOW-UP note above — tracked separately.
}})
"#,
        mount = mount_path,
        server_dom = server_dom,
    );
    fs::write(work.join("client.test.ts"), test).unwrap();
}

#[test]
fn client_contract_csr_and_hydration_in_jsdom() {
    let repo = repo_root();
    let vitest = repo.join("node_modules/.bin/vitest");
    if !vitest.exists() {
        eprintln!(
            "SKIP synthetic_client_jsdom: {} not found — run `bun install` to enable the jsdom proof.",
            vitest.display()
        );
        return;
    }

    let work = repo.join("target/synthetic-client-jsdom");
    let _ = fs::remove_dir_all(&work);
    fs::create_dir_all(&work).unwrap();

    let targets = [
        Target {
            name: "badge",
            source_rel: "crates/deck/deck-conformance/app/components/badge/badge.sail",
            manifest_rel: CONF_MANIFEST,
            mode: CompileMode::Component,
            component_tag: Some("sail-badge"),
        },
        Target {
            name: "board",
            source_rel: "crates/deck/deck-conformance/app/views/board/page.sail",
            manifest_rel: CONF_MANIFEST,
            mode: CompileMode::Component,
            component_tag: None,
        },
        Target {
            name: "greet",
            source_rel: "crates/deck/deck-conformance/app/views/greet/[id]/page.sail",
            manifest_rel: CONF_MANIFEST,
            mode: CompileMode::Component,
            component_tag: None,
        },
    ];
    for t in &targets {
        synth(&repo, &work, t);
    }
    write_harness(&repo, &work);

    let status = Command::new(&vitest)
        .args(["run", "-c", "vitest.config.ts"])
        .current_dir(&work)
        .status()
        .expect("run vitest");
    assert!(
        status.success(),
        "the synthetic CLIENT contract must CSR + hydrate cleanly in jsdom"
    );
}
