//! `Signal<T>` — the reactive source primitive.

use std::sync::{Arc, Mutex, Weak};

use crate::runtime::{SubFn, get_observer, is_batching, queue_pending, record_untrack};

type WeakSub = Weak<dyn Fn() + Send + Sync>;

struct Inner<T> {
    value: T,
    /// Weak refs so subscribers don't keep the signal alive, and dead entries
    /// are pruned automatically on the next `set` call.
    subs: Vec<WeakSub>,
}

/// A fine-grained reactive source. Cheap to clone (Arc under the hood).
///
/// `get()` reads the value and, when called inside an `effect` or `derived`,
/// registers the caller as a subscriber. `set()` / `set_unchecked()` write a
/// new value and synchronously notify all live subscribers (or queue them if
/// inside a `batch`).
///
/// `T` must be `Clone + Send + Sync + 'static`. Use `set()` when `T:
/// PartialEq` to skip no-op writes; `set_unchecked()` always notifies.
pub struct Signal<T: Clone + Send + Sync + 'static>(Arc<Mutex<Inner<T>>>);

impl<T: Clone + Send + Sync + 'static> Clone for Signal<T> {
    fn clone(&self) -> Self {
        Signal(Arc::clone(&self.0))
    }
}

// Signal is Send + Sync if T is Send + Sync (Mutex<Inner<T>> provides that).
unsafe impl<T: Clone + Send + Sync + 'static> Send for Signal<T> {}
unsafe impl<T: Clone + Send + Sync + 'static> Sync for Signal<T> {}

impl<T: Clone + Send + Sync + 'static> Signal<T> {
    pub fn new(initial: T) -> Self {
        Signal(Arc::new(Mutex::new(Inner {
            value: initial,
            subs: Vec::new(),
        })))
    }

    /// Read the current value, registering the current reactive scope as a
    /// subscriber. Call `peek()` for non-tracking reads.
    pub fn get(&self) -> T {
        let observer = get_observer();
        let (value, newly_subscribed_sub) = {
            let mut inner = self.lock();
            let newly_sub = observer.as_ref().and_then(|sub| {
                if subscribe_if_new(&mut inner.subs, Arc::clone(sub)) {
                    Some(Arc::clone(sub))
                } else {
                    None
                }
            });
            (inner.value.clone(), newly_sub)
        };

        // Record an untrack thunk so the active observer can drop its
        // subscription to this signal on its next tracked run / disposal.
        // We only record on the first subscription per (observer, signal) per
        // run — subsequent reads of the same signal dedupe via subscribe_if_new.
        if let Some(sub) = newly_subscribed_sub {
            let inner_arc = Arc::clone(&self.0);
            record_untrack(Box::new(move || {
                let mut inner = inner_arc.lock().unwrap_or_else(|e| e.into_inner());
                inner.subs.retain(|w| match w.upgrade() {
                    Some(s) => !Arc::ptr_eq(&s, &sub),
                    None => false,
                });
            }));
        }

        value
    }

    /// Read the current value without subscribing the current scope.
    pub fn peek(&self) -> T {
        self.lock().value.clone()
    }

    /// Write a new value. Skips notification when the new value equals the
    /// current value (`T: PartialEq`). Use `set_unchecked` to always notify.
    pub fn set(&self, value: T)
    where
        T: PartialEq,
    {
        let subs = {
            let mut inner = self.lock();
            if inner.value == value {
                return;
            }
            inner.value = value;
            drain_live(&mut inner.subs)
        };
        self.dispatch(subs);
    }

    /// Write a new value, always notifying subscribers regardless of equality.
    pub fn set_unchecked(&self, value: T) {
        let subs = {
            let mut inner = self.lock();
            inner.value = value;
            drain_live(&mut inner.subs)
        };
        self.dispatch(subs);
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Inner<T>> {
        self.0.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn dispatch(&self, subs: Vec<SubFn>) {
        if subs.is_empty() {
            return;
        }
        if is_batching() {
            queue_pending(subs);
        } else {
            for s in subs {
                s();
            }
        }
    }
}

/// Add `sub` to `subs` only if it isn't already present (pointer equality).
/// Returns true when this call added a new subscription.
fn subscribe_if_new(subs: &mut Vec<WeakSub>, sub: SubFn) -> bool {
    subs.retain(|w| w.strong_count() > 0);
    let already = subs
        .iter()
        .any(|w| w.upgrade().is_some_and(|s| Arc::ptr_eq(&s, &sub)));
    if !already {
        subs.push(Arc::downgrade(&sub));
    }
    !already
}

/// Upgrade all live subs, prune dead ones, return the live arcs.
fn drain_live(subs: &mut Vec<WeakSub>) -> Vec<SubFn> {
    let mut live = Vec::new();
    subs.retain(|w| {
        if let Some(s) = w.upgrade() {
            live.push(s);
            true
        } else {
            false
        }
    });
    live
}
