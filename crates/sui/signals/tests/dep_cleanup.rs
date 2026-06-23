//! Unsubscribe-on-rerun semantics. Effects and deriveds must drop subscriptions
//! to signals they no longer read on the previous tracked run. Without this,
//! conditional dep changes cause spurious re-runs and unbounded memory growth.

use std::sync::{Arc, Mutex};
use sui_signals::{Signal, derived, effect};

// ── B1. Conditional dep in effect ────────────────────────────────────────────

#[test]
fn effect_drops_dep_when_no_longer_read() {
    // Effect reads `cond`, and conditionally reads `a` xor `b`.
    // After flipping cond, writes to the previously-read signal should NOT
    // re-trigger the effect; only the currently-read one should.
    let cond: Signal<bool> = Signal::new(true);
    let a: Signal<i32> = Signal::new(0);
    let b: Signal<i32> = Signal::new(0);
    let runs = Arc::new(Mutex::new(Vec::<&'static str>::new()));

    let runs2 = runs.clone();
    let cond2 = cond.clone();
    let a2 = a.clone();
    let b2 = b.clone();
    let _e = effect(move || {
        if cond2.get() {
            a2.get();
            runs2.lock().unwrap().push("a-branch");
        } else {
            b2.get();
            runs2.lock().unwrap().push("b-branch");
        }
    });

    assert_eq!(*runs.lock().unwrap(), vec!["a-branch"]); // initial

    // Flip to b-branch.
    cond.set(false);
    assert_eq!(*runs.lock().unwrap(), vec!["a-branch", "b-branch"]);

    // Now `a` should NO LONGER be a dep. Writing `a` must not re-run.
    a.set(1);
    assert_eq!(
        *runs.lock().unwrap(),
        vec!["a-branch", "b-branch"],
        "writing former dep `a` after flip triggered a stale re-run"
    );

    // `b` should be a dep. Writing it must re-run.
    b.set(1);
    assert_eq!(
        *runs.lock().unwrap(),
        vec!["a-branch", "b-branch", "b-branch"]
    );
}

// ── E1. Conditional dep in derived ──────────────────────────────────────────

#[test]
fn derived_drops_dep_when_no_longer_read() {
    // Fresh fixture — the derived's compute counter is what we assert on.
    let cond: Signal<bool> = Signal::new(true);
    let a: Signal<i32> = Signal::new(10);
    let b: Signal<i32> = Signal::new(20);
    let compute_count = Arc::new(Mutex::new(0usize));

    let count2 = compute_count.clone();
    let cond2 = cond.clone();
    let a2 = a.clone();
    let b2 = b.clone();
    let d = derived(move || {
        *count2.lock().unwrap() += 1;
        if cond2.get() { a2.get() } else { b2.get() }
    });
    let _ = d.peek();
    let initial = *compute_count.lock().unwrap();
    assert!(initial >= 1);

    // Flip to b-branch. compute should re-run.
    cond.set(false);
    let after_flip = *compute_count.lock().unwrap();
    assert!(after_flip > initial, "flip should recompute");

    // Write old dep `a`. Must NOT recompute (a is no longer read).
    a.set(99);
    let after_a = *compute_count.lock().unwrap();
    assert_eq!(
        after_a, after_flip,
        "writing former dep `a` triggered a stale derived recompute"
    );

    // Write new dep `b`. Must recompute.
    b.set(99);
    let after_b = *compute_count.lock().unwrap();
    assert!(after_b > after_a, "new dep `b` should recompute");
}

// ── E2. Derived → derived → effect chain ────────────────────────────────────

#[test]
fn derived_chain_cleans_up_intermediate_stale_deps() {
    let a: Signal<i32> = Signal::new(1);
    let b: Signal<i32> = Signal::new(100);
    let toggle: Signal<bool> = Signal::new(true);

    // mid switches between a-branch and b-branch.
    let toggle2 = toggle.clone();
    let a2 = a.clone();
    let b2 = b.clone();
    let mid = derived(move || if toggle2.get() { a2.get() } else { b2.get() });

    let mid2 = mid.clone();
    let outer = derived(move || mid2.get() * 2);
    let outer_runs = Arc::new(Mutex::new(0usize));

    let runs2 = outer_runs.clone();
    let outer2 = outer.clone();
    let _e = effect(move || {
        outer2.get();
        *runs2.lock().unwrap() += 1;
    });
    let initial = *outer_runs.lock().unwrap();

    toggle.set(false); // mid switches to b-branch
    let after_flip = *outer_runs.lock().unwrap();
    assert!(after_flip > initial);

    // Writing `a` (no longer in mid's deps) must not re-run the outer effect.
    a.set(42);
    let after_a = *outer_runs.lock().unwrap();
    assert_eq!(after_a, after_flip, "stale a-dep re-triggered chain");
}

// ── E3. Same signal read multiple times in one run dedups correctly ─────────

#[test]
fn same_signal_read_multiple_times_dedupes_subscription() {
    let s: Signal<i32> = Signal::new(0);
    let runs = Arc::new(Mutex::new(0usize));

    let runs2 = runs.clone();
    let s2 = s.clone();
    let _e = effect(move || {
        // Read three times in one run.
        let _ = s2.get();
        let _ = s2.get();
        let _ = s2.get();
        *runs2.lock().unwrap() += 1;
    });
    assert_eq!(*runs.lock().unwrap(), 1);

    // A single write must trigger exactly one re-run, not three.
    s.set(1);
    assert_eq!(*runs.lock().unwrap(), 2);
}

// ── E10. Late set after Disposer::drop must not fire ────────────────────────

#[test]
fn writes_after_dispose_do_not_fire_effect() {
    let s: Signal<i32> = Signal::new(0);
    let runs = Arc::new(Mutex::new(0usize));

    let runs2 = runs.clone();
    let s2 = s.clone();
    let d = effect(move || {
        s2.get();
        *runs2.lock().unwrap() += 1;
    });
    assert_eq!(*runs.lock().unwrap(), 1);

    drop(d);
    s.set(1);
    s.set(2);
    s.set(3);
    assert_eq!(*runs.lock().unwrap(), 1);
}

// ── B2. Disposer drop removes subs proactively (memory) ─────────────────────

/// Sanity: after dropping the effect, the signal should not retain a live
/// weak reference to it. We can't directly inspect Signal internals, but we
/// can verify by writing the signal and checking no effect fires (covered by
/// `writes_after_dispose_do_not_fire_effect`) AND that repeated dispose +
/// recreate cycles don't accumulate. Run a stress loop and confirm we don't
/// panic / OOM / behave unexpectedly.
#[test]
fn dispose_recreate_cycle_does_not_accumulate() {
    let s: Signal<i32> = Signal::new(0);
    for i in 0..1000i32 {
        let s2 = s.clone();
        let d = effect(move || {
            let _ = s2.get();
        });
        drop(d);
        s.set(i);
    }
    // If subs are not proactively pruned, this could grow unbounded.
    // Implicitly verified by reaching this point without OOM.
}
