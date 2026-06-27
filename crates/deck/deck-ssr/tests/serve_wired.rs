//! Acceptance #1: the ship-js SSR path is WIRED into deck-core's serve path and
//! is the engine — `serve_view_payload` returns ship-js first paint (renderer
//! registered). The Rust render-eval engine was removed in the JS-only SSR
//! cutover, so ship-js is the single path; there is no `DECK_SSR` opt-back.
//!
//! This drives the SAME public entrypoint the daemon's HTTP handler calls
//! (`deck_core::serve_view_payload[_subpath]`), through the registered
//! `deck-ssr` renderer (pooled isolate, synthesize→bundle→run) — proving the
//! seam end-to-end, not just the capability in isolation.

use std::sync::Arc;

use deck_core::{DaemonMode, DataPort};
use deck_ssr::build::repo_root;
use serde_json::Value;

/// Host data surface for the `<data live source="api.demo.clock()">` view.
struct DemoData;
impl DataPort for DemoData {
    fn resolve(&self, domain: &str, op: &str, _args: &Value) -> Option<Value> {
        match (domain, op) {
            ("api", "demo.clock") => Some(serde_json::json!({ "now": "12:34:56" })),
            _ => None,
        }
    }
}

/// Pin the resolver at the conformance app + register the ship-js renderer.
/// Idempotent (OnceLock + a process-global root), safe to call once per binary.
fn setup() {
    let repo = repo_root().expect("repo root");
    deck_core::resolver::set_project_root(repo.join("crates/deck/deck-conformance/app"));
    deck_ssr::serve::register(Some(Arc::new(DemoData))).expect("register ship-js renderer");
}

fn payload_html(resp: &deck_core::DeckResponse) -> String {
    assert_eq!(resp.status, 200, "serve must be 200; body: {}", String::from_utf8_lossy(&resp.body));
    let v: Value = serde_json::from_slice(&resp.body).expect("payload is JSON");
    v["html"].as_str().expect("payload carries html").to_string()
}

/// Slice 2 — strip the ship-js SSR hydration markers
/// (`<!--sui:text:bN-->...<!--/sui-->`) so the assertions below check the
/// rendered text content, not the SSR-only hydration hint. A char-by-char
/// scan handles the two marker shapes the synthetic emit injects; avoids
/// pulling the `regex` crate in for what is fundamentally fixed-string
/// stripping.
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
fn wired_serve_renders_ship_js_first_paint() {
    setup();

    // ── board (a REAL controller({...}) DSL view) ──────────────────────────
    // ship-js runs the controller, so first paint carries the controller-signal
    // rows: the `{#if}` branch renders and the per-row `class:` directive fires.
    let on_raw = payload_html(&deck_core::serve_view_payload(DaemonMode::Development, "board"));
    let on = strip_markers(&on_raw);
    assert!(on.contains("<ul>"), "ship-js {{#if}} branch renders; got:\n{on}");
    assert!(
        on.matches("class=\"flagged\"").count() == 1,
        "per-row class: directive; got:\n{on}"
    );

    // ── feed (<data live> + host-dispatch) ─────────────────────────────────
    // The live op resolves over the registered DataPort, so first paint reflects
    // the resolved state, not `loading`.
    let feed_on_raw = payload_html(&deck_core::serve_view_payload(DaemonMode::Development, "feed"));
    let feed_on = strip_markers(&feed_on_raw);
    assert!(
        feed_on.contains("state: ok"),
        "ship-js resolves the live <data> over host-dispatch; got:\n{feed_on}"
    );

    // ── greet/[id] (route param via host-dispatch) ─────────────────────────
    // The router captures `id` from the sub-path; ship-js resolves
    // `route.params.id()` over host-dispatch into first paint.
    let greet_on_raw = payload_html(
        &deck_core::serve_view_payload_subpath(DaemonMode::Development, "greet", "zelda"),
    );
    let greet_on = strip_markers(&greet_on_raw);
    assert!(
        greet_on.contains("Hello, zelda"),
        "ship-js resolves the route param into first paint; got:\n{greet_on}"
    );
}
