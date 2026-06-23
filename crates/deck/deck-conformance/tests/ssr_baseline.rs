//! Slice 1 — SSR baseline + asset handling.
//!
//! Proves deck serves a controller-less view with NO build step, NO daemon,
//! NO ship: the view-host compiles the `.sui` on the fly, server-renders it,
//! and splices the SSR snapshot into the host HTML. Covers the first two
//! reliability axes — build output (the served HTML) and asset handling.
//!
//! Run: `cargo test -p deck-conformance`

use axum::body::Body;
use axum::http::{Request, StatusCode};
use deck_conformance::{deck_app, default_app_dir};
use tower::ServiceExt; // for `oneshot`

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

/// `/_deck/view/hello` → 200, and the server-rendered view markup is in the host HTML.
#[tokio::test]
async fn ssr_splices_rendered_view_into_host() {
    let (status, body) = get("/_deck/view/hello").await;
    assert_eq!(status, StatusCode::OK, "view-host should serve /_deck/view/hello");
    assert!(
        body.contains("Hello from deck"),
        "SSR snapshot must be spliced into the host HTML; got:\n{body}"
    );
}

/// deck SSR-evaluates the constant `1 + 1` → `2` AND wraps reactive text in a
/// hydration anchor (`<!--sui:text:ID-->2<!--/sui-->`) so the client adopts the
/// node. The value proves server-side eval; the anchor proves hydratability.
#[tokio::test]
async fn ssr_evaluates_constant_and_emits_hydration_anchor() {
    let (_status, body) = get("/_deck/view/hello").await;
    let stripped = strip_html_comments(&body);
    assert!(
        stripped.contains("<strong>2</strong>"),
        "expected SSR to fold `1 + 1` → 2 (anchors stripped); raw:\n{body}"
    );
    assert!(
        body.contains("<!--sui:text:"),
        "reactive text should carry a `<!--sui:text:…-->` hydration anchor"
    );
}

/// Honesty: an unknown view is a real 404, never a dressed-up 200.
#[tokio::test]
async fn missing_view_is_404_not_200() {
    let (status, _body) = get("/_deck/view/does-not-exist").await;
    assert_eq!(status, StatusCode::NOT_FOUND, "missing view must 404");
}

/// Strip HTML comments so SSR hydration anchors don't defeat structural
/// assertions on the rendered markup.
fn strip_html_comments(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(start) = rest.find("<!--") {
        out.push_str(&rest[..start]);
        match rest[start..].find("-->") {
            Some(end) => rest = &rest[start + end + 3..],
            None => {
                rest = "";
                break;
            }
        }
    }
    out.push_str(rest);
    out
}
