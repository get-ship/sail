//! Slice 7 — production mode: immutable cache headers + ETags on framework assets.
//!
//! With `DaemonMode::Production`:
//!   - `/_deck/runtime.js` must carry `Cache-Control: immutable, max-age=…` AND
//!     an `ETag` header.
//!   - In Development the same asset must NOT be immutable-cached.
//!
//! (The route-payload error-redaction cases moved out with `ship-views`; those
//! `/_deck/views/*` routes are no longer served by this harness.)
//!
//! Uses tower `oneshot` — no server, no browser.
//!
//! Run: `cargo test -p deck-conformance --test prod_mode`

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use axum::response::Response;
use deck::DaemonMode;
use deck_conformance::{deck_app_with_mode, default_app_dir};
use tower::ServiceExt;

async fn prod_get(path: &str) -> Response {
    deck_app_with_mode(default_app_dir(), DaemonMode::Production)
        .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
        .await
        .expect("router response")
}

fn hdr(resp: &Response, name: header::HeaderName) -> Option<String> {
    resp.headers()
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(str::to_string)
}

/// In Production, `/_deck/runtime.js` must carry an immutable Cache-Control
/// header and an ETag, enabling the browser to cache it forever.
#[tokio::test]
async fn runtime_js_is_immutable_with_etag_in_prod() {
    let resp = prod_get("/_deck/runtime.js").await;
    assert_eq!(resp.status(), StatusCode::OK);

    let cc = hdr(&resp, header::CACHE_CONTROL).unwrap_or_default();
    assert!(
        cc.contains("immutable"),
        "Cache-Control must contain 'immutable' in prod; got: {cc}"
    );
    assert!(
        cc.contains("max-age"),
        "Cache-Control must contain 'max-age' in prod; got: {cc}"
    );

    let etag = hdr(&resp, header::ETAG);
    assert!(
        etag.is_some(),
        "ETag must be present in prod for framework assets"
    );
    // ETag must be a quoted string per RFC 7232.
    let etag = etag.unwrap();
    assert!(
        etag.starts_with('"') && etag.ends_with('"'),
        "ETag must be a quoted string; got: {etag}"
    );
}

/// In Development mode the same endpoint must NOT be immutable-cached.
#[tokio::test]
async fn runtime_js_is_not_cached_in_dev() {
    let resp = deck_app_with_mode(default_app_dir(), DaemonMode::Development)
        .oneshot(
            Request::builder()
                .uri("/_deck/runtime.js")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("router response");
    assert_eq!(resp.status(), StatusCode::OK);

    let cc = hdr(&resp, header::CACHE_CONTROL).unwrap_or_default();
    assert!(
        cc.contains("no-cache"),
        "Cache-Control must be no-cache in dev; got: {cc}"
    );
    assert!(
        !cc.contains("immutable"),
        "Cache-Control must NOT be immutable in dev; got: {cc}"
    );
    assert!(
        resp.headers().get(header::ETAG).is_none(),
        "dev must not emit an ETag for cached assets"
    );
}
