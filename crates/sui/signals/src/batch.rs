//! `batch(fn)` — coalesce multiple signal writes into one notification cycle.

use crate::runtime::{batch_enter, batch_exit};

/// Group signal writes inside `f` into a single notification cycle. All
/// subscribers that would fire during `f` are deferred and flushed exactly
/// once when the outermost `batch` returns. Nests safely.
///
/// # Example
/// ```rust
/// use sui_signals::{Signal, effect, batch};
///
/// let x: Signal<i32> = Signal::new(0);
/// let y: Signal<i32> = Signal::new(0);
/// let sum = std::sync::Arc::new(std::sync::Mutex::new(0i32));
///
/// let sum_clone = sum.clone();
/// let x2 = x.clone();
/// let y2 = y.clone();
/// let _d = effect(move || { *sum_clone.lock().unwrap() = x2.get() + y2.get(); });
///
/// batch(|| {
///     x.set(1);
///     y.set(2);
/// });
/// assert_eq!(*sum.lock().unwrap(), 3); // effect ran once, not twice
/// ```
pub fn batch<F: FnOnce()>(f: F) {
    batch_enter();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
    batch_exit();
    if let Err(payload) = result {
        std::panic::resume_unwind(payload);
    }
}
