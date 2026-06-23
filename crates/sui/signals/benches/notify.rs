//! Microbenchmarks for the signal notification hot path.
//!
//! Run with `cargo bench -p sui-signals`. Results land in
//! `target/criterion/`. Aim is characterization, not a regression gate —
//! treat absolute numbers as a baseline; care about ratios.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use criterion::{Criterion, criterion_group, criterion_main};
use sui_signals::{Signal, batch, effect};

/// Fan-out: one signal feeds N effects. Measures cost of a single `set`
/// notifying N subscribers (synchronous dispatch).
fn bench_fanout(c: &mut Criterion) {
    let mut group = c.benchmark_group("fanout");
    for &n in &[1usize, 16, 256, 4096] {
        let s: Signal<u64> = Signal::new(0);
        let counter = Arc::new(AtomicUsize::new(0));

        // Hold disposers so effects stay alive for the whole benchmark.
        let _effects: Vec<_> = (0..n)
            .map(|_| {
                let c = Arc::clone(&counter);
                let s = s.clone();
                effect(move || {
                    let _ = s.get();
                    c.fetch_add(1, Ordering::Relaxed);
                })
            })
            .collect();

        group.bench_with_input(format!("n={n}"), &n, |b, _| {
            let s = s.clone();
            let mut v = 0u64;
            b.iter(|| {
                v = v.wrapping_add(1);
                s.set(v);
            });
        });
    }
    group.finish();
}

/// One effect subscribes to N independent signals. Measures cost of tracking
/// many deps in one run.
fn bench_many_deps(c: &mut Criterion) {
    let mut group = c.benchmark_group("many_deps");
    for &n in &[1usize, 16, 256] {
        let signals: Vec<Signal<u64>> = (0..n).map(|_| Signal::new(0)).collect();
        let counter = Arc::new(AtomicUsize::new(0));

        let c_clone = Arc::clone(&counter);
        let sigs_clone: Vec<_> = signals.to_vec();
        let _e = effect(move || {
            for s in &sigs_clone {
                let _ = s.get();
            }
            c_clone.fetch_add(1, Ordering::Relaxed);
        });

        // Bench: write the FIRST signal repeatedly. Per write, the effect re-runs
        // and re-subscribes to all N deps. Captures the per-run tracking cost.
        let first = signals[0].clone();
        group.bench_with_input(format!("n={n}"), &n, |b, _| {
            let s = first.clone();
            let mut v = 0u64;
            b.iter(|| {
                v = v.wrapping_add(1);
                s.set(v);
            });
        });
    }
    group.finish();
}

/// Batch coalescing: writing N signals inside `batch()` should fire each
/// subscriber once, not N times. Measures `batch()` overhead vs unbatched.
fn bench_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch");

    let a: Signal<u64> = Signal::new(0);
    let b: Signal<u64> = Signal::new(0);
    let cc: Signal<u64> = Signal::new(0);
    let counter = Arc::new(AtomicUsize::new(0));

    let c_clone = Arc::clone(&counter);
    let a2 = a.clone();
    let b2 = b.clone();
    let c2 = cc.clone();
    let _e = effect(move || {
        let _ = a2.get();
        let _ = b2.get();
        let _ = c2.get();
        c_clone.fetch_add(1, Ordering::Relaxed);
    });

    group.bench_function("unbatched_3_writes", |bn| {
        let a = a.clone();
        let b = b.clone();
        let cc = cc.clone();
        let mut v = 0u64;
        bn.iter(|| {
            v = v.wrapping_add(1);
            a.set(v);
            b.set(v);
            cc.set(v);
        });
    });

    group.bench_function("batched_3_writes", |bn| {
        let a = a.clone();
        let b = b.clone();
        let cc = cc.clone();
        let mut v = 0u64;
        bn.iter(|| {
            v = v.wrapping_add(1);
            batch(|| {
                a.set(v);
                b.set(v);
                cc.set(v);
            });
        });
    });

    group.finish();
}

criterion_group!(benches, bench_fanout, bench_many_deps, bench_batch);
criterion_main!(benches);
