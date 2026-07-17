//! SSR acceptance: a SYNTHESIZED view renders correct first-paint HTML through
//! ship-js (deno_core/V8), with the route param resolved over host-dispatch.
//! This is the production TS-SSR path (the Rust render-eval engine was removed).

use std::sync::Arc;

use deck_ssr::build::{repo_root, synthesize_and_bundle};
use deck_ssr::host::SsrDispatch;
use deck_ssr::run::render;
use deck_core::DataPort;
use serde_json::{json, Value};
use sail_compiler::CompileMode;

const GREET: &str = "crates/deck/deck-conformance/app/views/greet/[id]/page.sui";
const GLYPH: &str = "crates/deck/deck-conformance/app/views/glyph/page.sui";
const BOARD: &str = "crates/deck/deck-conformance/app/views/board/page.sui";
const FEED: &str = "crates/deck/deck-conformance/app/views/feed/page.sui";
const STYLED: &str = "crates/deck/deck-conformance/app/views/styled.view/page.sui";
const MANIFEST: &str = "crates/deck/deck-conformance/app/app.manifest.jsonc";

/// A test data port: answers the demo ops the host-dispatch wire forwards.
struct DemoData;
impl DataPort for DemoData {
    fn resolve(&self, domain: &str, op: &str, _args: &Value) -> Option<Value> {
        match (domain, op) {
            ("api", "demo.clock") => Some(json!({ "now": "12:34:56" })),
            _ => None,
        }
    }
}

/// Slice 2 — strip the ship-js SSR hydration text markers
/// (`<!--sui:text:bN-->...<!--/sui-->`) so the rendered-content assertions
/// check the visible payload, not the SSR-only hydration hint the markers
/// carry. Char-by-char scan avoids pulling `regex` in for what is fixed-
/// string stripping.
fn strip_markers(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if s[i..].starts_with("<!--/sui-->") {
            i += "<!--/sui-->".len();
            continue;
        }
        let prefix = "<!--sui:text:b";
        if s[i..].starts_with(prefix)
            && let Some(end_rel) = s[i + prefix.len()..].find("-->")
        {
            i += prefix.len() + end_rel + "-->".len();
            continue;
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}
#[test]
fn greet_first_paint_via_ship_js_with_route_through_host_dispatch() {
    let repo = repo_root().expect("repo root");
    let build_dir = repo.join("target/deck-ssr-test/greet");

    // Synthesize + Rolldown-bundle into one self-contained ES module.
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("tokio runtime for bundling");
    let bundle = rt
        .block_on(synthesize_and_bundle(&repo, GREET, MANIFEST, CompileMode::Component, &[], &build_dir))
        .expect("synthesize + bundle");
    assert!(bundle.len() > 1000, "bundle should be a real self-contained module");
    // `ship` is the only external import (the host wire); nothing else leaks.
    assert!(
        !bundle.contains("@ship/sail-runtime") && !bundle.contains("@ship/deck/router"),
        "framework specifiers must be inlined to their server entries; only `ship` stays external"
    );

    // Render in the isolate with the route the deck router matched for the request.
    let dispatch = Arc::new(SsrDispatch::for_request("/greet/zelda", json!({ "id": "zelda" }), json!({})));
    let out = render(&bundle, dispatch).expect("render in ship-js");

    // First paint produced by ship-js run_module. The post-floor fixture
    // carries `sx=` class utilities on the heading, so check the text
    // payload rather than a literal `<h1>…</h1>` substring.
    let html = strip_markers(&out.html);
    assert!(
        html.contains("Hello, zelda"),
        "first paint must contain the route-resolved heading; got:\n{}",
        html
    );
    // The value came through the host-dispatch wire.
    assert_eq!(out.route_params["id"], "zelda", "route param resolved via host-dispatch");
    assert!(
        out.call_path.iter().any(|op| op == "route.current"),
        "the route value must have crossed the wire (route.current); call path: {:?}",
        out.call_path
    );
}

#[test]
fn board_control_flow_first_paint_via_ship_js() {
    // The control-flow-heavy surface: {#if}/{#for}/aria-pressed:/nested <demo-chip>,
    // backed by a typed authored controller. Proves the full TemplateEmitter
    // (bindIf/bindFor) runs server-side on the server-DOM and serializes.
    //
    // Pre-floor, the per-row state assertion was `class="flagged"` — the
    // `class:` directive emitted a global CSS hook the floor banned. The
    // migration converted `class:flagged={item.flagged}` to the closed
    // `aria-pressed={item.flagged}` semantics; the assertion keys on the
    // SAME ARIA contract the framework's toggleable primitives use.
    let repo = repo_root().expect("repo root");
    let build_dir = repo.join("target/deck-ssr-test/board");
    let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
    let bundle = rt
        .block_on(synthesize_and_bundle(
            &repo,
            BOARD,
            MANIFEST,
            CompileMode::Component,
            &[],
            &build_dir,
        ))
        .expect("synthesize + bundle board");

    let dispatch = Arc::new(SsrDispatch::for_request("/board", json!({}), json!({})));
    let out = render(&bundle, dispatch).expect("render board");

    // {#if items().length > 0} → true → the list renders both rows. The
    // post-floor layout primitive is `<column gap="2">` — the suite confirms
    // the if-branch serializes regardless of which element the lowered column
    // becomes (the suite asserts the row payload, not the wrapper).
    assert!(
        out.html.contains("Alpha") && out.html.contains("Bravo"),
        "{{#for}} must render every row; got:\n{}",
        out.html
    );
    // Nested component renders as its custom-element tag with slotted content.
    assert!(out.html.contains("<demo-chip>"), "nested component tag must render; got:\n{}", out.html);
    // aria-pressed={item.flagged} — only Bravo (flagged) carries the attr.
    // `bindAttr` lowers a boolean `true` value to an empty-string attribute
    // (the framework's boolean-attr convention), so we assert PRESENCE on
    // Bravo and ABSENCE on Alpha rather than a literal value.
    let html_no_markers = strip_markers(&out.html);
    let alpha_row = find_row_for(&html_no_markers, "Alpha");
    assert!(
        !alpha_row.contains("aria-pressed"),
        "Alpha row (not flagged) must NOT carry aria-pressed; got:\n{alpha_row}"
    );
    let bravo_row = find_row_for(&html_no_markers, "Bravo");
    assert!(
        bravo_row.contains("aria-pressed"),
        "Bravo row (flagged) MUST carry aria-pressed; got:\n{bravo_row}"
    );
    // The remove buttons (on:click handlers) render too.
    assert!(out.html.contains("remove"), "on: handlers' elements must render; got:\n{}", out.html);
}

#[test]
fn glyph_svg_first_paint_serializes_namespaced() {
    // The broader server-DOM case (acceptance #3): an inline `<svg>` subtree.
    // The synthetic render builds it via `createElementNS(SVG_NS, …)`, and the
    // shared server-DOM serializes namespaced elements — empty SVG elements
    // self-close (`<circle .../>`), which HTML void elements never do.
    let repo = repo_root().expect("repo root");
    let build_dir = repo.join("target/deck-ssr-test/glyph");
    let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
    let bundle = rt
        .block_on(synthesize_and_bundle(&repo, GLYPH, MANIFEST, CompileMode::Component, &[], &build_dir))
        .expect("synthesize + bundle glyph");

    let dispatch = Arc::new(SsrDispatch::for_request("/glyph", json!({}), json!({})));
    let out = render(&bundle, dispatch).expect("render glyph");

    assert!(out.html.contains("<svg"), "svg root must render; got:\n{}", out.html);
    assert!(
        out.html.contains("<circle") && out.html.contains("r=\"10\""),
        "svg child + its attribute must render; got:\n{}",
        out.html
    );
    assert!(out.html.contains("<path"), "second svg child must render; got:\n{}", out.html);
    // Namespace-aware serialization: empty SVG elements self-close. (HTML void
    // elements like <br> never emit the slash — this is the SVG branch.)
    assert!(
        out.html.contains("/>"),
        "empty SVG elements must self-close (namespaced serialization); got:\n{}",
        out.html
    );
}

#[test]
fn feed_data_client_value_resolves_via_host_dispatch() {
    // A controller-side `live(() => call('api.demo.clock', {}))` source (the
    // real living-async surface — no `<data>`/`<client>` authoring elements
    // involved). `call` crosses the host-dispatch wire directly; the fake
    // `DataPort` (`DemoData`, above) answers it; after the async flush, first
    // paint carries the resolved state (not `loading`).
    let repo = repo_root().expect("repo root");
    let build_dir = repo.join("target/deck-ssr-test/feed");
    let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
    let bundle = rt
        .block_on(synthesize_and_bundle(&repo, FEED, MANIFEST, CompileMode::Component, &[], &build_dir))
        .expect("synthesize + bundle feed");

    let dispatch = Arc::new(
        SsrDispatch::for_request("/feed", json!({}), json!({})).with_data(Arc::new(DemoData)),
    );
    let out = render(&bundle, dispatch).expect("render feed");

    // The live data op crossed the host-dispatch wire.
    assert!(
        out.call_path.iter().any(|op| op == "api.demo.clock"),
        "the live() source op must cross the wire; call path: {:?}",
        out.call_path
    );
    // After the flush, the live signal is resolved (ok), not loading.
    let html = strip_markers(&out.html);
    assert!(
        html.contains("state: ok"),
        "first paint must reflect the resolved live state via host-dispatch; got:\n{}",
        html
    );
}

/// Slice the rendered HTML around a `<demo-chip>…</demo-chip>` carrying the
/// given label, returning the row's enclosing `<div …>` opening tag (a
/// per-row substring the {#for} iteration emits). The board test uses this
/// to assert aria-pressed presence per row without indexing into specific
/// column-tag lowering output.
fn find_row_for(html: &str, label: &str) -> String {
    let needle = format!("<demo-chip>{label}</demo-chip>");
    let chip_idx = html.find(&needle).unwrap_or_else(|| panic!("row {label} not in html"));
    let div_open = html[..chip_idx]
        .rfind("<div")
        .unwrap_or_else(|| panic!("no enclosing <div for {label}"));
    let div_close = html[div_open..].find('>').unwrap() + div_open;
    html[div_open..=div_close].to_string()
}
