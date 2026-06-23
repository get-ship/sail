//! Fine-grained reactive signals for the Ship daemon.
//!
//! All primitives are `Send + Sync` and safe to share across tokio tasks.
//! The algorithm mirrors `packages/sui/runtime/src/signal.ts`:
//! signals are sources, `derived` values are memoized computeds, and
//! `effect` subscribes and re-runs on dependency change. Reads inside a
//! tracked scope (effect / derived) auto-subscribe; writes notify synchronously
//! (or coalesce inside `batch`).
//!
//! ## Quick start
//!
//! ```rust
//! use sui_signals::{Signal, effect, derived, batch, untrack};
//!
//! let count: Signal<i32> = Signal::new(0);
//! let double = derived({
//!     let c = count.clone();
//!     move || c.get() * 2
//! });
//!
//! let log = std::sync::Arc::new(std::sync::Mutex::new(Vec::<i32>::new()));
//! let log2 = log.clone();
//! let d = double.clone();
//! let _dispose = effect(move || { log2.lock().unwrap().push(d.get()); });
//!
//! count.set(1);
//! count.set(2);
//! assert_eq!(*log.lock().unwrap(), vec![0, 2, 4]);
//! ```

mod runtime;

mod signal;
mod effect;
mod derived;
mod batch;
mod untrack;

pub use signal::Signal;
pub use effect::{Disposer, effect};
pub use derived::{Derived, derived};
pub use batch::batch;
pub use untrack::untrack;
