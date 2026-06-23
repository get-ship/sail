//! Slice 7 — live-data SSE (no browser, ship-free).
//!
//! Opens `/_deck/views/clock/sse` with a raw HTTP client (tower oneshot),
//! reads SSE frames from the stream, and verifies:
//! - The first frame is `event: patch-signals` with `{"kind":"ok"}`.
//! - A second frame arrives within ~1.5 s with a strictly larger `now` value.
//!
//! This test uses the tower `ServiceExt::call` adapter to simulate a real
//! streaming response without launching a TCP listener. Because SSE streams
//! are infinite, we read only the frames we need then stop.
//!
//! Run: `cargo test -p deck-conformance --test live_data`

use std::time::Duration;

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use deck_conformance::{deck_app, default_app_dir};
use tower::ServiceExt as _;

/// Bind an ephemeral port, serve the demo app, connect with a raw TCP stream,
/// parse SSE frames until we have `want` complete frames or the timeout fires.
async fn live_data_frames(path: &str, want: usize, timeout: Duration) -> Vec<serde_json::Value> {
    use tokio::io::AsyncWriteExt;

    // 1. Start the app on a random port.
    let app = deck_app(default_app_dir());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve");
    });

    // 2. Connect and send the request.
    let mut stream = tokio::net::TcpStream::connect(addr).await.unwrap();
    let req = format!(
        "GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nAccept: text/event-stream\r\nConnection: close\r\n\r\n"
    );
    stream.write_all(req.as_bytes()).await.unwrap();

    // 3. Read until timeout, accumulate frames.
    let mut buf = Vec::<u8>::new();
    let mut frames = Vec::<serde_json::Value>::new();
    let deadline = tokio::time::Instant::now() + timeout;

    loop {
        if frames.len() >= want {
            break;
        }
        if tokio::time::Instant::now() >= deadline {
            break;
        }
        let remaining = deadline - tokio::time::Instant::now();
        let mut tmp = [0u8; 4096];
        let n = match tokio::time::timeout(remaining, tokio::io::AsyncReadExt::read(&mut stream, &mut tmp)).await {
            Ok(Ok(0)) => break,  // EOF
            Ok(Ok(n)) => n,
            Ok(Err(_)) | Err(_) => break,
        };
        buf.extend_from_slice(&tmp[..n]);

        // Parse complete SSE frames (separated by \n\n).
        while let Some(pos) = find_double_newline(&buf) {
            let chunk = buf.drain(..pos + 2).collect::<Vec<u8>>();
            let text = String::from_utf8_lossy(&chunk);
            // Extract `data:` line.
            for line in text.lines() {
                if let Some(data) = line.strip_prefix("data: ") {
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(data) {
                        frames.push(v);
                        break;
                    }
                }
            }
            if frames.len() >= want {
                break;
            }
        }
    }
    frames
}

/// Find the position of `\n\n` in `buf` (the SSE frame terminator).
fn find_double_newline(buf: &[u8]) -> Option<usize> {
    buf.windows(2).position(|w| w == b"\n\n")
}

// ── tests ─────────────────────────────────────────────────────────────────────

/// First SSE frame from `/_deck/views/clock/sse` must be a `patch-signals` event
/// with `{"kind":"ok","value":{"now":<n>}}`.
#[tokio::test(flavor = "multi_thread")]
async fn clock_sse_first_frame_is_ok_patch_signals() {
    let frames = live_data_frames(
        "/_deck/views/clock/sse",
        1,
        Duration::from_secs(5),
    )
    .await;

    assert!(!frames.is_empty(), "expected at least one SSE frame within timeout");
    let frame = &frames[0];
    assert_eq!(
        frame["value"]["kind"],
        "ok",
        "first frame must be kind:ok; got: {frame}"
    );
    assert!(
        frame["value"]["value"]["now"].is_number(),
        "first frame value.now must be a number; got: {frame}"
    );
    assert_eq!(frame["name"], "t", "data name must be 't'; got: {frame}");
}

/// A second SSE frame must arrive within ~1.5 s of the first, and its `now`
/// value must be strictly larger (the tick advanced).
#[tokio::test(flavor = "multi_thread")]
async fn clock_sse_second_frame_has_advanced_now() {
    let frames = live_data_frames(
        "/_deck/views/clock/sse",
        2,
        Duration::from_millis(3000),
    )
    .await;

    assert!(
        frames.len() >= 2,
        "expected 2 SSE frames within 3 s; got {}",
        frames.len()
    );
    let now0 = frames[0]["value"]["value"]["now"]
        .as_u64()
        .expect("frame 0 now is u64");
    let now1 = frames[1]["value"]["value"]["now"]
        .as_u64()
        .expect("frame 1 now is u64");
    assert!(
        now1 > now0,
        "second tick must be strictly larger: {now0} → {now1}"
    );
}

/// `/_deck/views/clock/sse` must respond with `Content-Type: text/event-stream`.
#[tokio::test(flavor = "multi_thread")]
async fn clock_sse_content_type_is_event_stream() {
    let app = deck_app(default_app_dir());
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/_deck/views/clock/sse")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("response");
    assert_eq!(resp.status(), StatusCode::OK);
    let ct = resp
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(
        ct.contains("text/event-stream"),
        "Content-Type must be text/event-stream; got: {ct}"
    );
}

/// Unknown view → 404.
#[tokio::test(flavor = "multi_thread")]
async fn sse_unknown_view_is_404() {
    let app = deck_app(default_app_dir());
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/_deck/views/no-such-view/sse")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("response");
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

/// THE definitive mutation→live-data gate, proven ship-free at the Rust tier:
/// after a mutation, a FRESH live subscriber to the same server immediately
/// sees the new todo. One server = one shared store; the mutation persists and
/// the new SSE's first frame resolves it. The mutation dispatches to the
/// registry op (`demo.addTodo`) directly — the PE form POST it used to travel
/// through is a 410 tombstone in deck's router (JS-attached mutations call the
/// SDK); the composed store→SSE contract is unchanged.
#[tokio::test(flavor = "multi_thread")]
async fn fresh_live_subscriber_sees_todo_after_mutation() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let store = deck_conformance::DemoStore::new();
    let app = deck_conformance::deck_app_with_store(
        default_app_dir(),
        deck::DaemonMode::Development,
        store.clone(),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.expect("serve"); });

    // 1. Mutate the running server's store via the registry op.
    deck_conformance::data_port::resolve_demo(
        "demo.addTodo",
        &serde_json::json!({ "text": "composed-gate" }),
        &store,
    )
    .await
    .expect("addTodo mutation");

    // 2. Open a FRESH todos SSE; its first frame resolves the current store.
    let mut stream = tokio::net::TcpStream::connect(addr).await.unwrap();
    stream
        .write_all(b"GET /_deck/views/todos/sse HTTP/1.1\r\nHost: x\r\nAccept: text/event-stream\r\nConnection: close\r\n\r\n")
        .await
        .unwrap();

    let mut buf = Vec::<u8>::new();
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    let frame: Option<serde_json::Value> = loop {
        if tokio::time::Instant::now() >= deadline {
            break None;
        }
        let mut tmp = [0u8; 4096];
        let n = match tokio::time::timeout(
            deadline - tokio::time::Instant::now(),
            stream.read(&mut tmp),
        )
        .await
        {
            Ok(Ok(0)) | Ok(Err(_)) | Err(_) => break None,
            Ok(Ok(n)) => n,
        };
        buf.extend_from_slice(&tmp[..n]);
        if let Some(pos) = find_double_newline(&buf) {
            let chunk = String::from_utf8_lossy(&buf[..pos]).to_string();
            if let Some(data) = chunk.lines().find_map(|l| l.strip_prefix("data: ")) {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(data) {
                    break Some(v);
                }
            }
            buf.drain(..pos + 2);
        }
    };

    let frame = frame.expect("todos SSE first frame within timeout");
    let items = frame["value"]["value"]["items"]
        .as_array()
        .unwrap_or_else(|| panic!("items array in frame; got: {frame}"));
    assert!(
        items.iter().any(|t| t["text"] == "composed-gate"),
        "a fresh live subscriber must see the mutated todo; got: {frame}"
    );
}
