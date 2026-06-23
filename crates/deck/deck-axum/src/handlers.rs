//! Thin axum shims. Each extracts axum params, calls the matching
//! deck-core function, and maps the resulting `DeckResponse` onto an
//! `axum::response::Response` via [`into_axum`](crate::into_axum). No
//! hosting logic lives here — it all sits in `deck-core`.

use axum::{
    Extension, Json,
    extract::{OriginalUri, Path, Query},
    response::Response,
};
use serde_json::Value;

use deck_core::{DaemonBuildInfo, DaemonMode, ViewSetupQuery};

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

// (formerly `serve_sui_chunk` + `serve_catalog_workspace`) — purged
// with the runtime `.deck/` reads. The routes are gone too (deck-axum
// `router()`).

// ── View handlers ─────────────────────────────────────────────────────────────

pub async fn serve_sui_view_setup(
    mode: Option<Extension<DaemonMode>>,
    bundler: Option<Extension<deck_bundler::Bundler>>,
    Path(name): Path<String>,
    Query(q): Query<ViewSetupQuery>,
) -> Response {
    let mode = mode_from_ext(mode);
    // Deck owns bundling (#167). If the view bundle isn't on disk yet, build it
    // in-process via Rolldown — the controller is inlined natively, no bun. The
    // daemon's watcher keeps `.deck` warm afterward; this covers the cold miss.
    let no_sub = q.sub.as_deref().filter(|s| !s.is_empty()).is_none();
    if no_sub
        && deck_core::resolve_view_js(&name).is_none()
        && let Some(Extension(b)) = bundler.as_ref()
    {
        // P0-2: when the in-process build FAILS (e.g. a broken paired
        // `controller.ts`), surface the REAL compile error instead of swallowing
        // it with `let _ =` and falling through to deck-core's generic "the
        // in-process bundler must build views/{name}/view" fallback, which masks
        // the actual syntax/type error. Only a `Bundle` failure (the view exists
        // but did not compile) short-circuits; `UnknownTarget`/`NoCompiler` fall
        // through so the resolve path yields the right 404 / diagnostic.
        if let Err(e) = b.build(&format!("views/{name}/view")).await
            && let deck_bundler::BundlerError::Bundle { .. } = &e
        {
            return into_axum(deck_core::DeckResponse::render_error(
                format!("view `{name}` failed to compile:\n{e}"),
                mode,
            ));
        }
    }
    into_axum(deck_core::serve_sui_view_setup(mode, &name, q.sub.as_deref()))
}

// ── View-host handler ─────────────────────────────────────────────────────────

pub async fn serve_view_host(
    mode: Option<Extension<DaemonMode>>,
    OriginalUri(uri): OriginalUri,
) -> Response {
    into_axum(deck_core::serve_view_host(mode_from_ext(mode), uri.path()))
}

// ── View-payload handlers ─────────────────────────────────────────────────────

pub async fn serve_view_payload(
    mode: Option<Extension<DaemonMode>>,
    Path(name): Path<String>,
) -> Response {
    into_axum(deck_core::serve_view_payload(mode_from_ext(mode), &name))
}

/// `GET /_deck/views/{name}/{*sub_path}` — the JSON payload for a
/// nested/parameterized view route. The wildcard path segment carries
/// one or more URL segments (e.g. `greet/zelda` or `studio/agents/42`);
/// the runtime folds the sub into the resolved `name` (and the
/// `__subscribe__` URL) and walks the resolver to capture `[param]`
/// segments. Mirrors the path-split pattern `serve_view_host` uses,
/// because axum's `Path<(String, String)>` only captures a fixed
/// 2-tuple and the deeper the nesting the more brittle that becomes.
pub async fn serve_view_payload_subpath(
    mode: Option<Extension<DaemonMode>>,
    OriginalUri(uri): OriginalUri,
) -> Response {
    // The sub-path payload surface is at `/_deck/views/{name}/{*sub_path}`.
    // Segment layout: ["_deck", "views", name, ...sub]. Name is at index 2;
    // the sub path is everything after the name, joined back with `/`.
    let mut segs = uri.path().trim_start_matches('/').split('/');
    let name = segs
        .nth(2)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    let sub_path: String = segs.collect::<Vec<_>>().join("/");
    match name {
        Some(name) => into_axum(deck_core::serve_view_payload_subpath(
            mode_from_ext(mode),
            &name,
            &sub_path,
        )),
        // Fallback: degenerate path that didn't match the 3-segment
        // `/_deck/views/<name>/<sub…>` shape — delegate to the top-level
        // payload handler, which produces the canonical 404 / name-error JSON.
        None => {
            let name = uri
                .path()
                .trim_start_matches('/')
                .rsplit('/')
                .next()
                .unwrap_or("");
            into_axum(deck_core::serve_view_payload(mode_from_ext(mode), name))
        }
    }
}

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
