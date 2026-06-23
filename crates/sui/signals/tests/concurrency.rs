//! Concurrency tests — verify Send+Sync semantics and multi-task safety.

use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use sui_signals::{Signal, effect};

fn assert_send<T: Send>() {}
fn assert_sync<T: Sync>() {}
fn assert_send_sync<T: Send + Sync>() {}

// Compile-time check: these calls fail to compile if Signal is not Send+Sync.
#[test]
fn signal_bounds_are_send_sync() {
    assert_send::<Signal<i32>>();
    assert_sync::<Signal<i32>>();
    assert_send_sync::<Signal<i32>>();
    assert_send_sync::<Signal<String>>();
    assert_send_sync::<Signal<Vec<u8>>>();
}

// ── Cross-thread signal writes ────────────────────────────────────────────────

#[test]
fn signal_clone_is_send_to_thread() {
    let s: Signal<i32> = Signal::new(0);
    let s2 = s.clone();
    let handle = std::thread::spawn(move || {
        s2.set(42);
    });
    handle.join().unwrap();
    assert_eq!(s.peek(), 42);
}

#[test]
fn multi_thread_writes_no_deadlock() {
    let s: Signal<i32> = Signal::new(0);
    let handles: Vec<_> = (0..8)
        .map(|i| {
            let s = s.clone();
            std::thread::spawn(move || { s.set_unchecked(i); })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }
    // No panic or deadlock. Value is whichever thread won the race.
    let v = s.peek();
    assert!((0..8).contains(&v));
}

#[test]
fn signal_set_from_different_thread_notifies_effect() {
    // Effects run synchronously on the thread that calls set(). This test
    // verifies there's no deadlock when set is called from a spawned thread.
    let s: Signal<i32> = Signal::new(0);
    let received = Arc::new(Mutex::new(Vec::<i32>::new()));

    let r = received.clone();
    let s2 = s.clone();
    let _d = effect(move || {
        let v = s2.get();
        r.lock().unwrap().push(v);
    });
    assert_eq!(*received.lock().unwrap(), vec![0]);

    let handle = std::thread::spawn(move || {
        // Effect fires synchronously here, on this spawned thread.
        s.set(99);
    });
    handle.join().unwrap();

    assert_eq!(*received.lock().unwrap(), vec![0, 99]);
}

// ── Tokio async task send ─────────────────────────────────────────────────────

#[tokio::test]
async fn signal_passes_across_tokio_tasks() {
    let s: Signal<u32> = Signal::new(0);
    let s2 = s.clone();

    let result = tokio::spawn(async move {
        s2.set(7);
        s2.peek()
    })
    .await
    .unwrap();

    assert_eq!(result, 7);
    assert_eq!(s.peek(), 7);
}

#[tokio::test]
async fn signal_used_from_many_tasks_no_panic() {
    let s: Signal<u32> = Signal::new(0);
    let counter = Arc::new(AtomicUsize::new(0));

    let tasks: Vec<_> = (1..=16u32)
        .map(|i| {
            let s = s.clone();
            let c = Arc::clone(&counter);
            tokio::spawn(async move {
                s.set_unchecked(i);
                c.fetch_add(1, Ordering::Relaxed);
            })
        })
        .collect();

    for t in tasks {
        t.await.unwrap();
    }

    assert_eq!(counter.load(Ordering::Relaxed), 16);
    let _ = s.peek(); // alive and readable after all writes
}

#[test]
fn signal_in_arc_accessed_concurrently() {
    let sig = Arc::new(Signal::<i32>::new(0));
    let barrier = Arc::new(std::sync::Barrier::new(4));

    let handles: Vec<_> = (0..4i32)
        .map(|i| {
            let sig = Arc::clone(&sig);
            let barrier = Arc::clone(&barrier);
            std::thread::spawn(move || {
                barrier.wait();
                sig.set_unchecked(i);
                sig.peek()
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }
}
