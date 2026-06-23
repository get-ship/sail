//! Slice 2 — controller → bundling.
//!
//! A view WITH a controller can't be served from source: its compiled module
//! imports `./controller`, so deck must produce a Rolldown bundle. This proves
//! deck's on-demand bundler (wired as an `Extension` by `deck_app`) builds
//! `<app>/.deck/views/counter/view.js` on the first request and serves it —
//! the *bundling* + *build-output* reliability axes.
//!
//! The dynamic value (count = 0, then increments) is NOT asserted here: the
//! signal's initial value lives in the controller (JS), so SSR renders an empty
//! reactive slot and the value only appears after client hydration. That full
//! loop is the e2e test's job (`e2e/`).
//!
//! Run: `cargo test -p deck-conformance --test bundling`

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

/// The controller view renders its static structure server-side. The dynamic
/// `{count()}` is an empty hydration slot (its value is controller state, not a
/// constant) — so we assert structure + the anchor, not the value.
#[tokio::test]
async fn ssr_renders_counter_structure() {
    let (status, body) = get("/_deck/view/counter").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("Counter"), "SSR should render the counter heading");
    assert!(body.contains("increment"), "SSR should render the button");
    assert!(
        body.contains("<!--sui:text:"),
        "the dynamic {{count()}} should be an empty, hydratable text anchor"
    );
}

/// The first request for a controller view's setup module triggers an
/// on-demand Rolldown build, written to `<app>/.deck/views/counter/view.js`,
/// then served. Proves deck's host-wired bundler works with no daemon.
#[tokio::test]
async fn controller_view_builds_a_bundle_on_demand() {
    let bundle = default_app_dir().join(".deck/views/counter/view.js");
    // Remove any prior artifact so we prove THIS request built it.
    let _ = std::fs::remove_file(&bundle);

    let (status, body) = get("/_deck/views/counter/view.js").await;
    assert_eq!(
        status,
        StatusCode::OK,
        "view.js should build + serve; got head:\n{}",
        &body[..body.len().min(600)]
    );
    assert!(
        bundle.exists(),
        "the Rolldown bundle should be written to {}",
        bundle.display()
    );
    // The runtime stays external (single-instance) — the bundle imports it,
    // never inlines it.
    assert!(
        body.contains("sui-runtime"),
        "bundle should import the external runtime singleton; head:\n{}",
        &body[..body.len().min(600)]
    );
}
