//! Payload shape, error-body, and first-party-view tests for `serve_view_payload`
//! and `serve_view_host`. Companion to `view_payload_tests.rs` (SSR/cache
//! tests live there).

use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

use crate::DaemonMode;
use crate::view_host::serve_view_host;
use crate::view_payload::{serve_view_payload, serve_view_payload_subpath};

use crate::test_lock::ProjectRootGuard;

fn lock_with_dir(p: &std::path::Path) -> ProjectRootGuard {
    ProjectRootGuard::new(p)
}


fn body_str(resp: &crate::DeckResponse) -> String {
    String::from_utf8(resp.body.clone()).unwrap()
}

fn repo_root() -> &'static std::path::Path {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| {
        let mut cur = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        loop {
            let cargo = cur.join("Cargo.toml");
            if cargo.is_file()
                && std::fs::read_to_string(&cargo)
                    .map(|s| s.contains("[workspace]"))
                    .unwrap_or(false)
            {
                return cur;
            }
            if !cur.pop() {
                panic!("could not find repo root from {}", env!("CARGO_MANIFEST_DIR"));
            }
        }
    });
    ROOT.get().unwrap()
}

#[test]
fn top_level_view_payload_shape() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = lock_with_dir(tmp.path());
    let view_dir = tmp.path().join(".ship/views/demo");
    fs::create_dir_all(&view_dir).unwrap();
    fs::write(view_dir.join("page.sui"),
        r#"<div><p>{count()}</p></div>"#).unwrap();
    fs::write(view_dir.join("index.controller.ts"),
        "import { controller, signal } from '@ship/sui-runtime'\nexport default controller({ scope: () => ({ count: signal(0) }) })\n").unwrap();
    fs::write(view_dir.join("index.css"), "p { color: red; }").unwrap();

    let resp = serve_view_payload(DaemonMode::Development, "demo");
    assert_eq!(resp.status, 200);
    let v: serde_json::Value = serde_json::from_str(&body_str(&resp)).unwrap();
    assert_eq!(v["name"], "demo");
    assert!(v["sub"].is_null());
    let html = v["html"].as_str().unwrap();
    assert!(html.contains("<p>"), "html should contain rendered template, got {html}");
    assert!(!html.contains("__set_init__"),
        "html must NOT include the script island — scope is a separate field");
    assert_eq!(v["css"], "p { color: red; }");
    assert!(v["scope"].is_object(), "scope should be a JSON object, got {}", v["scope"]);
    assert_eq!(v["moduleUrl"], "/_deck/views/demo/view.js");
}

#[test]
fn sub_path_payload_concatenates_css() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = lock_with_dir(tmp.path());
    let view_dir = tmp.path().join(".ship/views/demo");
    let sub_dir = view_dir.join("[name]");
    fs::create_dir_all(&sub_dir).unwrap();
    fs::write(view_dir.join("page.sui"), r#"<div></div>"#).unwrap();
    fs::write(view_dir.join("index.css"), ".parent {}").unwrap();
    fs::write(sub_dir.join("page.sui"), r#"<span></span>"#).unwrap();
    fs::write(sub_dir.join("index.css"), ".child {}").unwrap();

    let resp = serve_view_payload_subpath(DaemonMode::Development, "demo", "anything");
    assert_eq!(resp.status, 200);
    let v: serde_json::Value = serde_json::from_str(&body_str(&resp)).unwrap();
    assert_eq!(v["sub"], "anything");
    let css = v["css"].as_str().unwrap();
    assert!(css.contains(".parent"), "css missing parent: {css}");
    assert!(css.contains(".child"), "css missing child: {css}");
    assert!(
        css.find(".parent").unwrap() < css.find(".child").unwrap(),
        "parent must precede child for cascade order"
    );
    assert_eq!(v["moduleUrl"], "/_deck/views/demo/view.js?sub=anything");
}

#[test]
fn unknown_view_returns_404_json() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = lock_with_dir(tmp.path());
    let resp = serve_view_payload(DaemonMode::Development, "nope");
    assert_eq!(resp.status, 404);
    let v: serde_json::Value = serde_json::from_str(&body_str(&resp)).unwrap();
    assert_eq!(v["error"]["view_not_found"]["name"], "nope");
}

/// The human-facing host route serves a rendered HTML 404 — not the raw JSON
/// the programmatic payload endpoint emits — and surfaces the views the daemon
/// CAN serve so the founder can recover. The JSON contract above is untouched.
#[test]
fn host_unknown_view_serves_html_404_with_available_views() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = lock_with_dir(tmp.path());

    // Two real views so the not-found page has something to list.
    for name in ["counter", "dashboard"] {
        let dir = tmp.path().join(format!(".ship/views/{name}"));
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("page.sui"), "<p>hi</p>").unwrap();
    }

    let resp = serve_view_host(DaemonMode::Development, "/_deck/view/home");
    assert_eq!(resp.status, 404, "missing view must 404");
    assert!(
        resp.content_type.contains("text/html"),
        "human-facing 404 must be HTML, got: {}",
        resp.content_type
    );
    let body = body_str(&resp);
    assert!(body.contains("<!doctype html>"), "must be a rendered page; got: {body}");
    assert!(body.contains("home"), "must name the requested view; got: {body}");
    assert!(body.contains("href=\"/\""), "must link back to the shell root; got: {body}");
    // Available-views list pulled from the resolver's roots.
    assert!(body.contains("/_deck/view/counter"), "must list servable views; got: {body}");
    assert!(body.contains("/_deck/view/dashboard"), "must list servable views; got: {body}");
    // The page must NOT leak the JSON error shape the payload endpoint uses.
    assert!(!body.contains("\"view_not_found\""), "must not dump raw JSON; got: {body}");
}

#[test]
fn invalid_name_returns_404_json() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = lock_with_dir(tmp.path());
    let resp = serve_view_payload(DaemonMode::Development, "..");
    assert_eq!(resp.status, 404);
    let v: serde_json::Value = serde_json::from_str(&body_str(&resp)).unwrap();
    assert!(v["error"]["invalid_name"].is_object(), "got: {v}");
}

#[test]
fn compile_error_returns_500_with_position() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = lock_with_dir(tmp.path());
    let view_dir = tmp.path().join(".ship/views/broken");
    fs::create_dir_all(&view_dir).unwrap();
    fs::write(view_dir.join("page.sui"), "<div><span></div>").unwrap();

    let resp = serve_view_payload(DaemonMode::Development, "broken");
    assert_eq!(resp.status, 500);
    let v: serde_json::Value = serde_json::from_str(&body_str(&resp)).unwrap();
    let body = &v["error"]["compile_error"];
    assert_eq!(body["name"], "broken");
    assert!(body["line"].as_u64().unwrap() >= 1);
    assert!(!body["message"].as_str().unwrap().is_empty());
}


