//! Ops watch channel — SSE bus for model/file/log/task events.
//!
//! Endpoint: `GET /sail.ops.v1.watch` (SSE; simpler than WS on the existing axum stack).
//! Hosts (e.g. `sail dev` file watcher) call [`publish`] to fan out events.
//!
//! **Path convention:** `path` is workspace-relative with forward slashes
//! (e.g. `apps/sail-site/routes/page.sail`). Hosts must relativize absolute
//! notify paths before publishing (see [`relativize_watch_path`]).

use std::convert::Infallible;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use axum::extract::Query;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use futures_util::stream;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::sync::broadcast;

use crate::typed::Json;

const CAPACITY: usize = 256;

/// Event kind on the watch wire.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WatchKind {
    Model,
    File,
    Log,
    Task,
}

impl WatchKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Model => "model",
            Self::File => "file",
            Self::Log => "log",
            Self::Task => "task",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "model" => Some(Self::Model),
            "file" => Some(Self::File),
            "log" => Some(Self::Log),
            "task" => Some(Self::Task),
            _ => None,
        }
    }
}

/// Wire event: `{ kind, path, rev, payload }`.
///
/// `path` is workspace-relative (forward slashes). `payload` is a typed JSON blob.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchEvent {
    pub kind: WatchKind,
    pub path: String,
    pub rev: u64,
    pub payload: Json,
}

struct Bus {
    tx: broadcast::Sender<WatchEvent>,
    rev: AtomicU64,
    /// Last events for late subscribers / tests (ring, newest last).
    recent: Mutex<Vec<WatchEvent>>,
}

impl Bus {
    fn new() -> Self {
        let (tx, _) = broadcast::channel(CAPACITY);
        Self {
            tx,
            rev: AtomicU64::new(0),
            recent: Mutex::new(Vec::new()),
        }
    }

    fn publish(&self, kind: WatchKind, path: impl Into<String>, payload: Value) -> WatchEvent {
        let rev = self.rev.fetch_add(1, Ordering::SeqCst) + 1;
        let ev = WatchEvent {
            kind,
            path: path.into(),
            rev,
            payload: Json(payload),
        };
        if let Ok(mut g) = self.recent.lock() {
            g.push(ev.clone());
            if g.len() > 64 {
                let drain = g.len() - 64;
                g.drain(0..drain);
            }
        }
        let _ = self.tx.send(ev.clone());
        ev
    }
}

fn bus() -> &'static Bus {
    static BUS: OnceLock<Bus> = OnceLock::new();
    BUS.get_or_init(Bus::new)
}

/// Publish a watch event to all SSE subscribers.
pub fn publish(kind: WatchKind, path: impl Into<String>, payload: Value) -> WatchEvent {
    bus().publish(kind, path, payload)
}

/// Convenience: file change under a **workspace-relative** path.
pub fn publish_file(path: impl Into<String>) -> WatchEvent {
    let path = path.into().replace('\\', "/");
    publish(
        WatchKind::File,
        path.clone(),
        json!({ "op": "change", "path": path }),
    )
}

/// Relativize an absolute (or absolute-looking) path under `workspace_root`.
///
/// Returns forward-slash relative path when under root; otherwise returns the
/// path normalized with `/` (best-effort, no `..` collapse beyond strip prefix).
pub fn relativize_watch_path(workspace_root: &Path, path: &Path) -> String {
    let norm = |p: &Path| p.to_string_lossy().replace('\\', "/");
    // Prefer canonical comparison when both exist.
    let root_c = workspace_root
        .canonicalize()
        .unwrap_or_else(|_| workspace_root.to_path_buf());
    let path_c = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    if let Ok(rel) = path_c.strip_prefix(&root_c) {
        return norm(rel);
    }
    // Non-canonical strip (tests / not-yet-written files).
    let root_s = norm(workspace_root).trim_end_matches('/').to_string();
    let path_s = norm(path);
    if let Some(rest) = path_s.strip_prefix(&root_s) {
        return rest.trim_start_matches('/').to_string();
    }
    // Already relative
    if !path.is_absolute() {
        return path_s.trim_start_matches("./").to_string();
    }
    path_s
}

/// Publish a file event after relativizing under `workspace_root`.
pub fn publish_file_under(workspace_root: &Path, absolute_or_rel: &Path) -> WatchEvent {
    let rel = relativize_watch_path(workspace_root, absolute_or_rel);
    publish_file(rel)
}

/// Subscribe to the broadcast bus (for tests / non-HTTP consumers).
pub fn subscribe() -> broadcast::Receiver<WatchEvent> {
    bus().tx.subscribe()
}

/// Current revision counter.
pub fn current_rev() -> u64 {
    bus().rev.load(Ordering::SeqCst)
}

/// Recent events (test helper).
pub fn recent_events() -> Vec<WatchEvent> {
    bus().recent.lock().map(|g| g.clone()).unwrap_or_default()
}

#[derive(Debug, Deserialize)]
pub struct WatchQuery {
    /// Comma-separated kinds filter (optional).
    #[serde(default)]
    pub kinds: Option<String>,
    /// Path prefix filter (optional).
    #[serde(default, rename = "pathPrefix")]
    pub path_prefix: Option<String>,
}

fn parse_kinds(s: &Option<String>) -> Option<Vec<WatchKind>> {
    let s = s.as_ref()?.trim();
    if s.is_empty() {
        return None;
    }
    let kinds: Vec<WatchKind> = s.split(',').filter_map(WatchKind::parse).collect();
    if kinds.is_empty() { None } else { Some(kinds) }
}

/// `GET /sail.ops.v1.watch` — Server-Sent Events stream.
pub async fn watch_sse(Query(q): Query<WatchQuery>) -> Response {
    let kinds = parse_kinds(&q.kinds);
    let prefix = q
        .path_prefix
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    let rx = subscribe();
    let stream = stream::unfold(rx, move |mut rx| {
        let kinds = kinds.clone();
        let prefix = prefix.clone();
        async move {
            loop {
                match rx.recv().await {
                    Ok(ev) => {
                        if let Some(ref ks) = kinds {
                            if !ks.contains(&ev.kind) {
                                continue;
                            }
                        }
                        if let Some(ref p) = prefix {
                            if !ev.path.starts_with(p.as_str()) {
                                continue;
                            }
                        }
                        let data = serde_json::to_string(&ev).unwrap_or_else(|_| "{}".into());
                        let event = Event::default().data(data);
                        return Some((Ok::<_, Infallible>(event), rx));
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        // Synthetic gap signal so clients can resync.
                        let lag = WatchEvent {
                            kind: WatchKind::Log,
                            path: String::new(),
                            rev: current_rev(),
                            payload: Json(json!({ "op": "lagged", "n": n })),
                        };
                        let data = serde_json::to_string(&lag).unwrap_or_else(|_| "{}".into());
                        let event = Event::default().data(data);
                        return Some((Ok::<_, Infallible>(event), rx));
                    }
                    Err(broadcast::error::RecvError::Closed) => return None,
                }
            }
        }
    });

    Sse::new(stream)
        .keep_alive(
            KeepAlive::new()
                .interval(Duration::from_secs(15))
                .text("ping"),
        )
        .into_response()
}

/// Thin handle that always routes through the process-global bus (one rev space).
///
/// Local `subscribe()` still works for tests that want a hub-scoped receiver —
/// those get a fan-out copy of the same events.
#[derive(Clone, Default)]
pub struct WatchHub {
    /// Optional local fan-out for test isolation of subscribers.
    local_tx: Option<broadcast::Sender<WatchEvent>>,
}

impl WatchHub {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(CAPACITY);
        Self { local_tx: Some(tx) }
    }

    /// Publish via global [`publish`] so rev/recent/SSE stay coherent.
    pub fn publish(&self, kind: WatchKind, path: impl Into<String>, payload: Value) -> WatchEvent {
        let ev = publish(kind, path, payload);
        if let Some(ref tx) = self.local_tx {
            let _ = tx.send(ev.clone());
        }
        ev
    }

    pub fn subscribe(&self) -> broadcast::Receiver<WatchEvent> {
        if let Some(ref tx) = self.local_tx {
            tx.subscribe()
        } else {
            subscribe()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::Instant;

    #[tokio::test]
    async fn publish_file_received_within_500ms() {
        let mut rx = subscribe();
        let path = format!("routes/test-{}.sail", current_rev());
        let t0 = Instant::now();
        let sent = publish_file(&path);
        let got = tokio::time::timeout(Duration::from_millis(500), rx.recv())
            .await
            .expect("timeout waiting for watch event")
            .expect("recv");
        assert!(t0.elapsed() < Duration::from_millis(500));
        assert_eq!(got.kind, WatchKind::File);
        assert_eq!(got.path, path);
        assert_eq!(got.rev, sent.rev);
    }

    #[test]
    fn kind_roundtrip() {
        for k in [
            WatchKind::Model,
            WatchKind::File,
            WatchKind::Log,
            WatchKind::Task,
        ] {
            assert_eq!(WatchKind::parse(k.as_str()), Some(k));
        }
    }

    #[test]
    fn relativize_under_workspace() {
        let root = PathBuf::from("/ws/root");
        let abs = PathBuf::from("/ws/root/apps/demo/routes/page.sail");
        assert_eq!(
            relativize_watch_path(&root, &abs),
            "apps/demo/routes/page.sail"
        );
        assert_eq!(
            relativize_watch_path(&root, Path::new("routes/x.sail")),
            "routes/x.sail"
        );
    }

    #[test]
    fn relativize_prefix_filter_friendly() {
        let root = PathBuf::from("/home/u/proj");
        let abs = root.join("routes/foo.sail");
        let rel = relativize_watch_path(&root, &abs);
        assert!(rel.starts_with("routes/"), "got {rel}");
        assert!(!rel.starts_with('/'));
    }

    #[test]
    fn hub_publish_uses_global_rev() {
        let hub = WatchHub::new();
        let before = current_rev();
        let ev = hub.publish(WatchKind::Log, "x", json!({"ok": true}));
        assert!(ev.rev > before);
        assert_eq!(current_rev(), ev.rev);
    }
}
