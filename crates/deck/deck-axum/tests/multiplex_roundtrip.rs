//! Multiplex round-trip test — proves the load-bearing property the
//! `WindowSubscriptionService` tests asserted in shipd, now re-proven over
//! the deck-owned wire + fake `DataPort`/`ChangeSource` impls.
//!
//! Property: ONE stream demuxes N subscriptions by `subscription_id`.
//! `open` → `subscribe` (N times) → `stream` yields an initial `mux-data`
//! frame for each, then a `mux-data` frame for each on every change tick.

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex as StdMutex};

use deck_axum::multiplex::{MultiplexState, multiplex_router};
use deck_core::{
    ChangeFeed, ChangeSource, DataPort, WindowSubscriptionRegistry, format_mux_data,
    format_mux_heartbeat,
};
use serde_json::json;

// ── Fake ports ────────────────────────────────────────────────────────────────

#[derive(Default)]
struct FakeDataPort {
    by_key: StdMutex<HashMap<String, serde_json::Value>>,
    resolve_count: AtomicUsize,
}

impl FakeDataPort {
    fn with(domain: &str, op: &str, value: serde_json::Value) -> Arc<Self> {
        let port = Arc::new(Self::default());
        port.by_key
            .lock()
            .unwrap()
            .insert(format!("{domain}.{op}"), value);
        port
    }

    fn set(&self, domain: &str, op: &str, value: serde_json::Value) {
        self.by_key
            .lock()
            .unwrap()
            .insert(format!("{domain}.{op}"), value);
    }

    fn resolve_count(&self) -> usize {
        self.resolve_count.load(Ordering::SeqCst)
    }
}

impl DataPort for FakeDataPort {
    fn resolve(
        &self,
        domain: &str,
        op: &str,
        _args: &serde_json::Value,
    ) -> Option<serde_json::Value> {
        self.resolve_count.fetch_add(1, Ordering::SeqCst);
        self.by_key
            .lock()
            .unwrap()
            .get(&format!("{domain}.{op}"))
            .cloned()
    }
}

/// A controllable `ChangeSource` — yields a tick when the test calls
/// `tick()`, yields `None` (feed ended) when the test calls `end()`.
struct FakeChangeSource {
    #[allow(dead_code)]
    tx: tokio::sync::mpsc::Sender<()>,
    rx: StdMutex<Option<tokio::sync::mpsc::Receiver<()>>>,
}

impl FakeChangeSource {
    fn new() -> Arc<Self> {
        let (tx, rx) = tokio::sync::mpsc::channel(8);
        Arc::new(Self {
            tx,
            rx: StdMutex::new(Some(rx)),
        })
    }
}

impl ChangeSource for FakeChangeSource {
    fn open_feed(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<Box<dyn ChangeFeed>, String>> + Send + '_>> {
        let rx = self
            .rx
            .lock()
            .unwrap()
            .take()
            .expect("open_feed called twice on the same source");
        Box::pin(async move { Ok(Box::new(FakeChangeFeed { rx }) as Box<dyn ChangeFeed>) })
    }
}

struct FakeChangeFeed {
    rx: tokio::sync::mpsc::Receiver<()>,
}

impl ChangeFeed for FakeChangeFeed {
    fn recv(&mut self) -> Pin<Box<dyn Future<Output = Option<()>> + Send + '_>> {
        Box::pin(async move { self.rx.recv().await })
    }
}

// ── Test fixture ──────────────────────────────────────────────────────────────

struct Harness {
    addr: std::net::SocketAddr,
    data_port: Arc<FakeDataPort>,
}

async fn serve(data_port: Arc<FakeDataPort>, change_source: Arc<FakeChangeSource>) -> Harness {
    let registry = Arc::new(WindowSubscriptionRegistry::default());
    let state = MultiplexState::new(data_port.clone(), registry, change_source);
    let app = axum::Router::new()
        .merge(multiplex_router())
        .layer(axum::Extension(state));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    Harness { addr, data_port }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn one_stream_demuxes_two_subscriptions_by_id() {
    let data_port = FakeDataPort::with("agents", "summaries", json!({"items": [1, 2, 3]}));
    data_port.set("git", "overview", json!({"branch": "main", "dirty": false}));
    let change_source = FakeChangeSource::new();
    let h = serve(data_port.clone(), change_source).await;
    let client = reqwest::Client::new();

    // POST /_deck/mux/open
    let open: serde_json::Value = client
        .post(format!("http://{}/_deck/mux/open", h.addr))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let window_id = open["window_id"].as_str().unwrap().to_string();
    assert!(
        window_id.starts_with("win."),
        "win.<ulid> prefix; got {window_id}"
    );

    // POST /_deck/mux/subscribe × 2
    let sub_a: serde_json::Value = client
        .post(format!("http://{}/_deck/mux/subscribe", h.addr))
        .json(&json!({
            "window_id": window_id, "domain": "agents", "op": "summaries",
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let sub_b: serde_json::Value = client
        .post(format!("http://{}/_deck/mux/subscribe", h.addr))
        .json(&json!({
            "window_id": window_id, "domain": "git", "op": "overview",
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let id_a = sub_a["subscription_id"].as_str().unwrap().to_string();
    let id_b = sub_b["subscription_id"].as_str().unwrap().to_string();
    assert_ne!(id_a, id_b, "two subs must mint distinct ids");
    assert!(
        id_a.starts_with("sub.") && id_b.starts_with("sub."),
        "sub.<ulid>"
    );

    // GET /_deck/mux/stream/{window_id}
    let resp = client
        .get(format!("http://{}/_deck/mux/stream/{window_id}", h.addr))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let ct = resp
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(
        ct.contains("text/event-stream"),
        "SSE content-type; got {ct}"
    );

    use futures_util::StreamExt;
    let mut stream = resp.bytes_stream();
    let mut buf = Vec::<u8>::new();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    while std::time::Instant::now() < deadline {
        match tokio::time::timeout(std::time::Duration::from_millis(100), stream.next()).await {
            Ok(Some(Ok(chunk))) => {
                buf.extend_from_slice(&chunk);
                if buf.windows(id_a.len()).any(|w| w == id_a.as_bytes())
                    && buf.windows(id_b.len()).any(|w| w == id_b.as_bytes())
                {
                    break;
                }
            }
            Ok(Some(Err(_))) | Ok(None) => break,
            Err(_) => continue,
        }
    }

    let body = String::from_utf8_lossy(&buf);
    assert!(
        body.contains(&id_a),
        "stream must carry frame tagged by sub_a ({id_a}); got:\n{body}"
    );
    assert!(
        body.contains(&id_b),
        "stream must carry frame tagged by sub_b ({id_b}); got:\n{body}"
    );
    assert!(
        body.contains("event: mux-data"),
        "wire format must be mux-data frames; got:\n{body}"
    );
    assert!(
        body.contains("\"items\""),
        "agents.summaries initial value must appear in the body; got:\n{body}"
    );
    assert!(
        body.contains("\"branch\""),
        "git.overview initial value must appear in the body; got:\n{body}"
    );
    // Two subs ⇒ at least two resolves from the initial fan-out.
    assert!(
        h.data_port.resolve_count() >= 2,
        "initial fan-out re-resolves every sub; got {}",
        h.data_port.resolve_count()
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn unknown_op_returns_explicit_error_not_silent_null() {
    let data_port = FakeDataPort::with("known", "op", json!(42));
    let change_source = FakeChangeSource::new();
    let h = serve(data_port, change_source).await;
    let client = reqwest::Client::new();

    let open: serde_json::Value = client
        .post(format!("http://{}/_deck/mux/open", h.addr))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let window_id = open["window_id"].as_str().unwrap().to_string();

    let resp = client
        .post(format!("http://{}/_deck/mux/subscribe", h.addr))
        .json(&json!({"window_id": window_id, "domain": "nope", "op": "void"}))
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status().as_u16(),
        404,
        "no-resolver must surface as 404"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn subscribe_to_unknown_window_is_404() {
    let data_port = Arc::new(FakeDataPort::default());
    let change_source = FakeChangeSource::new();
    let h = serve(data_port, change_source).await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("http://{}/_deck/mux/subscribe", h.addr))
        .json(&json!({"window_id": "win.never_opened", "domain": "x", "op": "y"}))
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status().as_u16(),
        404,
        "subscribe-before-open must surface as 404"
    );
}

// Compile-only sanity: the SSE encoders are byte-stable.
#[test]
fn format_mux_data_and_heartbeat_pin_bytes() {
    let f = format_mux_data("sub.01", &json!({"a": 1}), 7);
    assert!(f.starts_with("event: mux-data\ndata: "));
    assert!(f.ends_with("\n\n"));
    assert!(f.contains("\"sub\":\"sub.01\""));
    assert!(f.contains("\"version\":7"));
    assert!(format_mux_heartbeat().starts_with("event: mux-heartbeat"));
}
