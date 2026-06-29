//! Host-injection tests that do NOT depend on a server render:
//! `<data live>` payload scope carries `__subscribe__` from the host.
//!
//! The <form mutation> action-injection tests moved to the conformance /
//! deck-ssr layer: they need server-rendered HTML, which now comes solely from
//! ship-js (V8). `inject_form_actions` itself is unit-tested in `form_inject`;
//! its application to real SSR output is covered where V8 is linked.

use std::fs;

use deck_core::DaemonMode;
use crate::test_lock::ProjectRootGuard;
use crate::view_payload::{serve_view_payload, serve_view_payload_subpath};

fn lock_with_dir(p: &std::path::Path) -> ProjectRootGuard {
    ProjectRootGuard::new(p)
}

fn body_str(resp: &deck_core::DeckResponse) -> String {
    String::from_utf8(resp.body.clone()).unwrap()
}

/// serve_view_payload on a `<data live>` view injects `__subscribe__` into scope.
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
    assert!(scope.is_object(), "scope must be an object; got: {scope}");
    let sub = scope["__subscribe__"].as_str().unwrap_or("");
    assert_eq!(
        sub, "/_deck/views/host-inject-live/sse",
        "host must inject __subscribe__ for live view; got: {sub}"
    );
}

/// serve_view_payload_subpath injects `__subscribe__` with the sub-path folded in.
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
