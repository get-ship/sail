//! A pool of `ScriptExecutor` VM threads — the production isolate shape.
//!
//! A single `ScriptExecutor` runs one isolate at a time on its dedicated VM
//! thread (a V8 isolate is thread-bound and not re-entrant across `.await`).
//! Concurrency therefore comes from MANY such threads — one live isolate per
//! thread, jobs distributed round-robin across them. This is the shape the
//! single-shot [`crate::run`] repeats once; the serve path
//! ([`crate::serve`]) drives the pool so concurrent requests don't serialize
//! on one isolate.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use serde_json::Value;
use ship_js::deno::ScriptExecutor;
use ship_js::Dispatch;

/// A fixed set of VM threads; jobs are dispatched round-robin. Cheap to clone
/// the inner executors are channel senders.
pub struct ScriptPool {
    execs: Vec<ScriptExecutor>,
    next: AtomicUsize,
}

impl ScriptPool {
    /// Spawn `size` VM threads (min 1). Each holds at most one live isolate.
    pub fn new(size: usize) -> Self {
        let size = size.max(1);
        let execs = (0..size).map(|_| ScriptExecutor::spawn()).collect();
        Self { execs, next: AtomicUsize::new(0) }
    }

    /// A pool sized to the host: one VM thread per core, less headroom for the
    /// async runtime + request threads (mirrors the executor's own cap).
    pub fn for_host() -> Self {
        let cores = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
        Self::new(cores.saturating_sub(2).max(1))
    }

    /// Number of VM threads in the pool.
    pub fn size(&self) -> usize {
        self.execs.len()
    }

    /// Run one module on the next VM thread. The returned future is `Send` (it
    /// only awaits a oneshot), so it composes inside any async host.
    pub async fn run(
        &self,
        dispatch: Arc<dyn Dispatch>,
        sdk: String,
        main: String,
    ) -> anyhow::Result<Vec<Value>> {
        let i = self.next.fetch_add(1, Ordering::Relaxed) % self.execs.len();
        self.execs[i].run(dispatch, sdk, main).await
    }
}
