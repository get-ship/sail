//! Notification reentrance, batch nesting from effects, panic recovery, and
//! cross-thread effect dispatch.

use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use sui_signals::{Signal, batch, derived, effect};

// ── E5. `set` inside a subscriber (reentrance) ──────────────────────────────

#[test]
fn set_inside_subscriber_is_consistent() {
    // An effect on signal `a` writes signal `b`. An effect on `b` records
    // the value. Both effects must observe consistent state after a single
    // top-level `a.set`.
    let a: Signal<i32> = Signal::new(0);
    let b: Signal<i32> = Signal::new(0);
    let log = Arc::new(Mutex::new(Vec::<i32>::new()));

    let b_writer = b.clone();
    let a2 = a.clone();
    let _e1 = effect(move || {
        let v = a2.get();
        // Write b synchronously inside a's effect.
        b_writer.set(v * 10);
    });

    let log2 = log.clone();
    let b2 = b.clone();
    let _e2 = effect(move || {
        log2.lock().unwrap().push(b2.get());
    });
    assert_eq!(*log.lock().unwrap(), vec![0]); // initial state of b

    a.set(3);
    // After a=3: e1 fires, writes b=30, which fires e2, which logs 30.
    assert_eq!(*log.lock().unwrap(), vec![0, 30]);
}

#[test]
fn set_inside_subscriber_to_same_signal_terminates() {
    // An effect that writes the SAME signal it reads is a self-loop. With
    // PartialEq short-circuit it should terminate after one extra notify.
    let s: Signal<i32> = Signal::new(0);
    let writes = Arc::new(AtomicUsize::new(0));

    let writes2 = Arc::clone(&writes);
    let s2 = s.clone();
    let _e = effect(move || {
        let v = s2.get();
        if v < 5 {
            writes2.fetch_add(1, Ordering::Relaxed);
            s2.set(v + 1);
        }
    });

    // Initial value 0 < 5 → set to 1 → effect re-runs → set to 2 → ... → set to 5.
    // At v=5, condition false, no more writes. So 5 internal writes.
    assert_eq!(s.peek(), 5);
    assert_eq!(writes.load(Ordering::Relaxed), 5);
}

// ── E6. batch inside an effect ──────────────────────────────────────────────

#[test]
fn batch_inside_effect_coalesces_correctly() {
    let trigger: Signal<i32> = Signal::new(0);
    let a: Signal<i32> = Signal::new(0);
    let b: Signal<i32> = Signal::new(0);
    let runs = Arc::new(AtomicUsize::new(0));

    // An observer of both a and b that counts re-runs.
    let runs2 = Arc::clone(&runs);
    let a_obs = a.clone();
    let b_obs = b.clone();
    let _watch = effect(move || {
        a_obs.get();
        b_obs.get();
        runs2.fetch_add(1, Ordering::Relaxed);
    });
    assert_eq!(runs.load(Ordering::Relaxed), 1); // initial

    // Effect on trigger that writes a and b inside a batch.
    let a_w = a.clone();
    let b_w = b.clone();
    let trigger2 = trigger.clone();
    let _e = effect(move || {
        let _ = trigger2.get();
        batch(|| {
            a_w.set(a_w.peek() + 1);
            b_w.set(b_w.peek() + 1);
        });
    });

    // Initial effect run wrote a=1, b=1, batched. The watcher re-ran twice
    // total at this point: once for its own initial, once for the batch flush
    // after the trigger-effect's initial run.
    let after_initial = runs.load(Ordering::Relaxed);
    assert!((2..=3).contains(&after_initial));

    let before = runs.load(Ordering::Relaxed);
    trigger.set(1);
    // trigger effect re-ran, wrote a=2 and b=2 in a batch.
    // watcher should re-run exactly once (batch coalesced both writes).
    let after = runs.load(Ordering::Relaxed);
    assert_eq!(
        after - before,
        1,
        "batched writes inside an effect must coalesce into one observer re-run"
    );
}

#[test]
fn nested_batch_only_flushes_at_outermost() {
    let s: Signal<i32> = Signal::new(0);
    let count = Arc::new(AtomicUsize::new(0));

    let c2 = Arc::clone(&count);
    let s2 = s.clone();
    let _e = effect(move || {
        s2.get();
        c2.fetch_add(1, Ordering::Relaxed);
    });
    assert_eq!(count.load(Ordering::Relaxed), 1); // initial

    batch(|| {
        s.set(1);
        let mid = count.load(Ordering::Relaxed);
        assert_eq!(mid, 1, "no flush mid-batch");
        batch(|| {
            s.set(2);
            let inner = count.load(Ordering::Relaxed);
            assert_eq!(inner, 1, "no flush inside inner batch");
        });
        let after_inner = count.load(Ordering::Relaxed);
        assert_eq!(
            after_inner, 1,
            "inner batch exit did not flush (still inside outer)"
        );
        s.set(3);
    });
    // Outer batch exits → single flush.
    assert_eq!(count.load(Ordering::Relaxed), 2);
}

// ── E9. Panic in effect closure does not corrupt tracking state ─────────────

#[test]
fn panicking_effect_does_not_leave_observer_installed() {
    use std::panic::{catch_unwind, AssertUnwindSafe};

    let s: Signal<i32> = Signal::new(0);

    let s2 = s.clone();
    let result = catch_unwind(AssertUnwindSafe(|| {
        let _d = effect(move || {
            s2.get();
            panic!("boom");
        });
        // unreachable — effect's eager run panics
    }));
    assert!(result.is_err());

    // After the panic, the observer must be unset. Verify by creating a new
    // signal-read context: read s outside any effect and confirm no observer
    // is tracking by checking that a later set does not fire any effect.
    let outside = Signal::new(0);
    let fired = Arc::new(AtomicUsize::new(0));
    let f2 = Arc::clone(&fired);
    let outside2 = outside.clone();
    let _d = effect(move || {
        outside2.get();
        f2.fetch_add(1, Ordering::Relaxed);
    });
    assert_eq!(fired.load(Ordering::Relaxed), 1);

    // Most importantly: writing the signal from the panicked effect must NOT
    // resurrect the dead effect.
    s.set(99);
    s.set(100);
    assert_eq!(
        fired.load(Ordering::Relaxed),
        1,
        "writing s after a panicked effect's death must not fire anything"
    );
}

#[test]
fn panicking_derived_does_not_leave_observer_installed() {
    use std::panic::{catch_unwind, AssertUnwindSafe};

    let s: Signal<i32> = Signal::new(0);

    let s2 = s.clone();
    let result = catch_unwind(AssertUnwindSafe(|| {
        let _d: sui_signals::Derived<i32> = derived(move || {
            s2.get();
            panic!("boom");
        });
    }));
    assert!(result.is_err());

    // The derived's internal effect panicked during its initial run. After
    // that, the runtime must be back to a clean state — writes to s should
    // not invoke any phantom subscriber.
    s.set(1);
    s.set(2);
    // Implicit: reaching this point without panic means no observer was
    // left dangling that fired during set.
}

// ── E8. Cross-thread effect re-run ──────────────────────────────────────────

#[test]
fn effect_created_on_main_thread_reruns_on_writing_thread() {
    let s: Signal<i32> = Signal::new(0);
    let log = Arc::new(Mutex::new(Vec::<(std::thread::ThreadId, i32)>::new()));

    let log2 = log.clone();
    let s2 = s.clone();
    let _d = effect(move || {
        let v = s2.get();
        log2.lock().unwrap().push((std::thread::current().id(), v));
    });
    let main_id = std::thread::current().id();
    assert_eq!(log.lock().unwrap()[0].0, main_id);

    let s3 = s.clone();
    let writer_handle = std::thread::spawn(move || {
        s3.set(42);
        std::thread::current().id()
    });
    let writer_id = writer_handle.join().unwrap();

    let entries = log.lock().unwrap().clone();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[1].0, writer_id, "effect re-ran on the writing thread");
    assert_eq!(entries[1].1, 42);
}

// ── E4. Disposer drop during mid-notify ─────────────────────────────────────

#[test]
fn disposing_an_effect_during_its_own_notification_is_safe() {
    // Effect e1 holds a clone of e2's Disposer in shared state. When a write
    // fires both, e1's run drops e2 (mid-notification). e2 should not re-run.
    let s: Signal<i32> = Signal::new(0);
    let other_disposer: Arc<Mutex<Option<sui_signals::Disposer>>> = Arc::new(Mutex::new(None));
    let e2_runs = Arc::new(AtomicUsize::new(0));

    // e2: counts runs.
    let runs2 = Arc::clone(&e2_runs);
    let s2 = s.clone();
    let e2 = effect(move || {
        s2.get();
        runs2.fetch_add(1, Ordering::Relaxed);
    });
    *other_disposer.lock().unwrap() = Some(e2);
    assert_eq!(e2_runs.load(Ordering::Relaxed), 1);

    // e1: on write, drops e2. Whether e1 fires before or after e2 in the
    // notification loop is undefined, but no panic / use-after-free should
    // occur, and post-drop writes must not fire e2.
    let other2 = Arc::clone(&other_disposer);
    let s3 = s.clone();
    let _e1 = effect(move || {
        let v = s3.get();
        if v == 99 {
            *other2.lock().unwrap() = None; // drops e2
        }
    });

    // First write — both effects fire.
    s.set(1);
    assert_eq!(e2_runs.load(Ordering::Relaxed), 2);

    // Trigger e2 disposal via e1.
    s.set(99);
    let after_dispose = e2_runs.load(Ordering::Relaxed);
    // Either e2 ran one more time before being dropped (3) or didn't (2).
    assert!(after_dispose == 2 || after_dispose == 3);

    // Subsequent writes must not fire e2.
    s.set(100);
    s.set(101);
    assert_eq!(e2_runs.load(Ordering::Relaxed), after_dispose);
}
