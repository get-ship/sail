//! Thin axum shims. Each extracts axum params, calls the matching
//! deck-core function, and maps the resulting `DeckResponse` onto an
//! `axum::response::Response` via [`into_axum`](crate::into_axum). No
//! hosting logic lives here — it all sits in `deck-core`.

use axum::{
    Extension, Json,
    extract::Path,
    response::Response,
};
use serde_json::Value;

use deck_core::{DaemonBuildInfo, DaemonMode};

use crate::{into_axum, mode_from_ext};
// ── Framework bundle handlers ─────────────────────────────────────────────────

pub async fn serve_sui_runtime(mode: Option<Extension<DaemonMode>>) -> Response {
    into_axum(deck_core::serve_sui_runtime(mode_from_ext(mode)))
}

pub async fn serve_sui_dom(mode: Option<Extension<DaemonMode>>) -> Response {
    into_axum(deck_core::serve_sui_dom(mode_from_ext(mode)))
}

pub async fn serve_sui_sdk_client(mode: Option<Extension<DaemonMode>>) -> Response {
    into_axum(deck_core::serve_sui_sdk_client(mode_from_ext(mode)))
}

pub async fn serve_deck_route(mode: Option<Extension<DaemonMode>>) -> Response {
    into_axum(deck_core::serve_deck_route(mode_from_ext(mode)))
}


/// `GET /_deck/icons/{file}` — one lucide icon. `{file}` is captured whole
/// (e.g. `search.svg`) because axum 0.8 forbids mixing path-params with
/// literal suffixes in the same segment; we strip `.svg` here.
pub async fn serve_sui_icon(
    mode: Option<Extension<DaemonMode>>,
    Path(file): Path<String>,
) -> Response {
    let name = file.strip_suffix(".svg").unwrap_or(&file);
    into_axum(deck_core::serve_sui_icon(mode_from_ext(mode), name))
}

pub async fn serve_sui_live_reload(mode: Option<Extension<DaemonMode>>) -> Response {
    into_axum(deck_core::serve_sui_live_reload(mode_from_ext(mode)))
}

pub async fn serve_sui_error_overlay(mode: Option<Extension<DaemonMode>>) -> Response {
    into_axum(deck_core::serve_sui_error_overlay(mode_from_ext(mode)))
}

// (formerly `serve_sui_chunk` + `serve_catalog_workspace`) — purged
// with the runtime `.deck/` reads. The routes are gone too (deck-axum
// `router()`).

// ── View handlers — MOVED to the `ship-views` crate ────────────────────────────
// `serve_sui_route_setup`, `serve_route_host`, `serve_route_payload`, and
// `serve_view_payload_subpath` (the named-view serving surface) now live in
// `ship-views::handlers` and are registered by `ship_views::router()`. deck-axum
// keeps ONLY deck's own routes/ SPA + framework surface (deck ⊥ ship-views).

// ── Meta handler (/_deck/meta) ─────────────────────────────────────────────────

/// `GET /_deck/meta` — wraps deck-core's plain JSON value in `axum::Json`.
pub async fn serve_meta(daemon: Option<Extension<DaemonBuildInfo>>) -> Json<Value> {
    Json(deck_core::serve_meta(daemon.map(|e| e.0)))
}

// ── HMR (dev) handler ─────────────────────────────────────────────────────────

/// `GET /_deck/dev/hmr` — the bundler's event firehose as the SSE stream the
/// deck live-reload client subscribes to (`event: module-changed` /
/// `compile-error`, JSON payload). This is the one HMR handler every deck host
/// should mount — subscribing the bundler directly, with no per-host
/// reimplementation. Host-wired: it needs an `Extension<deck_bundler::Bundler>`;
/// without one it serves an empty, immediately-idle stream rather than erroring,
/// so the route is safe to register unconditionally.
///
/// The optional `Extension<CancellationToken>` (same pattern as shipd's
/// sui_hmr.rs) terminates the stream on shutdown so the host's graceful-drain
/// can complete without waiting for every client to disconnect (#256 class).
/// Without the extension the stream runs until client disconnect (existing
/// behaviour; tests don't need to wire a token).
pub async fn serve_bundle_hmr(
    bundler: Option<Extension<deck_bundler::Bundler>>,
    cancel: Option<Extension<tokio_util::sync::CancellationToken>>,
) -> Response {
    use axum::body::Body;
    use axum::http::{StatusCode, header};
    // futures_util::StreamExt used via fully-qualified path below to disambiguate
    // from tokio_stream::StreamExt — both provide filter_map and take_until.
    use tokio_stream::wrappers::BroadcastStream;

    let sse = || {
        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/event-stream")
            .header(header::CACHE_CONTROL, "no-cache")
    };

    let Some(Extension(bundler)) = bundler else {
        // No bundler wired → no events; serve a valid, empty SSE stream.
        return sse()
            .body(Body::empty())
            .unwrap_or_else(|_| Response::new(Body::empty()));
    };

    let events = futures_util::StreamExt::filter_map(
        BroadcastStream::new(bundler.subscribe_events()),
        |res| async move {
            let (name, payload) = match res.ok()? {
                deck_bundler::BundlerEvent::ModuleChanged { target, out_path, bytes } => (
                    "module-changed",
                    serde_json::json!({ "target": target, "out_path": out_path, "bytes": bytes }),
                ),
                deck_bundler::BundlerEvent::CompileError { target, message, line, col, file } => (
                    "compile-error",
                    serde_json::json!({
                        "target": target, "message": message,
                        "line": line, "col": col, "file": file,
                    }),
                ),
            };
            Some(Ok::<_, std::io::Error>(format!("event: {name}\ndata: {payload}\n\n")))
        },
    );

    // Apply take_until so SIGTERM (via the cancel token) terminates the stream.
    // When no token is provided the stream runs until client disconnect, which
    // is the correct behaviour for test hosts and non-shipd deployments.
    let cancel_token = cancel.map(|e| e.0).unwrap_or_default();
    let stream = futures_util::StreamExt::take_until(
        events,
        async move { cancel_token.cancelled().await },
    );

    sse()
        .body(Body::from_stream(stream))
        .unwrap_or_else(|_| Response::new(Body::from("sse build failed")))
}
