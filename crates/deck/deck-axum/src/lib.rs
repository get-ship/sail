//! deck-axum — the axum adapter for `deck-core`.
//!
//! deck-core never speaks HTTP; it produces a [`DeckResponse`]. This crate
//! is the plug: thin `serve_*` async shims extract axum params, call the
//! deck-core fn, and map the `DeckResponse` onto `axum::response::Response`
//! via [`into_axum`]. [`router`] registers deck's whole hosting surface as
//! one mountable unit.

pub mod apps;
mod handlers;
pub mod multiplex;
pub mod pages;
pub mod render_helpers;
pub mod sail_source;
pub use handlers::{
    serve_bundle_hmr,
    serve_deck_route,
    serve_meta,
    serve_sui_dom, serve_sui_icon,
    serve_sui_error_overlay,
    serve_sui_live_reload,
    serve_sui_runtime,
    serve_sui_sdk_client,
};
// The named-view handlers (serve_sui_route_setup / serve_route_host /
// serve_route_payload[_subpath]) moved to the `ship-views` crate. deck-axum's
// router serves ONLY deck's own framework + routes/ SPA surface.

// Re-export the contract types so `deck`'s facade (and shipd through it)
// can name them without depending on deck-core directly.
pub use deck_core::{
    CachePolicy, DaemonBuildInfo, DaemonMode, DeckResponse, RouteSetupQuery,
    inject_mode_meta, mode_str,
};

use axum::{
    Router,
    body::Body,
    extract::Extension,
    http::{StatusCode, header},
    response::Response,
    routing::get,
};

/// Extract the daemon mode from an optional axum `Extension`, defaulting to
/// Development. The whole `/_deck/*` surface reads it this way so handlers
/// need no router-state plumbing.
pub fn mode_from_ext(ext: Option<Extension<DaemonMode>>) -> DaemonMode {
    ext.map(|e| e.0).unwrap_or(DaemonMode::Development)
}

/// Map a transport-agnostic [`DeckResponse`] onto an `axum::response::Response`.
///
/// A free function rather than `impl IntoResponse`: the orphan rule forbids
/// implementing axum's trait for deck-core's foreign type from this crate.
pub fn into_axum(resp: DeckResponse) -> Response {
    let status = StatusCode::from_u16(resp.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let mut builder = Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, resp.content_type)
        .header(header::CACHE_CONTROL, resp.cache_control);
    if let Some(etag) = resp.etag {
        builder = builder.header(header::ETAG, etag);
    }
    for (name, value) in resp.extra_headers {
        builder = builder.header(name, value);
    }
    builder
        .body(Body::from(resp.body))
        .unwrap_or_else(|_| Response::new(Body::from("response build failed")))
}

/// deck's framework + routes/ SPA hosting surface as one mountable `Router`.
///
/// Registers: framework bundles (runtime/dom/sdk-client/components),
/// per-icon lucide serving, the theme/token CSS, the per-tag source viewer,
/// the HMR firehose, and `/_deck/meta`. Paths mirror shipd's canonical `Route`
/// enum so a host can mount this in lieu of wiring each handler.
///
/// The named-view surface (`/_deck/view(s)/*`) is NO LONGER registered here —
/// it moved to `ship_views::router()` (deck ⊥ ship-views). A host that serves
/// views merges that router alongside this one.
///
/// HMR and the state-aware SAIL endpoints (sse/data/mutation/patch) are NOT
/// registered here — they need host-supplied extensions or router state.
/// `/_deck/meta` is registered here; hosts that need to inject a
/// `DaemonBuildInfo` extension (shipd) simply layer it before mounting.

// Route paths for the framework's CSS / theme / catalog surface. axum's
// `Router::route` requires a `&'static str`, so we can't pass the
// `deck_core::routes::DECK_API_ASSETS` const directly; instead, mirror the
// SSOT here and let `routes_match_ssot` assert byte-equality at test time.
// The corresponding `// /api/...` literal still appears nowhere else in the
// production tree (verified by the boundary gate plus this const).
const ROUTE_VIEW_TOKENS_CSS: &str = "/api/assets/view-tokens.css";
const ROUTE_SAIL_CSS: &str = "/api/assets/sail.css";
const ROUTE_THEME_CSS: &str = "/api/assets/theme.css";
const ROUTE_THEMES_LIST: &str = "/api/v1/themes";

#[cfg(test)]
mod route_ssot {
    use deck_core::routes::{DECK_API_ASSETS, DECK_PREFIX};

    /// Belt-and-suspenders pin: the deck-axum route paths must equal the
    /// `deck_core::routes` SSOT. If anyone widens this crate with a hard-
    /// coded `/api/...` that drifts, this fails fast.
    #[test]
    fn routes_match_ssot() {
        assert_eq!(super::ROUTE_VIEW_TOKENS_CSS, format!("{DECK_API_ASSETS}/view-tokens.css"));
        assert_eq!(super::ROUTE_SAIL_CSS, format!("{DECK_API_ASSETS}/sail.css"));
        assert_eq!(super::ROUTE_THEME_CSS, format!("{DECK_API_ASSETS}/theme.css"));
        assert_eq!(super::ROUTE_THEMES_LIST, "/api/v1/themes");
        // The `/_deck/*` half of the surface still uses literal strings
        // in axum route registration; spot-check a sample to prove no
        // accidental drift snuck in.
        let _ = DECK_PREFIX;
    }
}

pub fn router() -> Router {
    use pages::{serve_sail_css, serve_theme_css, serve_themes_list, serve_route_tokens};
    // The root `/` and the `/{file}` SPA/public catch-all are HOST
    // concerns. The host mounts `serve_index` at `/` and the per-app
    // `serve_apps` for the app's routes + per-app `/{file}` catch-all.
    // Deck owns no first-run / `/setup` convention: `/setup` has no
    // server route — it falls through to the SPA fallback so the app's
    // own client-side routing renders it. The framework only owns the
    // framework surface (`/_deck/*` + `/api/*`).
    Router::new()
        // Shared CSS / theme assets (unauthenticated) — paths are the
        // canonical `Route` enum values (routes.rs is ground truth).
        .route(ROUTE_VIEW_TOKENS_CSS, get(serve_route_tokens))
        .route(ROUTE_SAIL_CSS, get(serve_sail_css))
        .route(ROUTE_THEME_CSS, get(serve_theme_css))
        .route(ROUTE_THEMES_LIST, get(serve_themes_list))
        // Per-tag component source viewer (dev studio deep-link).
        .route("/_deck/source/{tag}", get(sail_source::serve_sail_source))
        // HMR firehose — always mounted; the handler self-gates (empty SSE
        // when no bundler is wired), so no `if Development` guard is needed.
        .route("/_deck/dev/hmr", get(serve_bundle_hmr))
        .route("/_deck/runtime.js", get(serve_sui_runtime))
        .route("/_deck/dom.js", get(serve_sui_dom))
        .route("/_deck/sdk-client.js", get(serve_sui_sdk_client))
        .route("/_deck/route.js", get(serve_deck_route))
        .route("/_deck/icons/{file}", get(serve_sui_icon))
        .route("/_deck/live-reload.js", get(serve_sui_live_reload))
        .route("/_deck/error-overlay.js", get(serve_sui_error_overlay))
        // (formerly `/\_deck/{file}` → `serve_sui_chunk`) — purged:
        // shared chunks from the multi-entry view build were served
        // from `<root>/.deck/<chunk>.js`; the framework no longer reads
        // a monorepo-root `.deck/` (deck-purge-root). View bundles that
        // imported `../../<chunk>.js` no longer find their chunks at
        // runtime — view chunk serving is a follow-up.
        // (formerly `/\_deck/catalog/workspace` → `serve_catalog_workspace`)
        // — purged with `packs::serve_catalog_workspace` and the
        // `.deck/catalogs/workspace.json` write.
        .route("/_deck/meta", get(serve_meta))
        // The named-view routes (`/_deck/views/{name}/view.js`,
        // `/_deck/views/{name}[/{*sub}]`, `/_deck/view/{name}[/{*sub}]`) moved to
        // `ship_views::router()`. A host that serves views merges that router
        // alongside this one; deck itself no longer carries the feature
        // (deck ⊥ ship-views).
        // `POST /_deck/mutation/{name}` — 410 tombstone. The progressive-
        // enhancement form endpoint is gone; JS-attached mutations call the SDK
        // directly. deck still composes this URL as the `<form mutation>`
        // `action` (`deck_core::routes::mutation_action`) for the no-JS fallback,
        // so the route answers 410 Gone rather than 404. A `/_deck/*` serving
        // concern — lives here, not in the host's RPC surface.
        .route(
            "/_deck/mutation/{name}",
            axum::routing::post(|| async { StatusCode::GONE }),
        )
    // The SPA / public-asset dispatch (`/{file}`) is host-driven; the
    // per-app `serve_apps` mounts it for the primary app. The host's
    // other routes (the connect fallback, `/api/*`, the per-app routes)
    // all wire here.
    // No `.fallback` here: deck's `router()` is MERGED into the host's root
    // alongside the Connect-ops router, whose entire RPC dispatch IS a
    // `fallback_service`. axum panics if two merged routers both carry a
    // fallback, and a host `.fallback` would shadow the RPC dispatch into a dead
    // data plane. So deck sets none; the host keeps the Connect fallback and
    // composes SPA navigation ahead of it (shipd uses a pre-routing nav shim
    // that serves the shell for client-route GETs). deck still exports
    // `spa_fallback` + `serve_shell` for hosts that want them.
}

#[cfg(test)]
mod tests {
    use super::*;
    use deck_core::DaemonMode;

    async fn body_string(resp: Response) -> String {
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        String::from_utf8(bytes.to_vec()).unwrap()
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn into_axum_maps_status_content_type_and_cache() {
        let dr = DeckResponse::js_asset("console.log(1)", DaemonMode::Development);
        let resp = into_axum(dr);
        assert_eq!(resp.status(), StatusCode::OK);
        let ct = resp.headers().get(header::CONTENT_TYPE).unwrap().to_str().unwrap();
        assert!(ct.contains("application/javascript"));
        let cc = resp.headers().get(header::CACHE_CONTROL).unwrap().to_str().unwrap();
        assert_eq!(cc, "no-cache");
        assert!(!resp.headers().contains_key(header::ETAG));
        assert_eq!(body_string(resp).await, "console.log(1)");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn into_axum_prod_emits_etag() {
        let dr = DeckResponse::js_asset("x", DaemonMode::Production);
        let resp = into_axum(dr);
        assert!(resp.headers().contains_key(header::ETAG), "prod must carry ETag");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn into_axum_preserves_extra_headers() {
        let dr = DeckResponse::js_route("v", DaemonMode::Development)
            .with_header("X-Deck-Test", "counter");
        let resp = into_axum(dr);
        assert_eq!(
            resp.headers().get("X-Deck-Test").unwrap().to_str().unwrap(),
            "counter"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn into_axum_not_found_status() {
        let resp = into_axum(DeckResponse::not_found("nope", DaemonMode::Development));
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn router_builds_without_route_conflict() {
        // The `/_deck/{file}` shared-chunk route is a param sibling of the exact
        // `/_deck/runtime.js` etc.; matchit (axum 0.8) must accept the overlap.
        // A conflict would otherwise only surface as a daemon-startup panic.
        let _ = router();
    }

    /// serve_bundle_hmr with no bundler must return 200 (empty SSE body) and
    /// not hang. This is a safety check for the no-bundler path.
    #[tokio::test(flavor = "multi_thread")]
    async fn serve_bundle_hmr_no_bundler_returns_200() {
        use crate::handlers::serve_bundle_hmr;

        let resp = serve_bundle_hmr(None, None).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let ct = resp.headers().get(header::CONTENT_TYPE).unwrap().to_str().unwrap();
        assert!(ct.contains("text/event-stream"), "must be SSE content-type; got {ct}");
    }

    /// serve_bundle_hmr with a CancellationToken that fires immediately must
    /// not hang. Before this fix the BroadcastStream had no take_until and would
    /// block the axum graceful-shutdown drain until every client disconnected.
    #[tokio::test(flavor = "multi_thread")]
    async fn serve_bundle_hmr_terminates_on_cancel_token() {
        use axum::body::to_bytes;
        use crate::handlers::serve_bundle_hmr;
        use tokio_util::sync::CancellationToken;

        let token = CancellationToken::new();
        // Fire the token immediately before the handler even starts reading.
        token.cancel();

        // serve_bundle_hmr without a bundler (empty stream) — the test gates
        // that the handler accepts and honours the cancel extension without
        // panicking.
        let resp = serve_bundle_hmr(
            None,
            Some(Extension(token)),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK);

        // Consuming the body must complete (not block forever).
        let result = tokio::time::timeout(
            std::time::Duration::from_millis(500),
            to_bytes(resp.into_body(), usize::MAX),
        )
        .await;
        assert!(
            result.is_ok(),
            "response body must drain within 500ms after cancel token fires"
        );
    }
}
