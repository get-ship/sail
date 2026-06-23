//! Slice 5 — dynamic / param sub-route.
//!
//! Proves the resolver → params → render → hydrate chain for a `[id]` segment:
//!   1. `GET /_deck/views/greet/zelda` resolves `views/greet/[id]/page.sui`,
//!      captures `params["id"] = "zelda"`, and returns 200 JSON.
//!   2. The JSON payload's `params` map contains `"id" → "zelda"`.
//!   3. The SSR `html` field contains the static scaffold of the template
//!      ("Hello") — the dynamic slot renders server-side with the resolved
//!      param.
//!
//! Pure deck-core via the axum router — no server, no browser.
//!
//! Run: `cargo test -p deck-conformance --test param`

use axum::body::Body;
use axum::http::{Request, StatusCode};
use deck_conformance::{deck_app, default_app_dir};
use tower::ServiceExt;

async fn get(path: &str) -> (StatusCode, String) {
    let resp = deck_app(default_app_dir())
        .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
        .await
        .expect("router response");
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .expect("collect body");
    (status, String::from_utf8_lossy(&bytes).into_owned())
}

/// The `[id]` resolver captures the dynamic segment: `greet/zelda` must
/// resolve to `views/greet/[id]/page.sui` and capture `params["id"] == "zelda"`.
///
/// Calls the payload endpoint (`/_deck/views/{name}/{sub}`) — this is the pure
/// deck-core path: resolver → params → compile → SSR.
#[tokio::test]
async fn param_id_captured_in_payload() {
    let (status, body) = get("/_deck/views/greet/zelda").await;
    assert_eq!(status, StatusCode::OK, "greet/zelda must resolve; got head:\n{}", &body[..body.len().min(400)]);

    let v: serde_json::Value = serde_json::from_str(&body)
        .unwrap_or_else(|e| panic!("payload must be JSON: {e}\nbody: {body}"));

    // Route param captured by the [id] segment resolver.
    assert_eq!(
        v["params"]["id"].as_str(),
        Some("zelda"),
        "params.id must be \"zelda\"; got payload: {v}"
    );

    // SSR html must contain the static "Hello" text from the template.
    let html = v["html"].as_str().unwrap_or("");
    assert!(
        html.contains("Hello"),
        "SSR html must contain 'Hello' from the template; got: {html}"
    );
}

/// The greet index view serves the listing page (no params).
#[tokio::test]
async fn greet_index_has_no_params() {
    let (status, body) = get("/_deck/views/greet").await;
    assert_eq!(status, StatusCode::OK);

    let v: serde_json::Value = serde_json::from_str(&body)
        .unwrap_or_else(|e| panic!("payload must be JSON: {e}"));

    let params = v["params"].as_object().expect("params must be an object");
    assert!(params.is_empty(), "greet index must have no route params; got: {params:?}");

    let html = v["html"].as_str().unwrap_or("");
    assert!(html.contains("Greet"), "greet index SSR must contain heading; got: {html}");
}

/// A missing sub-route (not matching any static or [param] segment) is a 404.
#[tokio::test]
async fn missing_view_is_404() {
    let (status, _body) = get("/_deck/views/does-not-exist").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
