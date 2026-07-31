//! The live-data multiplex transport — one SSE stream per webview that fans
//! out to N logical reads keyed on `subscription_id`.
//!
//! Mirrors `serve_bundle_hmr` in shape (host-supplied `Extension`s + an
//! `axum::Router` for callers to merge): the streaming loop calls
//! `DataPort::resolve` for every sub on every change tick and yields
//! `mux-data` / `mux-heartbeat` / `mux-close` SSE frames (see
//! `deck_core::live_wire::format_mux_*`). The transport is deck-owned —
//! `domain`/`op`/`args` are opaque to deck; the host binds its own state
//! (e.g. shipd's `ApiState`) inside the impl.
//!
//! Leak-fidelity (replaces the daemon-leak fix #256/#191/#193):
//!   * `WindowDropGuard` on stream end drops the bucket from the registry.
//!   * The host's `ChangeFeed::Drop` tears down the spawned kernel actor
//!     (e.g. shipd's `ActorGuard` analogue in `ShipChangeSource`).
//!   * The registry's idle reaper (35s = 2× heartbeat + slack) is the safety
//!     net behind the drop guard.
//!
use std::sync::Arc;
use std::time::Duration;

use axum::{
    Extension, Json, Router,
    body::Body,
    extract::Path,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::time::interval;
use tokio_stream::wrappers::ReceiverStream;

use deck_core::{
    ChangeFeed, ChangeSource, DataPort, WindowId, WindowSubscriptionRegistry, format_mux_close,
    format_mux_data, format_mux_heartbeat,
};

/// Server heartbeat interval — the cadence of the top-level `mux-heartbeat`
/// frames on the stream. The registry's idle reaper timeout is sized off
/// this (35s = 2× + slack, see `WindowSubscriptionRegistry::default`).
pub const HEARTBEAT_MS: u32 = 15_000;

// ── State carried by Extension ────────────────────────────────────────────────

/// One open multiplex stream. Held in a tower `Extension` so the host (e.g.
/// shipd) constructs it at router-mount time with the live
/// `DataPort` + `WindowSubscriptionRegistry` + `ChangeSource` impls.
#[derive(Clone)]
pub struct MultiplexState {
    pub data_port: Arc<dyn DataPort>,
    pub registry: Arc<WindowSubscriptionRegistry>,
    pub change_source: Arc<dyn ChangeSource>,
}

impl MultiplexState {
    pub fn new(
        data_port: Arc<dyn DataPort>,
        registry: Arc<WindowSubscriptionRegistry>,
        change_source: Arc<dyn ChangeSource>,
    ) -> Self {
        Self {
            data_port,
            registry,
            change_source,
        }
    }
}

// ── Control plane (POST JSON) ─────────────────────────────────────────────────

#[derive(Serialize)]
struct OpenResponseBody {
    window_id: WindowId,
    session_id: String,
    heartbeat_ms: u32,
}

/// `POST /_deck/mux/open` — mint a fresh `win.<ulid>` and bind a session.
async fn mux_open(Extension(state): Extension<MultiplexState>) -> Json<OpenResponseBody> {
    let session_id = format!("sess.{}", ulid::Ulid::new());
    let window_id = state.registry.open(session_id.clone());
    Json(OpenResponseBody {
        window_id,
        session_id,
        heartbeat_ms: HEARTBEAT_MS,
    })
}

#[derive(Deserialize)]
struct SubscribeRequest {
    window_id: String,
    domain: String,
    op: String,
    #[serde(default)]
    args: Value,
}

#[derive(Serialize)]
struct SubscribeResponseBody {
    subscription_id: String,
    initial_value: Value,
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

fn err(status: StatusCode, msg: impl Into<String>) -> Response {
    (status, Json(ErrorBody { error: msg.into() })).into_response()
}

/// `POST /_deck/mux/subscribe` — register a `(domain, op, args)` live read
/// against an open window. Returns the initial resolved value + the
/// `sub.<ulid>` id the client demuxes on. **Explicit errors only** — never
/// a silent `null` initial value when the resolver is missing or the
/// window is unknown (ADR 0001 / live_resolvers `resolve_fn` contract).
async fn mux_subscribe(
    Extension(state): Extension<MultiplexState>,
    Json(req): Json<SubscribeRequest>,
) -> Response {
    let initial = match state.data_port.resolve(&req.domain, &req.op, &req.args) {
        Some(v) => v,
        None => {
            return err(
                StatusCode::NOT_FOUND,
                format!("multiplex has no resolver for {}.{}", req.domain, req.op),
            );
        }
    };
    let subscription_id =
        match state
            .registry
            .subscribe(&req.window_id, req.domain, req.op, req.args)
        {
            Some(id) => id,
            None => {
                return err(
                    StatusCode::NOT_FOUND,
                    format!("unknown window '{}' — open first", req.window_id),
                );
            }
        };
    (
        StatusCode::OK,
        Json(SubscribeResponseBody {
            subscription_id,
            initial_value: initial,
        }),
    )
        .into_response()
}

#[derive(Deserialize)]
struct UnsubscribeRequest {
    window_id: String,
    subscription_id: String,
}

/// `POST /_deck/mux/unsubscribe` — drop a subscription from a window.
async fn mux_unsubscribe(
    Extension(state): Extension<MultiplexState>,
    Json(req): Json<UnsubscribeRequest>,
) -> Response {
    let removed = state
        .registry
        .unsubscribe(&req.window_id, &req.subscription_id);
    (
        StatusCode::OK,
        Json(serde_json::json!({ "removed": removed })),
    )
        .into_response()
}

// ── Data plane (SSE) ──────────────────────────────────────────────────────────

/// RAII GC: drops the bucket on stream end. The registry's idle reaper is
/// the backstop for windows that never get a clean stream close.
struct WindowDropGuard {
    registry: Arc<WindowSubscriptionRegistry>,
    window_id: String,
}

impl Drop for WindowDropGuard {
    fn drop(&mut self) {
        self.registry.drop_window(&self.window_id);
    }
}

/// `GET /_deck/mux/stream/{window_id}` — the one per-window server-stream.
/// Carries the initial value for every sub in the window, then heartbeats,
/// then re-emits every sub on every change tick from the host's
/// `ChangeSource`. Demuxed by `subscription_id` on the client side.
async fn mux_stream(
    Extension(state): Extension<MultiplexState>,
    Path(window_id): Path<String>,
) -> Response {
    let bucket = match state.registry.bucket(&window_id) {
        Some(b) => b,
        None => {
            return err(
                StatusCode::NOT_FOUND,
                format!("unknown window '{window_id}' — open first"),
            );
        }
    };

    let feed = match state.change_source.open_feed().await {
        Ok(f) => f,
        Err(e) => return err(StatusCode::INTERNAL_SERVER_ERROR, e),
    };

    // The streaming loop emits raw SSE frame strings; we then map them
    // through an mpsc so axum's `Body::from_stream` can pull at its own
    // pace. The producer future is held in a JoinHandle; when the response
    // body is dropped (client disconnect / cancel), we abort the producer.
    let (tx, rx) = tokio::sync::mpsc::channel::<Result<String, std::io::Error>>(16);

    let producer = stream_producer(
        bucket,
        state.registry.clone(),
        window_id.clone(),
        state.data_port.clone(),
        feed,
        tx.clone(),
    );
    let drain = tokio::spawn(producer);

    let stream = ReceiverStream::new(rx);
    let body = Body::from_stream(async_stream::stream! {
        let mut s = Box::pin(stream);
        while let Some(item) = s.next().await {
            yield item;
        }
        // Body finished — producer no longer needed. Abort unblocks the
        // drain task if the client disconnected mid-stream.
        drain.abort();
    });

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/event-stream")
        .header(header::CACHE_CONTROL, "no-cache")
        .body(body)
        .unwrap_or_else(|_| Response::new(Body::empty()))
}

async fn stream_producer(
    bucket: Arc<deck_core::WindowBucket>,
    registry: Arc<WindowSubscriptionRegistry>,
    window_id: String,
    data_port: Arc<dyn DataPort>,
    mut feed: Box<dyn ChangeFeed>,
    tx: tokio::sync::mpsc::Sender<Result<String, std::io::Error>>,
) {
    // WindowDropGuard — RAII GC: drops the bucket on stream end.
    let _drop_guard = WindowDropGuard {
        registry,
        window_id,
    };
    let mut version: u32 = 0;
    let mut hb = interval(Duration::from_millis(HEARTBEAT_MS as u64));
    hb.tick().await; // skip the immediate first tick

    // Initial fan-out: emit the current value of every sub in the window
    // at stream start. Subs added after this get their first value from
    // the unary SubscribeAck.
    bucket.touch();
    for sub in bucket.subs() {
        match data_port.resolve(&sub.domain, &sub.op, &sub.args) {
            Some(value) => {
                version += 1;
                if tx
                    .send(Ok(format_mux_data(&sub.id, &value, version)))
                    .await
                    .is_err()
                {
                    return;
                }
            }
            None => {
                // Resolver disappeared between Subscribe and Stream — surface
                // as an explicit close, not a silent skip.
                let msg = format!("no resolver for {}.{}", sub.domain, sub.op);
                let _ = tx.send(Ok(format_mux_close("resolver_gone", &msg))).await;
                return;
            }
        }
    }

    loop {
        tokio::select! {
            ev = feed.recv() => {
                let Some(_ev) = ev else {
                    // Feed ended — host tore down its source. Close the stream.
                    let _ = tx
                        .send(Ok(format_mux_close("feed_ended", "change feed ended")))
                        .await;
                    break;
                };
                // Coalesce a burst of ticks into one re-resolve sweep
                // (mirrors the burst-coalesce in the old Connect wire).
                bucket.touch();
                for sub in bucket.subs() {
                    if let Some(value) = data_port.resolve(&sub.domain, &sub.op, &sub.args) {
                        version += 1;
                        if tx.send(Ok(format_mux_data(&sub.id, &value, version))).await.is_err() {
                            return;
                        }
                    }
                }
            }
            _ = hb.tick() => {
                bucket.touch();
                if tx.send(Ok(format_mux_heartbeat().to_string())).await.is_err() {
                    return;
                }
            }
        }
    }
}

// ── Router ────────────────────────────────────────────────────────────────────

/// Mount the multiplex transport. Returns a `Router` the host merges into
/// ITS authenticated router group — same posture as shipd's Connect wires
/// (the multiplex is gated; see `apps/shipd/src/router.rs`).
///
/// The host injects ONE type via `Extension`:
///   * `Extension(MultiplexState::new(data_port, registry, change_source))`
///
/// All three impls are folded into MultiplexState so the host's mount step
/// is a single `Extension` layer + `.merge(deck_axum::multiplex_router())`.
pub fn multiplex_router() -> Router {
    Router::new()
        .route("/_deck/mux/open", post(mux_open))
        .route("/_deck/mux/subscribe", post(mux_subscribe))
        .route("/_deck/mux/unsubscribe", post(mux_unsubscribe))
        .route("/_deck/mux/stream/{window_id}", get(mux_stream))
}
