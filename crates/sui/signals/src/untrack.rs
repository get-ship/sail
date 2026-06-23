//! `untrack(fn)` — read signals without subscribing.

use crate::runtime::set_observer;

/// Execute `f`, suppressing any dependency tracking. Signals read inside `f`
/// are not registered as dependencies of the current reactive scope.
///
/// Useful inside `effect` or `derived` when a value is consulted but should
/// not trigger re-runs (e.g. reading a timestamp or a peer signal as context).
///
/// # Example
/// ```rust
/// use sui_signals::{Signal, effect, untrack};
///
/// let count: Signal<i32> = Signal::new(0);
/// let label: Signal<String> = Signal::new("items".into());
/// let out = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
///
/// let out2 = out.clone();
/// let count2 = count.clone();
/// let label2 = label.clone();
/// let _d = effect(move || {
///     let lbl = untrack(|| label2.get()); // not a dep
///     *out2.lock().unwrap() = format!("{}: {}", lbl, count2.get());
/// });
///
/// count.set(1); // re-runs effect
/// label.set("things".into()); // does NOT re-run effect
/// ```
pub fn untrack<T, F: FnOnce() -> T>(f: F) -> T {
    let prev = set_observer(None);
    let result = f();
    set_observer(prev);
    result
}
