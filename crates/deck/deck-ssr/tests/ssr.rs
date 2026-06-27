//! SSR acceptance: a SYNTHESIZED view renders correct first-paint HTML through
//! ship-js (deno_core/V8), with the route param resolved over host-dispatch.
//! This is the production TS-SSR path (the Rust render-eval engine was removed).

use std::sync::Arc;

use deck_ssr::build::{repo_root, synthesize_and_bundle, synthesize_and_bundle_with, ClientAlias};
use deck_ssr::host::SsrDispatch;
use deck_ssr::run::render;
use deck_core::DataPort;
use serde_json::{json, Value};
use sui_compiler::CompileMode;

const GREET: &str = "crates/deck/deck-conformance/app/views/greet/[id]/page.sui";
const GLYPH: &str = "crates/deck/deck-conformance/app/views/glyph/page.sui";
const BOARD: &str = "crates/deck/deck-conformance/app/views/board/page.sui";
const BOARD_CTRL: &str = "crates/deck/deck-conformance/app/views/board/controller.ts";
const FEED: &str = "crates/deck/deck-conformance/app/views/feed/page.sui";
const STYLED: &str = "crates/deck/deck-conformance/app/views/styled.view/page.sui";
const DEMO_SERVER: &str = "crates/deck/deck-conformance/app/clients/demo.server.ts";
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
        !bundle.contains("@ship/sui-runtime") && !bundle.contains("@ship/deck/router"),
        "framework specifiers must be inlined to their server entries; only `ship` stays external"
    );

    // Render in the isolate with the route the deck router matched for the request.
    let dispatch = Arc::new(SsrDispatch::for_request("/greet/zelda", json!({ "id": "zelda" }), json!({})));
    let out = render(&bundle, dispatch).expect("render in ship-js");

    // First paint produced by ship-js run_module.
    let html = strip_markers(&out.html);
    assert!(
        html.contains("<h1>Hello, zelda</h1>"),
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
    // The control-flow-heavy surface: {#if}/{#for}/class:/nested <demo-chip>,
    // backed by a typed authored controller. Proves the full TemplateEmitter
    // (bindIf/bindFor) runs server-side on the server-DOM and serializes.
    let repo = repo_root().expect("repo root");
    let build_dir = repo.join("target/deck-ssr-test/board");
    let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
    let bundle = rt
        .block_on(synthesize_and_bundle_with(
            &repo,
            BOARD,
            MANIFEST,
            CompileMode::Component,
            Some(&repo.join(BOARD_CTRL)),
            &[],
            &build_dir,
        ))
        .expect("synthesize + bundle board");

    let dispatch = Arc::new(SsrDispatch::for_request("/board", json!({}), json!({})));
    let out = render(&bundle, dispatch).expect("render board");

    // {#if items().length > 0} → true → the list renders both rows.
    assert!(out.html.contains("<ul>"), "if-branch (the list) must render; got:\n{}", out.html);
    assert!(
        out.html.contains("Alpha") && out.html.contains("Bravo"),
        "{{#for}} must render every row; got:\n{}",
        out.html
    );
    // Nested component renders as its custom-element tag with slotted content.
    assert!(out.html.contains("<demo-chip>"), "nested component tag must render; got:\n{}", out.html);
    // class:flagged={item.flagged} → only Bravo (flagged) carries the class.
    assert!(
        out.html.matches("class=\"flagged\"").count() == 1,
        "class: directive must reflect per-row state (one flagged row); got:\n{}",
        out.html
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
    // The <data live source="api.demo.clock()"> + <client> surface. The client
    // server entry resolves the source over the host-dispatch wire; after the
    // async flush, first paint carries the resolved state (not `loading`).
    let repo = repo_root().expect("repo root");
    let build_dir = repo.join("target/deck-ssr-test/feed");
    let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
    let client = ClientAlias {
        specifier: "@app/clients/demo".to_string(),
        server_entry: repo.join(DEMO_SERVER),
    };
    let bundle = rt
        .block_on(synthesize_and_bundle(&repo, FEED, MANIFEST, CompileMode::Component, &[client], &build_dir))
        .expect("synthesize + bundle feed");

    let dispatch = Arc::new(
        SsrDispatch::for_request("/feed", json!({}), json!({})).with_data(Arc::new(DemoData)),
    );
    let out = render(&bundle, dispatch).expect("render feed");

    // The live data op crossed the host-dispatch wire.
    assert!(
        out.call_path.iter().any(|op| op == "api.demo.clock"),
        "the <data> source op must cross the wire; call path: {:?}",
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

#[test]
fn styled_view_lifted_style_reaches_ssr_first_paint() {
    // The bug: a view's `<style>` was parsed and lifted into `Program.styles`,
    // but the synthetic render never emitted it, so SSR HTML came out
    // unstyled. The fix routes `Program.styles` through both `emit_render`
    // (the SSR contract that ship-js runs) and `emit_render_client` (the
    // CSR contract `mount.ts` drives) — both now inline an idempotent
    // `<style data-sui-style="<fnv1a>">…</style>` install block at the top
    // of `container`. The server DOM serializes that element into the
    // first-paint HTML verbatim, and the browser treats any in-document
    // `<style>` as an active stylesheet — so the same lifted CSS reaches
    // both the SSR and CSR paths from a single source.
    let repo = repo_root().expect("repo root");
    let build_dir = repo.join("target/deck-ssr-test/styled");
    let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
    let bundle = rt
        .block_on(synthesize_and_bundle(&repo, STYLED, MANIFEST, CompileMode::Component, &[], &build_dir))
        .expect("synthesize + bundle styled");

    let dispatch = Arc::new(SsrDispatch::for_request("/styled.view", json!({}), json!({})));
    let out = render(&bundle, dispatch).expect("render styled");


    // The view's own CSS reaches the serialized HTML. The unique selector
    // + comment string are intentional: they live in the conformance view
    // (`crates/deck/deck-conformance/app/views/styled.view/page.sui`) and
    // double as the round-trip probe — if the lifted CSS were ever dropped
    // on the floor again, this assertion fails.
    assert!(
        out.html.contains(".ssr-view-css-marker"),
        "lifted CSS selector must reach SSR first-paint; got:\n{}",
        out.html
    );
    assert!(
        out.html.contains("ssr-view-css-probe"),
        "lifted CSS marker comment must reach SSR first-paint; got:\n{}",
        out.html
    );
    // The CSS lives in a real <style> element (the install block creates
    // one), not escaped text inside another element — that's how the
    // browser picks it up.
    assert!(
        out.html.contains("<style") && out.html.contains("</style>"),
        "lifted CSS must be wrapped in a <style> element; got:\n{}",
        out.html
    );
    // Idempotence / dedupe key — the FNV-1a hash on the content. Stable
    // across re-renders, lets a re-mount find and skip the existing
    // <style> instead of duplicating it.
    assert!(
        out.html.contains("data-sui-style="),
        "dedupe data attribute must be present on the injected <style>; got:\n{}",
        out.html
    );
    // The view's visible markup still serializes too (sanity — the styles
    // install must not displace the body content).
    assert!(
        out.html.contains("ssr-view-css-title") && out.html.contains("styled"),
        "view markup must still render alongside the styles; got:\n{}",
        out.html
    );
}
