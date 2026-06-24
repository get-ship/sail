//! Host-injection tests for 2c:
//! - serve_view_payload / serve_view_host inject action+method on <form mutation>
//! - author action= is left untouched
//! - <data live> payload scope carries __subscribe__ from the host

use std::fs;

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

/// serve_view_payload on a mutation form view injects action + method into html.
#[test]
fn payload_injects_mutation_action_into_html() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = lock_with_dir(tmp.path());
    let view_dir = tmp.path().join("views/host-inject-form");
    fs::create_dir_all(&view_dir).unwrap();
    fs::write(
        view_dir.join("page.sui"),
        r#"<form mutation="ship.demo.create"><button type="submit">Go</button></form>"#,
    )
    .unwrap();

    let resp = serve_view_payload(DaemonMode::Development, "host-inject-form");
    assert_eq!(resp.status, 200);
    let v: serde_json::Value = serde_json::from_str(&body_str(&resp)).unwrap();
    let html = v["html"].as_str().unwrap();

    // The host must have injected action + method.
    assert!(
        html.contains("/_deck/mutation/host-inject-form?form=b0"),
        "host must inject mutation action into payload html; got: {html}"
    );
    assert!(
        html.contains(r#"method="post""#),
        "host must inject method=post; got: {html}"
    );
    // The structural marker must still be present.
    assert!(
        html.contains(r#"mutation="b0""#),
        "mutation marker must be preserved; got: {html}"
    );
}

/// serve_view_payload leaves an author-provided action= untouched.
#[test]
fn payload_leaves_author_action_untouched() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = lock_with_dir(tmp.path());
    let view_dir = tmp.path().join("views/host-inject-form-author");
    fs::create_dir_all(&view_dir).unwrap();
    fs::write(
        view_dir.join("page.sui"),
        r#"<form action="/custom" mutation="ship.demo.create"></form>"#,
    )
    .unwrap();

    let resp = serve_view_payload(DaemonMode::Development, "host-inject-form-author");
    assert_eq!(resp.status, 200);
    let v: serde_json::Value = serde_json::from_str(&body_str(&resp)).unwrap();
    let html = v["html"].as_str().unwrap();

    assert!(
        html.contains(r#"action="/custom""#),
        "author action must be preserved; got: {html}"
    );
    assert!(
        !html.contains("/_deck/mutation"),
        "host must not inject when author action present; got: {html}"
    );
}

/// serve_view_payload on a <data live> view injects __subscribe__ into scope.
#[test]
fn payload_injects_subscribe_into_scope_for_live_view() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = lock_with_dir(tmp.path());
    let view_dir = tmp.path().join("views/host-inject-live");
    fs::create_dir_all(&view_dir).unwrap();
    fs::write(
        view_dir.join("page.sui"),
        r#"<data name="t" source="ship.demo.clock()" live /><p>hello</p>"#,
    )
    .unwrap();

    let resp = serve_view_payload(DaemonMode::Development, "host-inject-live");
    assert_eq!(resp.status, 200);
    let v: serde_json::Value = serde_json::from_str(&body_str(&resp)).unwrap();
    let scope = &v["scope"];
    assert!(
        scope.is_object(),
        "scope must be an object; got: {scope}"
    );
    let sub = scope["__subscribe__"].as_str().unwrap_or("");
    assert_eq!(
        sub, "/_deck/views/host-inject-live/sse",
        "host must inject __subscribe__ for live view; got: {sub}"
    );
}

/// serve_view_payload_subpath injects __subscribe__ with the sub-path folded in.
#[test]
fn payload_subpath_injects_subscribe_with_sub() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = lock_with_dir(tmp.path());
    let view_dir = tmp.path().join("views/host-inject-live-sub");
    let sub_dir = view_dir.join("[id]");
    fs::create_dir_all(&sub_dir).unwrap();
    fs::write(view_dir.join("page.sui"), "<p>root</p>").unwrap();
    fs::write(
        sub_dir.join("page.sui"),
        r#"<data name="t" source="ship.demo.clock()" live /><p>hello</p>"#,
    )
    .unwrap();

    let resp = serve_view_payload_subpath(DaemonMode::Development, "host-inject-live-sub", "42");
    assert_eq!(resp.status, 200);
    let v: serde_json::Value = serde_json::from_str(&body_str(&resp)).unwrap();
    let sub = v["scope"]["__subscribe__"].as_str().unwrap_or("");
    assert_eq!(
        sub, "/_deck/views/host-inject-live-sub/42/sse",
        "sub-path live view must use combined name/sub for __subscribe__; got: {sub}"
    );
}

/// serve_view_host injects action+method into the SSR content for a mutation form.
#[test]
fn view_host_injects_mutation_action_into_ssr() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = lock_with_dir(tmp.path());
    let view_dir = tmp.path().join("views/host-host-inject-form");
    fs::create_dir_all(&view_dir).unwrap();
    fs::write(
        view_dir.join("page.sui"),
        r#"<form mutation="ship.demo.create"><button type="submit">Go</button></form>"#,
    )
    .unwrap();

    let resp = serve_view_host(DaemonMode::Development, "/_deck/view/host-host-inject-form");
    assert_eq!(resp.status, 200);
    let body = body_str(&resp);
    assert!(
        body.contains("/_deck/mutation/host-host-inject-form?form=b0"),
        "view-host must inject mutation action; got: {body}"
    );
    assert!(
        body.contains(r#"method="post""#),
        "view-host must inject method=post; got: {body}"
    );
}

/// serve_view_host with a pre-compiled .ssr.html that already has a mutation
/// marker still injects the action (idempotent path: already-injected is fine,
/// the key point is the marker-only artifact gets the injection).
#[test]
fn view_host_injects_into_precompiled_ssr_html() {
    use crate::view_host::evict_ssr_cache;
    let tmp = tempfile::tempdir().unwrap();
    let _g = lock_with_dir(tmp.path());
    let view_dir = tmp.path().join("views/host-host-precompiled-inject");
    let dist_dir = view_dir.join("dist");
    fs::create_dir_all(&dist_dir).unwrap();
    fs::write(view_dir.join("page.sui"), "<p>hi</p>").unwrap();
    // Pre-baked .ssr.html with only the mutation marker (no action/method yet).
    let ssr_content = r#"<form mutation="b0"><button>Go</button></form>"#;
    fs::write(dist_dir.join("page.ssr.html"), ssr_content).unwrap();
    evict_ssr_cache("host-host-precompiled-inject");

    let resp = serve_view_host(DaemonMode::Development, "/_deck/view/host-host-precompiled-inject");
    assert_eq!(resp.status, 200);
    let body = body_str(&resp);
    assert!(
        body.contains("/_deck/mutation/host-host-precompiled-inject?form=b0"),
        "host must inject action into precompiled .ssr.html; got: {body}"
    );
}
