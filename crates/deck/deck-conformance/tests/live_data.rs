//! Slice 7 — live-data SSE transport (no browser, ship-free).
//!
//! `data_port.rs`'s `serve_sse_inner` used to parse a route's live-query
//! directives to discover what to stream. That compiler-level authoring
//! surface is retired; the replacement discovery mechanism (how a
//! controller's own `live(() => …)` sources get pushed over this channel) is
//! a redesign owned elsewhere. Until it lands, every route's SSE channel is
//! keepalive-only — this file proves the TRANSPORT still works (right
//! content-type, 404 on an unknown route), not that data flows over it.
//!
//! Run: `cargo test -p deck-conformance --test live_data`

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use deck_conformance::{deck_app, default_app_dir};
use tower::ServiceExt as _;

/// `/_deck/views/clock/sse` must respond with `Content-Type: text/event-stream`.
#[tokio::test(flavor = "multi_thread")]
async fn clock_sse_content_type_is_event_stream() {
    let app = deck_app(default_app_dir());
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/_deck/views/clock/sse")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("response");
    assert_eq!(resp.status(), StatusCode::OK);
    let ct = resp
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(
        ct.contains("text/event-stream"),
        "Content-Type must be text/event-stream; got: {ct}"
    );
}

/// Unknown route → 404.
#[tokio::test(flavor = "multi_thread")]
async fn sse_unknown_view_is_404() {
    let app = deck_app(default_app_dir());
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/_deck/views/no-such-view/sse")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("response");
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}
