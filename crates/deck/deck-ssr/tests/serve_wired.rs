//! Acceptance #1: the ship-js SSR path is WIRED into deck-core's SSR seam and
//! is the engine — `deck_core::synthetic_ssr::first_paint` returns ship-js
//! first paint (renderer registered). The Rust render-eval engine was removed in
//! the JS-only SSR cutover, so ship-js is the single path; there is no
//! `DECK_SSR` opt-back.
//!
//! This drives the SAME seam the view serving calls — the registered `deck-ssr`
//! renderer (pooled isolate, synthesize→bundle→run) behind
//! `deck_core::synthetic_ssr::first_paint` — proving the seam end-to-end, not
//! just the capability in isolation. (The HTTP/JSON-payload wrapper around this
//! seam moved out of deck-core to the `ship-views` crate, where the payload
//! SHAPE is covered; deck-ssr can't depend on ship-views — deck ⊥ ship-views —
//! and links V8, so it tests the engine at the deck-core seam directly. The
//! resolve→SsrRequest→first_paint→inject_form_actions chain below mirrors
//! exactly what `ship_views::view_payload::serve_view_payload` does internally.)

use std::sync::Arc;

use deck_core::DataPort;
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

/// Build the ship-js renderer and return it with the mount-scoped project root
/// (the conformance app dir) — both threaded per-request into `resolve` /
/// `first_paint`. No process global: the engine is an owned `Arc` handle.
fn setup() -> (
    std::path::PathBuf,
    Arc<dyn deck_core::synthetic_ssr::SyntheticSsr>,
) {
    let repo = repo_root().expect("repo root");
    let engine = deck_ssr::serve::engine(Some(Arc::new(DemoData))).expect("build ship-js renderer");
    (repo.join("crates/deck/deck-conformance/app"), engine)
}

/// First-paint HTML through the deck-core SSR seam for a (sub-folded) view.
/// Mirrors `serve_view_payload`'s internal SSR call: resolve → build the
/// `SsrRequest` from the resolved source path + captured params → run the
/// registered ship-js renderer → host-inject `<form mutation>` actions.
fn payload_html(
    root: &std::path::Path,
    engine: &dyn deck_core::synthetic_ssr::SyntheticSsr,
    name: &str,
    sub: Option<&str>,
) -> String {
    let resolved = deck_core::resolver::resolve(root, name, sub).expect("resolve view");
    let source_path = resolved.source_path.as_deref().expect("resolved view has a source path");
    let params = serde_json::to_value(&resolved.params).unwrap_or(Value::Null);
    let view_name = match sub {
        Some(s) => format!("{name}/{s}"),
        None => name.to_string(),
    };
    let pathname = format!("/{view_name}");
    let req = deck_core::synthetic_ssr::SsrRequest {
        view_name: &view_name,
        sub_path: sub,
        source_path,
        pathname: &pathname,
        params,
        query: serde_json::json!({}),
    };
    let raw = deck_core::synthetic_ssr::first_paint(engine, &req).expect("ship-js produced first paint");
    deck_core::inject_form_actions(&raw, &view_name)
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
    let (root, engine) = setup();
    let engine = &*engine;

    // ── board (a REAL controller({...}) DSL view) ──────────────────────────
    // ship-js runs the controller, so first paint carries the controller-signal
    // rows: the `{#if}` branch renders and the per-row `aria-pressed:`
    // directive fires (post-floor migration; the prior `class:flagged`
    // global hook is banned by `sui-no-class`).
    let on_raw = payload_html(&root, engine, "board", None);
    let on = strip_markers(&on_raw);
    assert!(on.contains("Alpha") && on.contains("Bravo"), "ship-js {{#for}} renders every row; got:\n{on}");
    // Match the literal attribute form (space-prefixed, on the element
    // opening tag) — the SX stylesheet also references `[aria-pressed='true']`
    // for the pressed: state-prefix utility, so count only the attribute.
    assert!(
        on.matches(" aria-pressed=\"").count() == 1,
        "per-row aria-pressed: directive (one flagged row); got:\n{on}"
    );

    // ── feed (<data live> + host-dispatch) ─────────────────────────────────
    // The live op resolves over the registered DataPort, so first paint reflects
    // the resolved state, not `loading`.
    let feed_on_raw = payload_html(&root, engine, "feed", None);
    let feed_on = strip_markers(&feed_on_raw);
    assert!(
        feed_on.contains("state: ok"),
        "ship-js resolves the live <data> over host-dispatch; got:\n{feed_on}"
    );

    // ── greet/[id] (route param via host-dispatch) ─────────────────────────
    // The router captures `id` from the sub-path; ship-js resolves
    // `route.params.id()` over host-dispatch into first paint.
    let greet_on_raw = payload_html(&root, engine, "greet", Some("zelda"));
    let greet_on = strip_markers(&greet_on_raw);
    assert!(
        greet_on.contains("Hello, zelda"),
        "ship-js resolves the route param into first paint; got:\n{greet_on}"
    );
}
