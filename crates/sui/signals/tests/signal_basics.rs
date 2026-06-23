//! Algorithm-correctness tests — mirrors the TS signal.ts spec.

use std::sync::{Arc, Mutex};
use sui_signals::{Signal, batch, derived, effect, untrack};

// ── signal get/set ───────────────────────────────────────────────────────────

#[test]
fn signal_initial_value() {
    let s: Signal<i32> = Signal::new(42);
    assert_eq!(s.peek(), 42);
}

#[test]
fn signal_set_changes_value() {
    let s: Signal<i32> = Signal::new(0);
    s.set(7);
    assert_eq!(s.peek(), 7);
}

#[test]
fn signal_set_equal_no_notify() {
    let count = Arc::new(Mutex::new(0usize));
    let s: Signal<i32> = Signal::new(5);

    let c = count.clone();
    let s2 = s.clone();
    let _d = effect(move || {
        s2.get();
        *c.lock().unwrap() += 1;
    });
    assert_eq!(*count.lock().unwrap(), 1); // initial run

    s.set(5); // same value — should not notify
    assert_eq!(*count.lock().unwrap(), 1);

    s.set(6); // different value — should notify
    assert_eq!(*count.lock().unwrap(), 2);
}

// ── effect ───────────────────────────────────────────────────────────────────

#[test]
fn effect_runs_eagerly() {
    let ran = Arc::new(Mutex::new(false));
    let r = ran.clone();
    let _d = effect(move || { *r.lock().unwrap() = true; });
    assert!(*ran.lock().unwrap());
}

#[test]
fn effect_reruns_on_signal_change() {
    let log = Arc::new(Mutex::new(Vec::<i32>::new()));
    let s: Signal<i32> = Signal::new(1);

    let log2 = log.clone();
    let s2 = s.clone();
    let _d = effect(move || { log2.lock().unwrap().push(s2.get()); });

    s.set(2);
    s.set(3);

    assert_eq!(*log.lock().unwrap(), vec![1, 2, 3]);
}

#[test]
fn effect_disposer_stops_reruns() {
    let count = Arc::new(Mutex::new(0usize));
    let s: Signal<i32> = Signal::new(0);

    let c = count.clone();
    let s2 = s.clone();
    let disposer = effect(move || {
        s2.get();
        *c.lock().unwrap() += 1;
    });
    assert_eq!(*count.lock().unwrap(), 1);

    drop(disposer); // dispose
    s.set(1);

    assert_eq!(*count.lock().unwrap(), 1); // no re-run after disposal
}

#[test]
fn effect_tracks_multiple_signals() {
    let a: Signal<i32> = Signal::new(1);
    let b: Signal<i32> = Signal::new(2);
    let log = Arc::new(Mutex::new(Vec::<i32>::new()));

    let log2 = log.clone();
    let a2 = a.clone();
    let b2 = b.clone();
    let _d = effect(move || { log2.lock().unwrap().push(a2.get() + b2.get()); });

    a.set(10);
    b.set(20);

    assert_eq!(*log.lock().unwrap(), vec![3, 12, 30]);
}

// ── derived ──────────────────────────────────────────────────────────────────

#[test]
fn derived_computes_initial() {
    let s: Signal<i32> = Signal::new(4);
    let d = derived({ let s = s.clone(); move || s.get() * 2 });
    assert_eq!(d.peek(), 8);
}

#[test]
fn derived_updates_on_dep_change() {
    let s: Signal<i32> = Signal::new(3);
    let d = derived({ let s = s.clone(); move || s.get() * 3 });
    s.set(4);
    assert_eq!(d.peek(), 12);
}

#[test]
fn derived_notifies_effect() {
    let s: Signal<i32> = Signal::new(1);
    let doubled = derived({ let s = s.clone(); move || s.get() * 2 });
    let log = Arc::new(Mutex::new(Vec::<i32>::new()));

    let log2 = log.clone();
    let d2 = doubled.clone();
    let _e = effect(move || { log2.lock().unwrap().push(d2.get()); });

    s.set(2);
    s.set(5);

    assert_eq!(*log.lock().unwrap(), vec![2, 4, 10]);
}

// ── batch ────────────────────────────────────────────────────────────────────

#[test]
fn batch_defers_notifications() {
    let x: Signal<i32> = Signal::new(0);
    let y: Signal<i32> = Signal::new(0);
    let log = Arc::new(Mutex::new(Vec::<i32>::new()));

    let log2 = log.clone();
    let x2 = x.clone();
    let y2 = y.clone();
    let _d = effect(move || { log2.lock().unwrap().push(x2.get() + y2.get()); });
    assert_eq!(*log.lock().unwrap(), vec![0]); // initial

    batch(|| {
        x.set(1);
        y.set(2);
    });

    // Effect ran once after batch, not once per set
    assert_eq!(*log.lock().unwrap(), vec![0, 3]);
}

#[test]
fn batch_nests_safely() {
    let s: Signal<i32> = Signal::new(0);
    let count = Arc::new(Mutex::new(0usize));

    let c = count.clone();
    let s2 = s.clone();
    let _d = effect(move || { s2.get(); *c.lock().unwrap() += 1; });
    assert_eq!(*count.lock().unwrap(), 1); // initial

    batch(|| {
        batch(|| { s.set(1); });
        s.set(2);
    });

    assert_eq!(*count.lock().unwrap(), 2); // one re-run after outermost batch
}

// ── untrack ──────────────────────────────────────────────────────────────────

#[test]
fn untrack_prevents_subscription() {
    let s: Signal<i32> = Signal::new(0);
    let other: Signal<i32> = Signal::new(99);
    let count = Arc::new(Mutex::new(0usize));

    let c = count.clone();
    let s2 = s.clone();
    let o2 = other.clone();
    let _d = effect(move || {
        s2.get(); // tracked dep
        untrack(|| o2.get()); // not a dep
        *c.lock().unwrap() += 1;
    });
    assert_eq!(*count.lock().unwrap(), 1);

    other.set(100); // should not re-trigger effect
    assert_eq!(*count.lock().unwrap(), 1);

    s.set(1); // tracked — should re-trigger
    assert_eq!(*count.lock().unwrap(), 2);
}

// ── peek ─────────────────────────────────────────────────────────────────────

#[test]
fn peek_does_not_subscribe() {
    let s: Signal<i32> = Signal::new(0);
    let count = Arc::new(Mutex::new(0usize));

    let c = count.clone();
    let s2 = s.clone();
    let _d = effect(move || {
        s2.peek(); // peek — not a dep
        *c.lock().unwrap() += 1;
    });
    assert_eq!(*count.lock().unwrap(), 1);

    s.set(1);
    assert_eq!(*count.lock().unwrap(), 1); // no re-run
}

// ── set_unchecked ────────────────────────────────────────────────────────────

#[test]
fn set_unchecked_always_notifies() {
    let s: Signal<i32> = Signal::new(5);
    let count = Arc::new(Mutex::new(0usize));

    let c = count.clone();
    let s2 = s.clone();
    let _d = effect(move || { s2.get(); *c.lock().unwrap() += 1; });
    assert_eq!(*count.lock().unwrap(), 1);

    s.set_unchecked(5); // same value but always notifies
    assert_eq!(*count.lock().unwrap(), 2);
}
