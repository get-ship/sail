//! P0-2 — a broken paired `controller.ts` must surface the REAL compile error.
//!
//! Failure mode: `serve_sui_view_setup` builds a view bundle in-process on a
//! cold miss. When the build fails (e.g. a syntax error in the paired
//! `controller.ts`), the handler used to swallow the error (`let _ = b.build…`)
//! and fall through to deck-core's generic "the in-process bundler must build
//! views/{name}/view" message — masking the actual diagnostic and leaving the
//! author with no actionable error. The fix surfaces the real bundler error.
//!
//! This drives the real Rolldown path (the linked sui-compiler compiles the
//! `.sui`, then Rolldown transpiles the sibling `.ts` and fails on the syntax
//! error), so it proves the end-to-end handler behaviour, not a stub.

use axum::body::to_bytes;
use axum::extract::{Extension, Path, Query};

use ship_views::serve_sui_view_setup;
use deck_bundler::{Bundler, DeckConfig};
use deck_core::{DaemonMode, ViewSetupQuery};

fn write(path: &std::path::Path, body: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, body).unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn broken_controller_surfaces_real_compile_error_not_generic_fallback() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().to_path_buf();
    // A trivial view whose paired controller (`page.sui` → `controller.ts`) has
    // a hard syntax error: `const x = ;` does not parse.
    write(&root.join("views/broken/page.sui"), "<div>broken</div>\n");
    write(
        &root.join("views/broken/controller.ts"),
        "export default { setup() { const x = ; return {} } }\n",
    );

    let bundler = Bundler::start(root.clone(), None, DeckConfig::defaults());

    let resp = serve_sui_view_setup(
        Some(Extension(DaemonMode::Development)),
        Some(Extension(bundler)),
        Path("broken".to_string()),
        Query(ViewSetupQuery { sub: None }),
    )
    .await;

    let status = resp.status();
    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let text = String::from_utf8_lossy(&body);

    assert_eq!(status, 500, "a compile failure is a 500, not a 200 or 404; body:\n{text}");
    // The handler's real-error path ran — NOT deck-core's generic fallback.
    assert!(
        text.contains("failed to compile"),
        "must carry the handler's real-error prefix; got:\n{text}"
    );
    assert!(
        !text.contains("must build views/broken/view"),
        "must NOT be the masked generic fallback; got:\n{text}"
    );
    // The actual bundler diagnostic is included (the `BundlerError::Bundle`
    // chain), naming the failing controller — the actionable detail P0-2 surfaces.
    assert!(
        text.contains("bundle failed for") || text.contains("controller"),
        "must include the real bundler/compile diagnostic; got:\n{text}"
    );
}
