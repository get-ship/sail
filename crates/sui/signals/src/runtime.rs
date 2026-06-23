//! Thread-local reactive tracking state.
//!
//! All reactive primitives (signal reads, effect runs, derived recomputes)
//! are synchronous. Thread-locals are correct here: the tracking scope never
//! crosses an await point; each thread maintains its own observer stack.

use std::cell::{Cell, RefCell};
use std::sync::Arc;

/// A subscriber callback held alive by the owner (Effect/Derived).
/// Signals hold `Weak<dyn Fn()>` references to avoid keeping dead subscribers alive.
pub(crate) type SubFn = Arc<dyn Fn() + Send + Sync>;

/// Thunk that removes the active observer from a specific signal's subscriber
/// list. Recorded by `Signal::get` whenever it adds a new subscription; fired
/// by the effect machinery at the start of the next run and on `Disposer::drop`
/// to implement unsubscribe-on-rerun semantics.
pub(crate) type UntrackThunk = Box<dyn FnOnce() + Send + Sync>;

thread_local! {
    /// The currently-executing observer (effect run fn or derived invalidate fn).
    /// When Some, any `Signal::get()` call auto-subscribes this observer.
    pub(crate) static CURRENT_OBSERVER: RefCell<Option<SubFn>> = const { RefCell::new(None) };

    /// Collection of untrack thunks pushed by `Signal::get` during the current
    /// tracked run. `Some(...)` only inside an effect/derived run; `None`
    /// outside any tracking scope.
    pub(crate) static CURRENT_UNTRACKS: RefCell<Option<Vec<UntrackThunk>>> =
        const { RefCell::new(None) };

    /// Nesting depth of active `batch()` calls. Subscribers are deferred while > 0.
    pub(crate) static BATCH_DEPTH: Cell<u32> = const { Cell::new(0) };

    /// Subscribers queued for flush when the outermost `batch()` returns.
    pub(crate) static PENDING: RefCell<Vec<SubFn>> = const { RefCell::new(Vec::new()) };
}

/// Swap the current observer. Returns the previous one.
pub(crate) fn set_observer(next: Option<SubFn>) -> Option<SubFn> {
    CURRENT_OBSERVER.with(|obs| {
        let prev = obs.borrow().clone();
        *obs.borrow_mut() = next;
        prev
    })
}

/// Clone the current observer without consuming it.
pub(crate) fn get_observer() -> Option<SubFn> {
    CURRENT_OBSERVER.with(|obs| obs.borrow().clone())
}

/// Begin a tracked scope: install a fresh untrack buffer, return whatever was
/// there before so it can be restored on exit. Pair with `exit_tracking`.
pub(crate) fn enter_tracking() -> Option<Vec<UntrackThunk>> {
    CURRENT_UNTRACKS.with(|u| u.replace(Some(Vec::new())))
}

/// End a tracked scope. Returns the untracks collected during this scope and
/// restores the prior scope's buffer (if any).
pub(crate) fn exit_tracking(prev: Option<Vec<UntrackThunk>>) -> Vec<UntrackThunk> {
    CURRENT_UNTRACKS.with(|u| {
        let new = u.replace(prev);
        new.unwrap_or_default()
    })
}

/// Push an untrack thunk onto the current tracking scope. No-op when called
/// outside any effect/derived run (e.g. a one-shot `Signal::peek` or `get`
/// outside reactive context).
pub(crate) fn record_untrack(thunk: UntrackThunk) {
    CURRENT_UNTRACKS.with(|u| {
        if let Some(list) = u.borrow_mut().as_mut() {
            list.push(thunk);
        }
    });
}

/// True while inside a `batch()` call.
pub(crate) fn is_batching() -> bool {
    BATCH_DEPTH.with(|d| d.get() > 0)
}

pub(crate) fn batch_enter() {
    BATCH_DEPTH.with(|d| d.set(d.get() + 1));
}

/// Decrement batch depth. If reaching zero, flush all pending subscribers.
pub(crate) fn batch_exit() {
    BATCH_DEPTH.with(|d| {
        let depth = d.get();
        if depth > 0 {
            d.set(depth - 1);
        }
    });
    if !is_batching() {
        flush_pending();
    }
}

pub(crate) fn queue_pending(subs: Vec<SubFn>) {
    PENDING.with(|p| {
        let mut pending = p.borrow_mut();
        for sub in subs {
            // Deduplicate by pointer identity, matching TS `Set<Subscriber>` semantics.
            if !pending.iter().any(|s| Arc::ptr_eq(s, &sub)) {
                pending.push(sub);
            }
        }
    });
}

fn flush_pending() {
    let subs: Vec<SubFn> = PENDING.with(|p| std::mem::take(&mut *p.borrow_mut()));
    for s in subs {
        s();
    }
}
