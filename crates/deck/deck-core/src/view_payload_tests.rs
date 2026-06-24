use std::fs;

use crate::DaemonMode;
use crate::test_lock::ProjectRootGuard;
use crate::view_host::{evict_ssr_cache, serve_view_host};
use crate::view_payload::serve_view_payload;
fn lock_with_dir(p: &std::path::Path) -> ProjectRootGuard {
    ProjectRootGuard::new(p)
}

fn body_str(resp: &crate::DeckResponse) -> String {
    String::from_utf8(resp.body.clone()).unwrap()
}


/// When a pre-compiled `.ssr.html` file exists in `dist/`, the payload
/// endpoint must return its contents in the `html` field rather than
/// re-rendering at request time.
#[test]
fn payload_uses_precompiled_ssr_html() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = lock_with_dir(tmp.path());

    let view_dir = tmp.path().join("views/ssr-payload-precompiled");
    let dist_dir = view_dir.join("dist");
    fs::create_dir_all(&dist_dir).unwrap();
    fs::write(view_dir.join("page.sui"), "<div><p>source</p></div>").unwrap();
    let ssr_content = "<div><p data-ssr=\"1\">precompiled</p></div>";
    fs::write(dist_dir.join("page.ssr.html"), ssr_content).unwrap();

    let resp = serve_view_payload(DaemonMode::Development, "ssr-payload-precompiled");
    assert_eq!(resp.status, 200);
    let v: serde_json::Value = serde_json::from_str(&body_str(&resp)).unwrap();
    assert_eq!(
        v["html"].as_str().unwrap(),
        ssr_content,
        "html must come from precompiled .ssr.html"
    );
}

/// When no `.ssr.html` file exists the endpoint gracefully falls back to
/// the runtime-rendered HTML (existing behaviour is preserved).
#[test]
fn payload_falls_back_to_runtime_render_without_ssr_file() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = lock_with_dir(tmp.path());

    let view_dir = tmp.path().join("views/ssr-payload-fallback");
    fs::create_dir_all(&view_dir).unwrap();
    fs::write(view_dir.join("page.sui"), "<div><p>runtime</p></div>").unwrap();

    let resp = serve_view_payload(DaemonMode::Development, "ssr-payload-fallback");
    assert_eq!(resp.status, 200);
    let v: serde_json::Value = serde_json::from_str(&body_str(&resp)).unwrap();
    let html = v["html"].as_str().unwrap();
    assert!(
        html.contains("<p>"),
        "runtime-rendered html must contain the template; got: {html}"
    );
}

/// `/view-host/<name>` with a matching `.ssr.html` must splice the SSR
/// content into the shell's `<div id="sui-root">`.
#[test]
fn view_host_splices_precompiled_ssr() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = lock_with_dir(tmp.path());

    let view_dir = tmp.path().join("views/ssr-host-test-splice");
    let dist_dir = view_dir.join("dist");
    fs::create_dir_all(&dist_dir).unwrap();
    fs::write(view_dir.join("page.sui"), "<p>hi</p>").unwrap();
    let ssr_content = "<p data-test=\"ssr-splice\">spliced</p>";
    fs::write(dist_dir.join("page.ssr.html"), ssr_content).unwrap();

    // Evict any stale cache entry from a previous run.
    evict_ssr_cache("ssr-host-test-splice");

    let resp = serve_view_host(DaemonMode::Development, "/_deck/view/ssr-host-test-splice");
    assert_eq!(resp.status, 200);
    let body = body_str(&resp);
    assert!(
        body.contains(ssr_content),
        "view-host must splice SSR content into shell; got: {body}"
    );
    assert!(
        body.contains("<div id=\"sui-root\""),
        "sui-root div must be present in shell; got: {body}"
    );
}

/// When no `.ssr.html` exists, `/view-host/<name>` live-emits SSR from
/// source so the client has real markup to morph onto (no blank flash).
#[test]
fn view_host_without_ssr_live_emits_from_source() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = lock_with_dir(tmp.path());

    let view_dir = tmp.path().join("views/ssr-host-test-empty");
    fs::create_dir_all(&view_dir).unwrap();
    fs::write(view_dir.join("page.sui"), "<p>hi</p>").unwrap();

    let resp = serve_view_host(DaemonMode::Development, "/_deck/view/ssr-host-test-empty");
    assert_eq!(resp.status, 200);
    let body = body_str(&resp);
    assert!(
        body.contains("<!doctype html>"),
        "shell must be valid HTML; got: {body}"
    );
    assert!(
        body.contains("<div id=\"sui-root\"")
            && !body.contains("data-view=\"ssr-host-test-empty\"></div>"),
        "live-emit must populate sui-root when no .ssr.html exists; got: {body}"
    );
    assert!(
        body.contains("<p"),
        "live-emit must include source content; got: {body}"
    );
}

// ── Mode-specific tests ───────────────────────────────────────────────────────

/// In production mode, view-host caches the SSR body keyed on the `.ssr.html`
/// artifact's mtime. The unchanged file is a hit (cheap, no re-read), and a
/// changed file (mtime advanced) is a miss that re-reads. This replaces the old
/// "stale-forever" contract: a re-emitted artifact must be picked up without a
/// restart, never pinned to the first body the daemon happened to read.
#[test]
fn view_host_prod_ssr_cache_revalidates_on_mtime() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = lock_with_dir(tmp.path());
    let view_name = "ssr-mode-cache-test";
    let view_dir = tmp.path().join(format!("views/{view_name}"));
    let dist_dir = view_dir.join("dist");
    fs::create_dir_all(&dist_dir).unwrap();
    fs::write(view_dir.join("page.sui"), "<p>hi</p>").unwrap();
    let ssr_file = dist_dir.join("page.ssr.html");
    fs::write(&ssr_file, "content-v1").unwrap();
    evict_ssr_cache(view_name);

    // First request in prod — populates cache with v1.
    let resp = serve_view_host(DaemonMode::Production, &format!("/_deck/view/{view_name}"));
    assert!(body_str(&resp).contains("content-v1"), "first prod request must read from disk");

    // Re-emit the artifact and advance its mtime — the prod cache MUST revalidate
    // and serve the new body, not the stale v1 it cached on the first request.
    fs::write(&ssr_file, "content-v2").unwrap();
    let now = fs::metadata(&ssr_file).and_then(|m| m.modified()).unwrap();
    let bumped = now + std::time::Duration::from_secs(5);
    filetime::set_file_mtime(&ssr_file, filetime::FileTime::from_system_time(bumped)).unwrap();

    let resp = serve_view_host(DaemonMode::Production, &format!("/_deck/view/{view_name}"));
    let body = body_str(&resp);
    assert!(
        body.contains("content-v2"),
        "prod must pick up the changed artifact once its mtime advances (stale-forever is fixed); got: {body}"
    );
    assert!(!body.contains("content-v1"), "prod must not pin the original body forever");
}

/// In development mode, view-host bypasses the SSR cache — each request
/// reads fresh content from disk.
#[test]
fn view_host_dev_bypasses_ssr_cache() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = lock_with_dir(tmp.path());
    let view_name = "ssr-mode-dev-bypass";
    let view_dir = tmp.path().join(format!("views/{view_name}"));
    let dist_dir = view_dir.join("dist");
    fs::create_dir_all(&dist_dir).unwrap();
    fs::write(view_dir.join("page.sui"), "<p>hi</p>").unwrap();
    let ssr_file = dist_dir.join("page.ssr.html");
    fs::write(&ssr_file, "dev-v1").unwrap();

    evict_ssr_cache(view_name);

    let resp = serve_view_host(DaemonMode::Development, &format!("/_deck/view/{view_name}"));
    assert!(body_str(&resp).contains("dev-v1"), "dev first request reads from disk");

    fs::write(&ssr_file, "dev-v2").unwrap();

    let resp = serve_view_host(DaemonMode::Development, &format!("/_deck/view/{view_name}"));
    let body = body_str(&resp);
    assert!(body.contains("dev-v2"), "dev must bypass cache and read fresh content");
}

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

/// The payload compile (parse + build + render_split) is mtime-cached on the
/// source `.sui` (SPLIT_CACHE). Behavior contract: an edited source (mtime
/// advanced) is picked up — never stale-pinned to the first compile. Asserts the
/// OUTPUT, not a hit-count, so it holds even if the cache is later changed or
/// removed (it guards against a cache that pins stale output, the real risk).
#[test]
fn payload_split_cache_revalidates_on_source_mtime() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = lock_with_dir(tmp.path());
    let view_name = "split-cache-revalidate";
    let view_dir = tmp.path().join(format!("views/{view_name}"));
    fs::create_dir_all(&view_dir).unwrap();
    let src = view_dir.join("page.sui");
    fs::write(&src, "<p>split-v1</p>").unwrap();
    crate::view_payload::evict_split_cache(view_name);

    // First serve compiles + caches v1. No dist/.ssr.html exists, so the payload
    // html field comes from split.html — i.e. the cached compile output.
    let resp = serve_view_payload(DaemonMode::Production, view_name);
    assert_eq!(resp.status, 200);
    assert!(body_str(&resp).contains("split-v1"), "first serve must render source v1");

    // Edit the source and advance its mtime — must be a MISS that recompiles.
    fs::write(&src, "<p>split-v2</p>").unwrap();
    let now = fs::metadata(&src).and_then(|m| m.modified()).unwrap();
    let bumped = now + std::time::Duration::from_secs(5);
    filetime::set_file_mtime(&src, filetime::FileTime::from_system_time(bumped)).unwrap();

    let body = body_str(&serve_view_payload(DaemonMode::Production, view_name));
    assert!(body.contains("split-v2"), "edited source (advanced mtime) must be picked up; got: {body}");
    assert!(!body.contains("split-v1"), "must not stale-pin the first compile");
}
