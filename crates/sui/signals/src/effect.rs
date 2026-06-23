//! `effect(fn) -> Disposer` — reactive side effects.

use std::sync::{
    Arc, Mutex, OnceLock,
    atomic::{AtomicBool, Ordering},
};

use crate::runtime::{SubFn, UntrackThunk, enter_tracking, exit_tracking, set_observer};

struct EffectCore {
    active: AtomicBool,
    /// Self-reference used to pass this effect as the current observer during runs.
    /// Initialized once immediately after construction; always Some when `run` executes.
    run_ref: OnceLock<SubFn>,
    /// Unsubscribe thunks collected during the most-recent tracked run. Drained
    /// (and fired) at the start of the next run and on `Disposer::drop` to
    /// implement unsubscribe-on-rerun: deps the effect no longer reads stop
    /// invalidating it.
    untracks: Mutex<Vec<UntrackThunk>>,
}

/// Handle returned by `effect`. Severs the subscription and stops re-runs when dropped.
pub struct Disposer(Arc<EffectCore>);

impl Drop for Disposer {
    fn drop(&mut self) {
        // Flip the active flag first so any in-flight notification short-circuits.
        self.0.active.store(false, Ordering::Release);
        // Proactively remove this effect from every signal it was subscribed to.
        let untracks = std::mem::take(
            &mut *self
                .0
                .untracks
                .lock()
                .unwrap_or_else(|e| e.into_inner()),
        );
        for u in untracks {
            u();
        }
    }
}

/// Run `f` once eagerly to establish subscriptions, then re-run `f` whenever
/// any signal read inside `f` changes. Returns a `Disposer` that stops further
/// runs when dropped.
///
/// Re-runs are synchronous: they happen on the thread that calls `Signal::set`,
/// inside the signal's notification loop.
///
/// Dependencies are tracked per-run: signals read in run N are subscribed to,
/// signals read in run N-1 but not N are unsubscribed. Conditional dep
/// patterns (e.g. `if cond.get() { a.get() } else { b.get() }`) do not retain
/// stale subscriptions.
pub fn effect<F: Fn() + Send + Sync + 'static>(f: F) -> Disposer {
    let core = Arc::new(EffectCore {
        active: AtomicBool::new(true),
        run_ref: OnceLock::new(),
        untracks: Mutex::new(Vec::new()),
    });

    let core_weak = Arc::downgrade(&core);
    let f = Arc::new(f);

    let run: SubFn = Arc::new({
        let core_weak = core_weak.clone();
        let f = Arc::clone(&f);
        move || {
            let Some(core) = core_weak.upgrade() else { return };
            if !core.active.load(Ordering::Acquire) {
                return;
            }
            let Some(run_fn) = core.run_ref.get() else { return };

            // Drop prior subscriptions so we cleanly re-track this run.
            let old_untracks = std::mem::take(
                &mut *core.untracks.lock().unwrap_or_else(|e| e.into_inner()),
            );
            for u in old_untracks {
                u();
            }

            // Begin a fresh tracking scope and install ourselves as observer.
            let prev_untracks = enter_tracking();
            let prev_observer = set_observer(Some(Arc::clone(run_fn)));

            // Run f under tracking. If f panics, restore state before unwinding.
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f()));

            set_observer(prev_observer);
            let new_untracks = exit_tracking(prev_untracks);

            *core.untracks.lock().unwrap_or_else(|e| e.into_inner()) = new_untracks;

            if let Err(payload) = result {
                std::panic::resume_unwind(payload);
            }
        }
    });

    // Store the run Arc inside core so the closure can clone it as CURRENT_OBSERVER.
    // `run` outlives this store (it's also held locally), so the OnceLock never sees None.
    core.run_ref
        .set(Arc::clone(&run))
        .unwrap_or_else(|_| unreachable!("OnceLock set exactly once"));

    // Eager initial run establishes subscriptions.
    run();

    Disposer(core)
}
