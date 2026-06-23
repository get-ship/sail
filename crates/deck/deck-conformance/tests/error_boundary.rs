//! GAP 5 — Error boundary (ship-free, no browser).
//!
//! The `boom` view subscribes to `ship.demo.boom()`, which always returns an
//! error from the data port. The view wraps the risky source in
//! `{#while-loading}` / `{#while-error}` blocks so the template is never
//! blank — either the loading fallback or the error fallback is visible.
//!
//! This test proves the server-side contract:
//!   1. `/_deck/view/boom` returns 200 — the view compiles + SSR-renders without
//!      crashing (no white-screen, no 500).
//!   2. The SSR body is non-empty: the fallback infrastructure is present in the
//!      served HTML.
//!   3. The SSR body contains the `{#while-loading}` fallback text (`Loading`)
//!      — because SSR starts in loading state, the while-loading branch is the
//!      active one at first-paint. The while-error branch is a hidden template
//!      that the client runtime activates once the SSE channel delivers the
//!      error state.
//!   4. The HTML contains the `{#while-error}` element placeholder — deck
//!      emits `<template>` or `<sui-while-error…>` so the client can swap it
//!      in without a full re-render.
//!
//! No browser. No daemon. Ship-free.
//!
//! Run: `cargo test -p deck-conformance --test error_boundary`

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

/// The boom view SSR must return 200 and a non-empty body — the error boundary
/// prevents a white-screen even when the data source always errors.
#[tokio::test]
async fn boom_view_ssr_returns_200_non_empty() {
    let (status, body) = get("/_deck/view/boom").await;
    assert_eq!(
        status,
        StatusCode::OK,
        "boom view must SSR as 200 (error boundary prevents crash); got:\n{}",
        &body[..body.len().min(400)]
    );
    assert!(
        !body.trim().is_empty(),
        "SSR body must be non-empty (no white-screen)"
    );
}

/// The SSR body must contain the `{#while-loading}` fallback text — that branch
/// is active at first-paint because the SSE channel hasn't delivered yet.
/// This proves the view is non-blank on first load (the error-recovery path).
#[tokio::test]
async fn boom_view_ssr_renders_loading_fallback() {
    let (_, body) = get("/_deck/view/boom").await;
    assert!(
        body.contains("Loading") || body.contains("boom-fallback"),
        "SSR must render the loading fallback for the boom view; got head:\n{}",
        &body[..body.len().min(600)]
    );
}

/// The SSR payload must contain the `{#while-error}` structural placeholder so
/// the client runtime can activate the error fallback without a full re-render.
/// The compiler emits `<sui-while-error` or `<template` (hidden) for inactive branches.
#[tokio::test]
async fn boom_view_ssr_contains_while_error_placeholder() {
    let (_, body) = get("/_deck/view/boom").await;
    assert!(
        body.contains("sui-while-error") || body.contains("while-error"),
        "SSR must contain the while-error structural placeholder; got head:\n{}",
        &body[..body.len().min(600)]
    );
}

/// The boom view's SSR HTML must declare the view's heading — proves the view
/// itself renders beyond just the fallback structure.
#[tokio::test]
async fn boom_view_ssr_renders_view_heading() {
    let (_, body) = get("/_deck/view/boom").await;
    assert!(
        body.contains("Error Boundary"),
        "SSR must render the boom view heading; got head:\n{}",
        &body[..body.len().min(600)]
    );
}
