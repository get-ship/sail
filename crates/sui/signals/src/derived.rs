//! `derived(fn) -> Derived<T>` — memoized computed values.
//!
//! Implemented as a push-based Signal<T> backed by an internal effect: when
//! any tracked dependency changes the effect re-runs `fn` and calls `set` on
//! the inner signal, notifying derived's own subscribers only when the value
//! actually changes. Recompute is eager (happens on dep change, not on read),
//! which suits the daemon's subscription model.

use crate::effect::{Disposer, effect};
use crate::signal::Signal;
use crate::untrack::untrack;

/// A read-only computed signal. Cloning is cheap; all clones share state.
pub struct Derived<T: Clone + Send + Sync + PartialEq + 'static> {
    inner: Signal<T>,
    /// Keeps the internal effect alive; dropped when the last Derived clone drops.
    _owner: std::sync::Arc<Disposer>,
}

impl<T: Clone + Send + Sync + PartialEq + 'static> Clone for Derived<T> {
    fn clone(&self) -> Self {
        Derived {
            inner: self.inner.clone(),
            _owner: std::sync::Arc::clone(&self._owner),
        }
    }
}

impl<T: Clone + Send + Sync + PartialEq + 'static> Derived<T> {
    /// Read the current computed value, subscribing the current scope.
    pub fn get(&self) -> T {
        self.inner.get()
    }

    /// Read the current computed value without subscribing.
    pub fn peek(&self) -> T {
        self.inner.peek()
    }
}

/// Create a memoized computed value. `fn` runs once eagerly (untracked for
/// seeding) then re-runs under tracking whenever any dependency changes.
/// Subscribers are notified only when the recomputed value differs from the
/// cached one.
pub fn derived<T, F>(f: F) -> Derived<T>
where
    T: Clone + Send + Sync + PartialEq + 'static,
    F: Fn() -> T + Send + Sync + 'static,
{
    // Seed without tracking to avoid spurious dep during construction.
    let initial = untrack(&f);
    let inner = Signal::new(initial);

    let sig = inner.clone();
    let disposer = effect(move || {
        let val = f(); // tracked run — subscribes to deps
        sig.set(val); // no-op if value unchanged
    });

    Derived {
        inner,
        _owner: std::sync::Arc::new(disposer),
    }
}
