//! Per-window live-data subscription registry — the server side of the
//! live-data multiplex.
//!
//! One webview opens ONE stream and adds/removes logical reads via
//! `Subscribe`/`Unsubscribe`; this registry keys the per-window state on
//! `window_id` so the single stream fans out to N reads.
//!
//! The deck-axum streaming loop calls [`DataPort::resolve`](crate::data_port::DataPort::resolve)
//! per sub per tick — the registry stores pure transport state (no closures,
//! no host types). Ids are minted with the `ulid` crate (`win.<ulid>` /
//! `sub.<ulid>`) and the idle reaper timeout is sized off the heartbeat
//! constant in deck-axum.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;
use ulid::Ulid;

/// Opaque window id minted as `win.<ulid>`.
pub type WindowId = String;
/// Opaque subscription id minted as `sub.<ulid>`.
pub type SubscriptionId = String;

/// One logical live read inside a window. Pure transport state — the deck
/// streaming loop re-resolves `(domain, op, args)` via `DataPort::resolve`
/// on every change tick; the registry never stores a resolver closure.
#[derive(Clone)]
pub struct Subscription {
    pub id: SubscriptionId,
    pub domain: String,
    pub op: String,
    pub args: Value,
}

impl Subscription {
    /// Stable key for the subscription's `(domain, op, args)` triple — used
    /// by tests + by callers that want a content-addressable view of the
    /// bucket. Not used internally for dedup (the bucket is keyed on
    /// `subscription_id`).
    pub fn key(&self) -> String {
        format!("{}|{}|{}", self.domain, self.op, self.args)
    }
}

/// Per-window state: the live subscriptions + liveness for the idle reaper.
pub struct WindowBucket {
    pub window_id: WindowId,
    pub session_id: String,
    subs: Mutex<HashMap<SubscriptionId, Subscription>>,
    /// ms epoch of the last activity (open / subscribe / stream send); the
    /// reaper drops a bucket whose last activity is older than the idle timeout.
    last_active_ms: AtomicU64,
}

impl WindowBucket {
    fn new(window_id: WindowId, session_id: String) -> Self {
        Self {
            window_id,
            session_id,
            subs: Mutex::new(HashMap::new()),
            last_active_ms: AtomicU64::new(now_ms()),
        }
    }

    pub fn touch(&self) {
        self.last_active_ms.store(now_ms(), Ordering::Relaxed);
    }

    pub fn last_active_ms(&self) -> u64 {
        self.last_active_ms.load(Ordering::Relaxed)
    }

    pub fn subscription_count(&self) -> usize {
        self.subs.lock().unwrap_or_else(|e| e.into_inner()).len()
    }

    pub fn insert_sub(&self, sub: Subscription) {
        self.subs
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(sub.id.clone(), sub);
        self.touch();
    }

    pub fn remove_sub(&self, sub_id: &str) -> bool {
        let removed = self
            .subs
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(sub_id)
            .is_some();
        if removed {
            self.touch();
        }
        removed
    }

    /// Snapshot the current subscriptions for the stream bridge to re-resolve
    /// on a kernel event.
    pub fn subs(&self) -> Vec<Subscription> {
        self.subs
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .values()
            .cloned()
            .collect()
    }
}

/// Registry of all open windows, keyed on `window_id`.
pub struct WindowSubscriptionRegistry {
    windows: Mutex<HashMap<WindowId, Arc<WindowBucket>>>,
    idle_timeout_ms: u64,
}

impl Default for WindowSubscriptionRegistry {
    fn default() -> Self {
        // 35s = the default 15s heartbeat × 2 + 5s slack.
        Self::with_idle_timeout(35_000)
    }
}

impl WindowSubscriptionRegistry {
    pub fn with_idle_timeout(idle_timeout_ms: u64) -> Self {
        Self {
            windows: Mutex::new(HashMap::new()),
            idle_timeout_ms,
        }
    }

    /// Mint a fresh `win.<ulid>` and create its bucket bound to `session_id`.
    pub fn open(&self, session_id: String) -> WindowId {
        let window_id = format!("win.{}", Ulid::new());
        let bucket = Arc::new(WindowBucket::new(window_id.clone(), session_id));
        self.windows
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(window_id.clone(), bucket);
        window_id
    }

    pub fn bucket(&self, window_id: &str) -> Option<Arc<WindowBucket>> {
        self.windows
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(window_id)
            .cloned()
    }

    /// Add a subscription to `window_id`; returns the new `sub.<ulid>` id, or
    /// `None` if the window is unknown (client must `Open` first).
    pub fn subscribe(
        &self,
        window_id: &str,
        domain: String,
        op: String,
        args: Value,
    ) -> Option<SubscriptionId> {
        let bucket = self.bucket(window_id)?;
        let id = format!("sub.{}", Ulid::new());
        bucket.insert_sub(Subscription {
            id: id.clone(),
            domain,
            op,
            args,
        });
        Some(id)
    }

    pub fn unsubscribe(&self, window_id: &str, sub_id: &str) -> bool {
        self.bucket(window_id)
            .map(|b| b.remove_sub(sub_id))
            .unwrap_or(false)
    }

    /// Drop `window_id` entirely (window closed / stream gone / reaped).
    pub fn drop_window(&self, window_id: &str) -> bool {
        self.windows
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(window_id)
            .is_some()
    }

    pub fn window_count(&self) -> usize {
        self.windows.lock().unwrap_or_else(|e| e.into_inner()).len()
    }

    /// Drop every bucket idle longer than the idle timeout. Returns the count
    /// dropped. Called on a periodic sweep + as a safety net behind the
    /// stream-drop GC.
    pub fn reap(&self) -> usize {
        let cutoff = now_ms().saturating_sub(self.idle_timeout_ms);
        let mut windows = self.windows.lock().unwrap_or_else(|e| e.into_inner());
        let before = windows.len();
        windows.retain(|_, b| b.last_active_ms() >= cutoff);
        before - windows.len()
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_mints_unique_window_ids() {
        let reg = WindowSubscriptionRegistry::default();
        let a = reg.open("sess-a".into());
        let b = reg.open("sess-b".into());
        assert_ne!(a, b, "two opens must mint distinct window ids");
        assert!(a.starts_with("win."), "window id is win.<ulid>; got {a}");
        assert_eq!(reg.window_count(), 2);
        assert_eq!(reg.bucket(&a).unwrap().session_id, "sess-a");
    }

    #[test]
    fn subscribe_then_unsubscribe_mutates_the_bucket() {
        let reg = WindowSubscriptionRegistry::default();
        let w = reg.open("s".into());
        let sub = reg
            .subscribe(&w, "agents".into(), "summaries".into(), Value::Null)
            .expect("subscribe to a known window");
        assert!(sub.starts_with("sub."), "sub id is sub.<ulid>; got {sub}");
        assert_eq!(reg.bucket(&w).unwrap().subscription_count(), 1);
        assert!(reg.unsubscribe(&w, &sub), "unsubscribe an existing sub");
        assert_eq!(reg.bucket(&w).unwrap().subscription_count(), 0);
        assert!(!reg.unsubscribe(&w, &sub), "second unsubscribe is a no-op");
    }

    #[test]
    fn subscribe_to_unknown_window_is_none() {
        let reg = WindowSubscriptionRegistry::default();
        assert!(
            reg.subscribe("win.nope", "x".into(), "y".into(), Value::Null)
                .is_none(),
            "subscribe to an unopened window must return None"
        );
    }

    #[test]
    fn reap_drops_idle_buckets_keeps_active() {
        // 0ms idle timeout → any bucket not touched "now" is reaped; a freshly
        // touched bucket survives. Asserts the behavior (idle dropped, active
        // kept), not a wall-clock — no flakiness.
        let reg = WindowSubscriptionRegistry::with_idle_timeout(0);
        let stale = reg.open("stale".into());
        // Force the stale bucket's last-active into the past.
        reg.bucket(&stale)
            .unwrap()
            .last_active_ms
            .store(now_ms().saturating_sub(10_000), Ordering::Relaxed);
        let fresh = reg.open("fresh".into());
        reg.bucket(&fresh).unwrap().touch();

        let dropped = reg.reap();
        assert_eq!(dropped, 1, "exactly the stale bucket is reaped");
        assert!(reg.bucket(&stale).is_none(), "stale window gone");
        assert!(reg.bucket(&fresh).is_some(), "fresh window kept");
    }
}
