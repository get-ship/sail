//! The bound's end-to-end proof: a runaway controller is terminated by the
//! deadline and the SSR render path returns a CLEAN `Err` (not a hang, not a
//! panic). The gate (`deck_core::synthetic_ssr`) catches that `Err` and serves
//! CSR (an empty `#sui-root`, logged loudly) — this is the contract
//! reliability.rs already assumes ("a failure records the precise gap, never a
//! silent exclusion").
//!
//! These tests exercise the real `deck_ssr::run::render_with_limits` entry
//! point with a hand-rolled runaway bundle, so they cover the full chain:
//! `RenderOut` builder → `on_vm_thread` current-thread runtime →
//! `run_module_with_limits` → ship-js isolate → bound fires → `Err` returned
//! to the SSR gate.

use std::sync::Arc;
use std::time::{Duration, Instant};

use deck_ssr::host::SsrDispatch;
use deck_ssr::run::render_with_limits;
use serde_json::{json, Value};
use ship_js::deno::{run_module_with_limits, RunError, RunLimits, TerminateReason};
use ship_js::{Dispatch, DispatchError};

/// A bundle that infinite-loops on a counter, allocating slightly each tick
/// (so V8 checks the terminate flag at statement boundaries). Mirrors the
/// classic runaway-controller failure mode the bound must kill.
const RUNAWAY_BUNDLE: &str = r#"
let i = 0;
while (true) {
    i++;
    if (i % 1000 === 0) {
        // Forced statement boundary — V8 observes the terminate flag here.
        // (Optional: a tiny alloc also forces V8 to check; we keep it lean
        // so the test doesn't hit the default heap limit before the deadline.)
    }
}
"#;

/// A bundle that allocates unboundedly via string concat. Heap cap must fire
/// before the wall clock does (cap = 4 MiB, deadline = 30 s). Uses string
/// concat rather than array push because V8 observes the terminate flag at
/// JS statement boundaries — concat allocates per statement, so the cap
/// + terminate combo settles cleanly. (Array push can bury V8 in native
/// allocations between safe points and trigger the hard-OOM abort.)
const RUNAWAY_HEAP_BUNDLE: &str = r#"
let s = "";
while (true) {
    s += "x".repeat(4096);
}
"#;

/// The shortest end-to-end test: a runaway controller bundle goes through
/// `render_with_limits` with a short deadline; the bound fires and
/// `render_with_limits` returns an `Err` within ~the deadline — proving the
/// daemon survives, the SSR gate falls back, and the request doesn't hang.
#[test]
fn runaway_controller_terminated_within_deadline_does_not_hang_ssr() {
    // 250 ms deadline + 16 MiB heap cap. The deadline is the bound under
    // test; the heap cap is the safety net so a runaway can't push past V8's
    // default limit before the deadline fires. With EITHER bound active, a
    // runaway is killed and the SSR gate falls back.
    let limits = RunLimits {
        deadline: Duration::from_millis(250),
        max_heap_bytes: 16 * 1024 * 1024,
    };
    let dispatch = Arc::new(SsrDispatch::for_request("/runaway", json!({}), json!({})));
    let t0 = Instant::now();
    let r = render_with_limits(RUNAWAY_BUNDLE, dispatch, limits);
    let dt = t0.elapsed();
    // The bound MUST have fired — not hung. Generous upper bound (5 s) for
    // CI noise; in practice the test returns in ~250–500 ms.
    assert!(dt < Duration::from_secs(5), "runaway render hung: {dt:?}");
    // The error must be a `Terminated` — not a normal V8 error and not a
    // panic. The shape `render_with_limits` returns is `anyhow::Error`, so
    // we walk the chain to find the `RunError` source.
    let err = r.expect_err("runaway render must return Err, not Ok");
    let chain = format!("{err:#}");
    let kind = find_run_error_kind(&chain);
    // Either bound may fire first — the deadline is more likely on an
    // in-process loop, the heap cap if allocation outruns the deadline.
    // Both prove the daemon survived; assert SOME `Terminated` rather than
    // picking a winner.
    assert!(
        kind.is_some(),
        "expected a Terminated error, got chain:\n{chain}"
    );
}

/// A runaway heap allocation is killed by the heap cap (not the wall clock).
/// The render path returns `Err` and the cause is heap, not time.
#[test]
fn runaway_heap_allocation_terminated_by_heap_cap() {
    let limits = RunLimits {
        deadline: Duration::from_secs(30),
        max_heap_bytes: 4 * 1024 * 1024,
    };
    let dispatch = Arc::new(SsrDispatch::for_request("/runaway", json!({}), json!({})));
    let t0 = Instant::now();
    let r = render_with_limits(RUNAWAY_HEAP_BUNDLE, dispatch, limits);
    let dt = t0.elapsed();
    assert!(dt < Duration::from_secs(10), "heap-cap render took too long: {dt:?}");
    let err = r.expect_err("runaway heap render must return Err");
    let chain = format!("{err:#}");
    let kind = find_run_error_kind(&chain);
    assert_eq!(
        kind,
        Some(TerminateReason::NearHeapLimit),
        "expected Terminated(NearHeapLimit), got chain:\n{chain}"
    );
}

/// Calling `run_module_with_limits` directly on a runaway script — proves
/// the bound fires even OUTSIDE the deck-ssr render wrapper. (Reliability
/// tests cover the full path; this is the focused engine-level proof.)
#[test]
fn engine_run_module_with_limits_terminates_runaway_script() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let t0 = Instant::now();
    let r = rt.block_on(run_module_with_limits(
        arc_echo(),
        None,
        None,
        "export const noop = 1;".to_string(),
        RUNAWAY_BUNDLE.to_string(),
        RunLimits {
            deadline: Duration::from_millis(250),
            max_heap_bytes: 16 * 1024 * 1024,
        },
    ));
    let dt = t0.elapsed();
    assert!(dt < Duration::from_secs(5), "runaway module hung: {dt:?}");
    match r {
        Err(RunError::Terminated { reason, .. }) => {
            assert!(
                reason == TerminateReason::WallClockDeadline
                    || reason == TerminateReason::NearHeapLimit,
                "expected either bound; got {reason:?}"
            );
        }
        other => panic!("expected Terminated, got {other:?}"),
    }
}

/// Walk the formatted `anyhow::Error` chain to extract the `TerminateReason`
/// embedded by `run_with_deadline`. The chain contains BOTH Display (e.g.
/// "heap cap terminated after ...") AND Debug (e.g. "NearHeapLimit")
/// fragments because render_with_limits wraps `RunError`'s Debug into
/// anyhow's display. Match either form; the wall-clock check uses both
/// "WallClockDeadline" (Debug) and "wall-clock deadline" (Display).
fn find_run_error_kind(chain: &str) -> Option<TerminateReason> {
    if chain.contains("heap cap") || chain.contains("NearHeapLimit") {
        Some(TerminateReason::NearHeapLimit)
    } else if chain.contains("WallClockDeadline") || chain.contains("wall-clock deadline") {
        Some(TerminateReason::WallClockDeadline)
    } else {
        None
    }
}

struct Echo;
#[async_trait::async_trait]
impl Dispatch for Echo {
    async fn call(&self, op: &str, args: Value) -> Result<Value, DispatchError> {
        Ok(json!({ "op": op, "args": args }))
    }
}
fn arc_echo() -> Arc<dyn Dispatch> {
    Arc::new(Echo)
}
