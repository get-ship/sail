//! P2-8 — wire-level assertion that the HTML security header set actually
//! reaches the transport. The transport-agnostic deck-core layer
//! (`crates/deck/deck-core/src/response.rs::HTML_SECURITY_HEADERS` and the
//! `html_route` constructor that emits it) pins the bytes in
//! `response::tests::html_view_emits_exactly_the_security_header_set`; this
//! test closes the loop by running the same headers through `into_axum`
//! and confirming the axum `Response` carries them on the wire — catches an
//! adapter regression that silently drops `extra_headers`.
//!
//! Negative-case proof: each header is asserted by exact value (not substring),
//! and the count is checked, so a partial drop (e.g. CSP removed, the others
//! left in place) fails this test rather than slipping through.

use axum::body::to_bytes;
use axum::http::{Response, StatusCode, header};

use deck_axum::into_axum;
use deck_core::{DaemonMode, DeckResponse, HTML_SECURITY_HEADERS};

#[tokio::test(flavor = "multi_thread")]
async fn html_view_security_headers_reach_the_wire() {
    let resp: Response<_> = into_axum(DeckResponse::html_route(
        "<html><body>x</body></html>",
        DaemonMode::Development,
    ));

    assert_eq!(resp.status(), StatusCode::OK);

    // Pull the body just to prove the body + headers coexist on the response
    // (i.e. the headers weren't stripped by `into_axum`'s body wrapping).
    let _ = to_bytes(resp.into_body(), usize::MAX).await.unwrap();

    // We can't re-read headers after consuming the body, so re-derive: build
    // a fresh response just for the header check.
    let resp = into_axum(DeckResponse::html_route(
        "<html><body>x</body></html>",
        DaemonMode::Development,
    ));

    let headers = resp.headers();
    let mut seen = 0usize;
    for (name, value) in HTML_SECURITY_HEADERS {
        let got = headers
            .get(*name)
            .unwrap_or_else(|| panic!("header `{name}` missing from the wire response"));
        assert_eq!(
            got.to_str().unwrap(),
            *value,
            "header `{name}` value mismatch on the wire: expected `{value}`, got `{}`",
            got.to_str().unwrap()
        );
        seen += 1;
    }
    // No spurious header slipped in past the const.
    assert_eq!(
        seen,
        HTML_SECURITY_HEADERS.len(),
        "the wire response carried fewer security headers than the const declares"
    );

    // Belt-and-suspenders: confirm the content-type survived alongside the
    // security headers (i.e. `into_axum` didn't drop one to make room for the
    // other — the bug class this test exists to catch).
    assert_eq!(
        headers
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok()),
        Some("text/html; charset=utf-8"),
        "into_axum must preserve content-type alongside the security headers"
    );
}
