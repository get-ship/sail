use crate::DaemonMode;

// ── Asset handler cache headers ───────────────────────────────────────────────

#[test]
fn asset_dev_mode_no_cache() {
    let resp = crate::assets::serve_sui_runtime(DaemonMode::Development);
    assert_eq!(resp.cache_control, "no-cache");
}

#[test]
fn asset_prod_mode_immutable() {
    let resp = crate::assets::serve_sui_runtime(DaemonMode::Production);
    assert!(resp.cache_control.contains("immutable"), "prod asset must be immutable; got: {}", resp.cache_control);
    assert!(resp.cache_control.contains("max-age=31536000"), "prod asset must have max-age; got: {}", resp.cache_control);
}

#[test]
fn asset_prod_mode_has_etag() {
    let resp = crate::assets::serve_sui_runtime(DaemonMode::Production);
    assert!(resp.etag.is_some(), "prod asset must have ETag");
}

#[test]
fn asset_dev_mode_no_etag() {
    let resp = crate::assets::serve_sui_runtime(DaemonMode::Development);
    assert!(resp.etag.is_none(), "dev asset must not have ETag");
}

// ── Shared chunk serving (#267 per-route resolver, multi-entry build) — PURGED ──
// The `serve_sui_chunk` runtime read of `<root>/.deck/<file>` was the
// framework's hardcoded `.deck/` reach (deck-purge-root). The route
// `/_deck/{file}` is gone; chunk serving is a follow-up. The rejection
// shape (path traversal / nested / non-.js / missing) is preserved by
// `serve_scaffold_asset`'s path-traversal guard — same rule, different path.

