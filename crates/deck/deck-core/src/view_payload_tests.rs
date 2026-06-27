use std::fs;

use crate::DaemonMode;
use crate::test_lock::ProjectRootGuard;
use crate::view_payload::serve_view_payload;

fn lock_with_dir(p: &std::path::Path) -> ProjectRootGuard {
    ProjectRootGuard::new(p)
}

fn body_str(resp: &crate::DeckResponse) -> String {
    String::from_utf8(resp.body.clone()).unwrap()
}

// SSR-output tests (precompiled `.ssr.html`, render-eval live-emit, the
// view-host splice/cache) were removed with the render-eval engine: first-paint
// HTML now comes solely from ship-js, so its coverage lives in the conformance /
// deck-ssr layer (where V8 is linked), not here. These tests exercise behavior
// that does NOT depend on a server render: cache headers and compile-error shape.

/// Production mode: view-payload response has must-revalidate cache header and ETag.
#[test]
fn view_payload_prod_cache_header_and_etag() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = lock_with_dir(tmp.path());
    let view_dir = tmp.path().join("views/prod-cache-check");
    fs::create_dir_all(&view_dir).unwrap();
    fs::write(view_dir.join("page.sui"), "<div><p>hello</p></div>").unwrap();

    let resp = serve_view_payload(DaemonMode::Production, "prod-cache-check");
    assert_eq!(resp.status, 200);
    assert!(resp.cache_control.contains("must-revalidate"), "prod must have must-revalidate; got: {}", resp.cache_control);
    assert!(resp.etag.is_some(), "prod must have ETag header");
}

/// Development mode: view-payload response has no-cache header and no ETag.
#[test]
fn view_payload_dev_no_cache_no_etag() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = lock_with_dir(tmp.path());
    let view_dir = tmp.path().join("views/dev-cache-check");
    fs::create_dir_all(&view_dir).unwrap();
    fs::write(view_dir.join("page.sui"), "<div><p>hello</p></div>").unwrap();

    let resp = serve_view_payload(DaemonMode::Development, "dev-cache-check");
    assert_eq!(resp.status, 200);
    assert_eq!(resp.cache_control, "no-cache", "dev must use no-cache; got: {}", resp.cache_control);
    assert!(resp.etag.is_none(), "dev must not have ETag header");
}

/// Production mode: compile errors in view-payload are redacted (no source positions).
#[test]
fn view_payload_prod_compile_error_redacted() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = lock_with_dir(tmp.path());
    let view_dir = tmp.path().join("views/broken-prod");
    fs::create_dir_all(&view_dir).unwrap();
    fs::write(view_dir.join("page.sui"), "<div><span></div>").unwrap();

    let resp = serve_view_payload(DaemonMode::Production, "broken-prod");
    assert_eq!(resp.status, 500);
    let v: serde_json::Value = serde_json::from_str(&body_str(&resp)).unwrap();
    assert!(
        v["error"]["redacted_error"].is_object(),
        "prod must redact compile errors; got: {v}"
    );
}

/// Development mode: compile errors include source position detail.
#[test]
fn view_payload_dev_compile_error_includes_detail() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = lock_with_dir(tmp.path());
    let view_dir = tmp.path().join("views/broken-dev");
    fs::create_dir_all(&view_dir).unwrap();
    fs::write(view_dir.join("page.sui"), "<div><span></div>").unwrap();

    let resp = serve_view_payload(DaemonMode::Development, "broken-dev");
    assert_eq!(resp.status, 500);
    let v: serde_json::Value = serde_json::from_str(&body_str(&resp)).unwrap();
    let body = &v["error"]["compile_error"];
    assert!(body["line"].as_u64().unwrap() >= 1, "dev must include line number; got: {v}");
    assert!(!body["message"].as_str().unwrap_or("").is_empty(), "dev must include message; got: {v}");
}
