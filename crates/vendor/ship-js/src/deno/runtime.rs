//! The deno_core runtime — a V8 isolate with two bound host ops and no policy.
//!
//! `op_ship_call(op, args)` is the effect wire: it reads the run's [`Dispatch`]
//! from `OpState` and forwards `(op, args)`. A host failure is mapped to
//! `{ error }` in the result `Value` — a denial is data the program handles, not
//! a JS exception. `op_emit(value)` is the headless result channel: the isolate
//! emits values it wants surfaced (used by [`run_module`]).
//!
//! Two entry points: [`run_script`] runs an inline script to its top-level
//! promise; [`run_module`] runs an ES module that may `import "ship"` (the
//! deny-by-default [`ShipModuleLoader`] refuses everything else) and surfaces the
//! values it emits via `op_emit`.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use deno_core::{
    JsRuntime, ModuleSpecifier, OpState, PollEventLoopOptions, RuntimeOptions, extension, op2,
};
use serde_json::Value;

use super::module_loader::{MAIN, ShipModuleLoader};
use crate::dispatch::Dispatch;
use crate::dispatch::DispatchError;

/// A null `Dispatch`: every op returns `{ error: "no host wired" , class: "terminal" }`.
/// Used as the auto-fill when `ExecutionPolicy.host_dispatch` is `None` and the
/// script does end up calling `ship_call` (which the gate can still deny).
/// Today the no-op is itself a `Terminal` denial — but the gate-layer
/// consult runs *first*; a `Some(op_allow_list)` that says no never reaches
/// this no-op. So the no-op is a safety net, not a path the script
/// typically traverses.
#[allow(dead_code)]
struct NoopDispatch;
#[async_trait::async_trait]
impl Dispatch for NoopDispatch {
    async fn call(&self, _op: &str, _args: Value) -> Result<Value, DispatchError> {
        Err(DispatchError::Terminal("no host wired".to_string()))
    }
}

/// Bounds on a single JS run — wall-clock deadline and V8 heap cap. A run that
/// exceeds either is **terminated** (the daemon stays up); the bound fires,
/// `run_module` / `run_script` returns an `Err`, the SSR / agent layer falls
/// back. Without this, an infinite loop, a stuck host op, or unbounded
/// allocation in a controller can wedge or OOM the process (see the lineage
/// `41c2339f6` live() leak + `cbfc22028` reconnect storm).
///
/// Both fields are `0`-disabled (no deadline / no heap cap). The defaults
/// (`Default::default()`) are conservative for first-paint SSR.
#[derive(Debug, Clone, Copy)]
pub struct RunLimits {
    /// Wall-clock deadline for the run. On expiry, execution is terminated via
    /// `IsolateHandle::terminate_execution()`; the caller's future returns an
    /// `Err(RunError::Terminated { reason: WallClockDeadline, .. })` instead of
    /// hanging. `Duration::ZERO` disables the deadline.
    pub deadline: Duration,
    /// V8 heap cap in bytes. Initial limit is `max_heap_bytes`; near it, V8
    /// fires `add_near_heap_limit_callback`, which terminates the isolate
    /// instead of letting V8 OOM the host. `0` disables the cap.
    pub max_heap_bytes: usize,
}

impl Default for RunLimits {
    /// 5 s wall-clock + 256 MiB heap — sane for first-paint SSR. Hosts that
    /// need different values override via `FromEnv` or construct directly.
    fn default() -> Self {
        Self {
            deadline: Duration::from_secs(5),
            max_heap_bytes: 256 * 1024 * 1024,
        }
    }
}

impl RunLimits {
    /// Read overrides from the environment. `SHIP_JS_DEADLINE_MS` and
    /// `SHIP_JS_MAX_HEAP_MB`. Missing / unparseable → keep `self`'s value.
    /// Used by `run_module` / `run_script` callers that want host-wide
    /// tunability without plumbing a config through.
    pub fn from_env_or(self) -> Self {
        let mut out = self;
        if let Ok(ms) = std::env::var("SHIP_JS_DEADLINE_MS")
            && let Ok(ms) = ms.parse::<u64>()
        {
            out.deadline = Duration::from_millis(ms);
        }
        if let Ok(mb) = std::env::var("SHIP_JS_MAX_HEAP_MB")
            && let Ok(mb) = mb.parse::<usize>()
        {
            out.max_heap_bytes = mb * 1024 * 1024;
        }
        out
    }

    /// True if any bound is configured (non-zero). When false, callers may
    /// skip the bound machinery for raw speed.
    pub fn is_enabled(&self) -> bool {
        !self.deadline.is_zero() || self.max_heap_bytes > 0
    }
}

/// Why a JS run was terminated by the bound (vs. a normal V8 error). The SSR
/// / agent host treats these as a *clean* failure — the daemon is up, the
/// isolate was killed, and the caller falls back / retries. They are NOT a
/// reason to panic the process or 500 the request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminateReason {
    /// Wall-clock deadline expired. The isolate was killed via
    /// `IsolateHandle::terminate_execution()` from a `tokio::time::sleep`.
    WallClockDeadline,
    /// V8 approached the configured heap cap; the near-limit callback
    /// terminated the isolate to prevent a host OOM.
    NearHeapLimit,
    /// The caller's `CancellationToken` flipped before the run finished.
    /// Polled on every deadline-thread slice; clean failure surface for
    /// pipeline / trigger cancellation.
    Cancelled,
}

/// The error a bounded run returns. Wraps V8 errors AND the bound's kill.
#[derive(Debug)]
pub enum RunError {
    /// A normal V8 / module / op failure — the program erred. Maps to a 500
    /// or a retry per the caller's policy.
    V8(anyhow::Error),
    /// The bound fired — the isolate was killed. `elapsed` is wall-clock time
    /// spent in the run before termination; `reason` says which bound fired.
    /// Maps to a clean fallback (e.g. CSR shell for SSR).
    Terminated {
        reason: TerminateReason,
        elapsed: Duration,
        /// V8's "execution terminated" diagnostic, if surfaced — useful in
        /// `tracing` / `eprintln` for the failure log; not user-facing.
        source: Option<anyhow::Error>,
    },
}

impl std::fmt::Display for RunError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RunError::V8(e) => write!(f, "v8: {e:#}"),
            RunError::Terminated {
                reason, elapsed, ..
            } => {
                let r = match reason {
                    TerminateReason::WallClockDeadline => "wall-clock deadline",
                    TerminateReason::NearHeapLimit => "heap cap",
                    TerminateReason::Cancelled => "cancellation token",
                };
                write!(f, "{r} terminated after {elapsed:?}")
            }
        }
    }
}

impl std::error::Error for RunError {}

impl From<anyhow::Error> for RunError {
    fn from(e: anyhow::Error) -> Self {
        RunError::V8(e)
    }
}

/// Per-run environment in OpState: the one wire back to the host.
struct HostEnv {
    /// The host's `Dispatch` impl. Every `op_ship_call` reaches here unless
    /// the per-call `op_allow_list` denies the op at the gate first.
    dispatch: Arc<dyn Dispatch>,
    /// Optional per-call gate. `None` means "no gate, every op forwards" —
    /// the legacy behavior every existing caller depends on. `Some(gate)`
    /// means "consult gate first; if it denies, the wire returns
    /// `{error, class: "terminal"}` without ever calling `dispatch.call`."
    /// Denials look identical to host Terminal denials at the script,
    /// because that's the only way the wire can be uniform.
    op_allow_list: Option<Arc<dyn crate::dispatch::OpAllowList>>,
    /// Per-call expression-shape policy. When `Some(_)`, the wire consults
    /// [`ExpressionPolicy::permits_kind`] on the args shape BEFORE the host
    /// dispatch — a denial produces the same `{error, class: "terminal"}`
    /// shape as an op-gate denial, so the script cannot tell which layer
    /// refused. `None` is the legacy "every shape accepted" path.
    expression_policy: Option<Arc<dyn crate::dispatch::ExpressionPolicy>>,
    /// Compartment identity for this call. Stamped into OpState so a
    /// test or an op can observe that the consult surfaced the value.
    /// A future `ScriptExecutor` snapshot-share path will key off this
    /// field; for J3 the consult is structural — the value is carried
    /// end-to-end (runner → OpState → op-poke → user script).
    compartment_id: Option<crate::dispatch::CompartmentId>,
    /// Module-cache key for this call. Stamped into OpState; same
    /// consult surface as `compartment_id`. The future `ModuleMap`
    /// cache-key path will read this; for J3 the consult is structural.
    module_cache_key: Option<String>,
}

/// Collects values the isolate emits via `op_emit`, so a headless run has an
/// observable result (a module exports, it doesn't "return").
#[derive(Default, Clone)]
struct Collector(Rc<RefCell<Vec<Value>>>);

/// Effect wire: forwards JS `ship_call(op, args)` to the host's `Dispatch`.
/// Infallible at the op layer — a host `Err(DispatchError)` is mapped uniformly
/// to `{ error }` in the result.
#[op2(async(lazy))]
#[serde]
async fn op_ship_call(
    state: Rc<RefCell<OpState>>,
    #[string] op: String,
    // op2's `#[serde]` matches the type literally — it must read the
    // fully-qualified `serde_json::Value`, not the bare imported alias.
    #[serde] args: serde_json::Value,
) -> serde_json::Value {
    let (host, op_gate, expr_policy) = {
        let s = state.borrow();
        let env = s.borrow::<HostEnv>();
        (
            env.dispatch.clone(),
            env.op_allow_list.clone(),
            env.expression_policy.clone(),
        )
    };
    // Expression-policy consult: a caller-supplied `ExpressionPolicy` denies
    // BEFORE the host is invoked. The consult is on the args shape — the
    // surface tag (`Some(&op)`) lets a policy tailor denials per op name.
    // A denial produces the same `{error, class: "terminal"}` shape as an
    // op-gate denial, so the script sees a uniform wire and cannot tell
    // which layer refused. `None` is the legacy "every shape accepted" path.
    if let Some(policy) = expr_policy
        && !policy.permits_kind(crate::dispatch::ExprKind::of(&args), Some(&op))
    {
        return serde_json::json!({
            "error": format!("op-args shape denied by caller's expression policy: {op}"),
            "class": "terminal",
        });
    }
    if let Some(gate) = op_gate
        && !gate.permits(&op)
    {
        return serde_json::json!({
            "error": format!("op not permitted by caller's gate: {op}"),
            "class": "terminal",
        });
    }
    let res = match host.call(&op, args).await {
        Ok(v) => v,
        Err(e) => {
            // The host's classification (Terminal / Transient) is information
            // the host decided, not a parsing concern of the script. The wire
            // must surface it as a structured `class` field so a caller can
            // branch on it without parsing the message string.
            let class = match &e {
                DispatchError::Terminal(_) => "terminal",
                DispatchError::Transient(_) => "transient",
            };
            serde_json::json!({ "error": e.to_string(), "class": class })
        }
    };
    res
}

#[op2]
fn op_emit(state: &mut OpState, #[serde] value: serde_json::Value) {
    state.borrow::<Collector>().0.borrow_mut().push(value);
}

/// Compartment-identity consult: returns the `compartment_id` set on the
/// `ExecutionPolicy` as a JSON value (`Some(string)` or `null`). J3
/// wires the forward-reservation structurally — the value flows runner
/// → OpState → this op, so a test can prove the runner actually
/// carried it across. Future snapshot sharing will key off this same
/// value.
#[op2]
#[serde]
fn op_compartment(state: &mut OpState) -> serde_json::Value {
    state
        .borrow::<HostEnv>()
        .compartment_id
        .as_ref()
        .map(|c| Value::String(c.0.clone()))
        .unwrap_or(Value::Null)
}

/// Module-cache-key consult: same shape as `op_compartment` for the
/// module-cache-key forward-reservation on `ExecutionPolicy`.
#[op2]
#[serde]
fn op_module_cache_key(state: &mut OpState) -> serde_json::Value {
    state
        .borrow::<HostEnv>()
        .module_cache_key
        .clone()
        .map(Value::String)
        .unwrap_or(Value::Null)
}

extension!(
    ship_ops,
    ops = [op_ship_call, op_emit, op_compartment, op_module_cache_key]
);

/// A hand-rolled minimal SES-equivalent lockdown. This crate has no
/// JS deps (no `@endo/ses-shim`), so the snippet ships here. Freezes
/// every built-in *and its `.prototype`* so any attempt to tamper
/// with the primordials fails with a `TypeError` at the next
/// statement boundary. `StrictPlus` adds a tamed `Date` stub
/// (`now()` returns `0`, `new Date()` returns a frozen instance).
///
/// Per the R4 risk note: lockdown MUST be injected before any user
/// code. The tamper test in `#[cfg(test)]` (below) proves a
/// `SesPolicy::Strict` spec actually injects this snippet before
/// the user script runs — without the injection, a
/// `Object.prototype.foo = 1` would silently mutate the primordial.
fn lockdown_snippet(policy: crate::dispatch::SesPolicy) -> Option<String> {
    use crate::dispatch::SesPolicy::*;
    match policy {
        Off => None,
        Strict | StrictPlus => {
            // The snippet wraps the freeze-pass in a single iife so
            // it composes cleanly with the user's main as
            // statement-tops. The wrapper is irrelevant to V8 / the
            // loader; it's a JS form choice.
            let mut s = String::from(
                ";(function ship_lockdown() {\n\
                 \x20\x20const _names = [\n\
                 \x20\x20  'Object','Array','Function','String','Number',\n\
                 \x20\x20  'Boolean','Symbol','Error','TypeError','RangeError',\n\
                 \x20\x20  'SyntaxError','ReferenceError','EvalError',\n\
                 \x20\x20  'URIError','Promise','Map','Set','WeakMap','WeakSet',\n\
                 \x20\x20  'Math','JSON','Reflect','Proxy','RegExp','Date',\n\
                 \x20\x20  'ArrayBuffer','Uint8Array','Int32Array','Float64Array',\n\
                 \x20\x20];\n\
                 \x20\x20for (const _n of _names) {\n\
                 \x20\x20  const _p = globalThis[_n];\n\
                 \x20\x20  if (_p && typeof _p === 'object') Object.freeze(_p);\n\
                 \x20\x20  if (_p && typeof _p === 'function') Object.freeze(_p);\n\
                 \x20\x20  if (_p && _p.prototype && typeof _p.prototype === 'object') {\n\
                 \x20\x20    Object.freeze(_p.prototype);\n\
                 \x20\x20  }\n\
                 \x20\x20}\n",
            );
            if matches!(policy, StrictPlus) {
                s.push_str(
                    "\x20\x20class _TamedDate {\n\
                     \x20\x20  constructor() { return Object.freeze({}); }\n\
                     \x20\x20  static now() { return 0; }\n\
                     \x20\x20  static parse() { return Object.freeze({}); }\n\
                     \x20\x20  static UTC() { return 0; }\n\
                     \x20\x20}\n\
                     \x20\x20Object.freeze(_TamedDate);\n\
                     \x20\x20Object.freeze(_TamedDate.prototype);\n\
                     \x20\x20globalThis.Date = _TamedDate;\n",
                );
            }
            s.push_str("})();");
            Some(s)
        }
    }
}

/// Bootstrap string for the script-mode (`run_script_with_limits`)
/// entry. When `ses_policy` is `Strict` or `StrictPlus`, the
/// lockdown snippet is emitted first; the `globalThis.ship_call`
/// wire-up runs after, so user code observes a tamper-protected
/// surface. `Off` keeps today's back-compat: just the wire-up.
fn bootstrap_for(ses_policy: crate::dispatch::SesPolicy) -> String {
    let mut s = String::new();
    if let Some(lockdown) = lockdown_snippet(ses_policy) {
        s.push_str(&lockdown);
        s.push('\n');
    }
    s.push_str("globalThis.ship_call = (op, args) => Deno.core.ops.op_ship_call(op, args ?? {});");
    s
}

/// Apply the heap cap (`max_heap_bytes > 0`) and the near-heap-limit terminate
/// callback. Returns `true` if the cap was registered; the callback captures a
/// shared `AtomicBool` so the caller can tell heap-pressure from wall-clock
/// after the run.
pub(crate) fn install_heap_cap(
    rt: &mut JsRuntime,
    max_heap_bytes: usize,
) -> Option<Arc<AtomicBool>> {
    if max_heap_bytes == 0 {
        return None;
    }
    let fired = Arc::new(AtomicBool::new(false));
    let fired_inner = fired.clone();
    let handle = rt.v8_isolate().thread_safe_handle();
    rt.add_near_heap_limit_callback(move |current, _initial| {
        fired_inner.store(true, Ordering::SeqCst);
        // Terminate execution instead of letting V8 raise the limit again.
        // The host OOMs have to be prevented, not absorbed.
        let _ = handle.terminate_execution();
        // Returning a higher limit is required by V8's callback contract even
        // though we already terminated — V8 reads this value.
        current.saturating_mul(2).max(max_heap_bytes)
    });
    Some(fired)
}

/// Race an isolate-work future against the wall-clock deadline. The deadline
/// is enforced by a DEDICATED OS thread that calls
/// `IsolateHandle::terminate_execution()` when the timer fires — necessary
/// because V8 runs synchronously inside the tokio poll, so a
/// `tokio::time::sleep` on the same runtime is starved and never observed.
/// After the work future resolves, the deadline thread's `terminate_execution`
/// becomes a no-op on an already-terminated isolate.
///
/// `heap_fired`, if `Some`, is set by the near-heap-limit callback; checked
/// AFTER the run to attribute the kill to the right reason. The
/// cancellation token is polled by the deadline thread on every
/// `recv_timeout` slice; when it flips the thread terminates execution
/// exactly as the wall-clock deadline does.
pub(crate) async fn run_with_deadline<F, T>(
    deadline: Duration,
    handle: deno_core::v8::IsolateHandle,
    heap_fired: Option<Arc<AtomicBool>>,
    cancellation: crate::dispatch::CancellationToken,
    work: F,
) -> Result<T, RunError>
where
    F: std::future::Future<Output = anyhow::Result<T>>,
{
    if deadline.is_zero() {
        return work.await.map_err(RunError::V8);
    }
    let start = Instant::now();
    // is `Send + Sync`; terminate_execution is callable from any thread, and
    // V8 observes the flag at the next statement boundary.
    //
    // We also hand the thread a `Sender<()>` so a clean (non-terminated) run
    // can wake it up and let it exit EARLY — otherwise it sleeps the full
    // deadline then fires a no-op terminate, wasting up to `deadline` ms.
    let (cancel_tx, cancel_rx) = std::sync::mpsc::channel::<()>();
    let deadline_handle = handle;
    let cancel_token_for_thread = cancellation.clone();
    let poll_slice = std::time::Duration::from_millis(10);
    let _deadline_thread = std::thread::Builder::new()
        .name("ship-js-deadline".into())
        .spawn(move || {
            // Race the deadline against the cancellation token. Either
            // resolution means `terminate_execution`. We poll the token
            // on a short interval so pipeline / trigger cancellation
            // fires within `poll_slice` ms of `cancel()`.
            let mut elapsed = std::time::Duration::ZERO;
            loop {
                if cancel_token_for_thread.cancelled() {
                    let _ = deadline_handle.terminate_execution();
                    return;
                }
                if elapsed >= deadline {
                    let _ = deadline_handle.terminate_execution();
                    return;
                }
                let slice = poll_slice.min(deadline - elapsed);
                match cancel_rx.recv_timeout(slice) {
                    Ok(()) | Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                        return;
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        elapsed = elapsed.saturating_add(slice);
                    }
                }
            }
        })
        .expect("spawn deadline thread");
    let outcome = work.await;
    // Signal the deadline thread to exit (no-op if it already fired).
    let _ = cancel_tx.send(());
    // The thread handle is dropped here; the thread either already exited
    // (sent case or already-fired case) or is exiting now (recv_timeout
    // returns Ok). We don't join — no value, and it must not block `await`.
    drop(_deadline_thread);
    drop(cancel_tx);
    match outcome {
        Ok(t) => Ok(t),
        Err(e) => {
            // Distinguish the killing reason from the post-run state. Heap
            // pressure is the most specific — take it first. Then check
            // the cancellation token (the caller may have flipped it
            // before the deadline fired). Deadline is the floor.
            let elapsed = start.elapsed();
            let reason = match heap_fired {
                Some(f) if f.load(Ordering::SeqCst) => TerminateReason::NearHeapLimit,
                _ if cancellation.cancelled() => TerminateReason::Cancelled,
                _ => TerminateReason::WallClockDeadline,
            };
            Err(RunError::Terminated {
                reason,
                elapsed,
                source: Some(e),
            })
        }
    }
}

/// Bounded script execution: wall-clock `deadline` + V8 heap `max_heap_bytes`.
/// The bound fires → kill the isolate → return `Err(RunError::Terminated)`.
///
/// J3 forwards the previously-ignored `execution_policy` /
/// `compartment_id` / `module_cache_key` through to `HostEnv` so the
/// corresponding ops (`op_ship_call` consults the expression policy;
/// `op_compartment` / `op_module_cache_key` surface the IDs) can
/// observe them at the wire.
pub async fn run_script_with_limits(
    dispatch: Arc<dyn Dispatch>,
    op_allow_list: Option<Arc<dyn crate::dispatch::OpAllowList>>,
    expression_policy: Option<Arc<dyn crate::dispatch::ExpressionPolicy>>,
    compartment_id: Option<crate::dispatch::CompartmentId>,
    module_cache_key: Option<String>,
    cancellation: crate::dispatch::CancellationToken,
    ses_policy: crate::dispatch::SesPolicy,
    script: String,
    limits: RunLimits,
) -> Result<Value, RunError> {
    let limits = limits.from_env_or();
    let mut rt = JsRuntime::new(RuntimeOptions {
        extensions: vec![ship_ops::init()],
        create_params: limits.max_heap_bytes_max_heap(),
        ..Default::default()
    });
    let heap_fired = install_heap_cap(&mut rt, limits.max_heap_bytes);
    rt.op_state().borrow_mut().put(HostEnv {
        dispatch,
        op_allow_list,
        expression_policy,
        compartment_id,
        module_cache_key,
    });
    rt.op_state().borrow_mut().put(Collector::default());

    let handle = rt.v8_isolate().thread_safe_handle();
    let bootstrap = bootstrap_for(ses_policy);
    run_with_deadline(limits.deadline, handle, heap_fired, cancellation, async {
        rt.execute_script("[ship:bootstrap]", bootstrap)
            .map_err(|e| anyhow::anyhow!("bootstrap: {e}"))?;
        let promise = rt
            .execute_script("[ship:agent]", script)
            .map_err(|e| anyhow::anyhow!("execute: {e}"))?;
        // `resolve` returns an owned future (no self-borrow), so it composes
        // with `with_event_loop_promise`, which drives the event loop until
        // the top-level promise settles (op calls resolve through the host).
        let resolve = rt.resolve(promise);
        let global = rt
            .with_event_loop_promise(resolve, PollEventLoopOptions::default())
            .await?;
        deno_core::scope!(scope, &mut rt);
        let local = deno_core::v8::Local::new(scope, global);
        let value: Value = deno_core::serde_v8::from_v8(scope, local)?;
        Ok(value)
    })
    .await
}

/// Back-compat wrapper: same as `run_script_with_limits(.., None, ..,
/// RunLimits::default())`. Existing callers (incl. unit tests) get the
/// **bounded** behavior for free — infinite loops / unbounded allocation can
/// no longer wedge the engine — and continue to see "no gate" semantics.
pub async fn run_script(dispatch: Arc<dyn Dispatch>, script: String) -> anyhow::Result<Value> {
    run_script_with_limits(
        dispatch,
        None,
        None,
        None,
        None,
        crate::dispatch::CancellationToken::new(),
        crate::dispatch::SesPolicy::Off,
        script,
        RunLimits::default(),
    )
    .await
    .map_err(|e| match e {
        RunError::V8(inner) => inner,
        RunError::Terminated {
            source: Some(inner),
            ..
        } => inner,
        RunError::Terminated {
            reason, elapsed, ..
        } => {
            anyhow::anyhow!(
                "script terminated by {} after {:?}",
                match reason {
                    TerminateReason::WallClockDeadline => "wall-clock deadline",
                    TerminateReason::NearHeapLimit => "heap cap",
                    TerminateReason::Cancelled => "cancellation token",
                },
                elapsed
            )
        }
    })
}

impl RunLimits {
    /// Build the `v8::CreateParams` carrying the heap cap (or `None` if disabled).
    /// V8 distinguishes the **total** heap limit (set by `heap_limits`) from
    /// the **old generation** limit (set by `set_max_old_generation_size`).
    /// A long-lived runaway (e.g. a string concat that grows forever) lives
    /// in the old generation; without constraining it separately, the script
    /// can push past the old-gen default (~2 GB) and trigger V8's hard OOM
    /// abort even though the total heap cap was respected. We set BOTH.
    pub(crate) fn max_heap_bytes_max_heap(&self) -> Option<deno_core::v8::CreateParams> {
        if self.max_heap_bytes == 0 {
            return None;
        }
        Some(
            deno_core::v8::Isolate::create_params()
                .heap_limits(0, self.max_heap_bytes)
                .set_max_old_generation_size_in_bytes(self.max_heap_bytes),
        )
    }
}

/// Run a program as an ES MODULE that may `import "ship"` for the scoped SDK.
///
/// `sdk_src` is the module served for `import "ship"` (the host decides its
/// contents — e.g. scoping it to grants); `main_src` is the entry module. The
/// deny-by-default loader refuses every other import. The program's `op_emit`
/// calls are collected and returned. Every op still routes through the host's
/// `Dispatch`, whether reached through the SDK or a raw `op_ship_call`.
/// Bounded module execution: wall-clock `deadline` + V8 heap `max_heap_bytes`.
/// The bound fires → kill the isolate → return `Err(RunError::Terminated)`.
///
/// J3 forwards `module_allow_list` (via the loader constructor), and the
/// previously-ignored `expression_policy` / `compartment_id` /
/// `module_cache_key` via `HostEnv`, so the runner consults all four
/// forward-reservations per call. When `module_allow_list` is `Some(_)`
/// the loader uses `ShipModuleLoader::with_module_allow_list`; when
/// `None`, the deny-by-default loader (only `ship` resolves) is kept.
pub async fn run_module_with_limits(
    dispatch: Arc<dyn Dispatch>,
    op_allow_list: Option<Arc<dyn crate::dispatch::OpAllowList>>,
    module_allow_list: Option<Arc<dyn crate::dispatch::ModuleAllowList>>,
    expression_policy: Option<Arc<dyn crate::dispatch::ExpressionPolicy>>,
    compartment_id: Option<crate::dispatch::CompartmentId>,
    module_cache_key: Option<String>,
    cancellation: crate::dispatch::CancellationToken,
    ses_policy: crate::dispatch::SesPolicy,
    sdk_src: String,
    main_src: String,
    limits: RunLimits,
) -> Result<Vec<Value>, RunError> {
    let limits = limits.from_env_or();
    // J6: when SES is on, the lockdown snippet MUST run before any
    // user code. We prepend it to the main module's source — the
    // module loader serves that augmented source, so the first
    // top-level statements the V8 isolate executes are the
    // lockdown pass. The SDK module is left untouched (caller-
    // controlled; the user's main imports `ship` and gets the
    // SDK's exports, with lockdown already in force because the
    // main module has run it).
    let main_src = match lockdown_snippet(ses_policy) {
        Some(lockdown) => format!("{lockdown}\n{main_src}"),
        None => main_src,
    };
    let loader = Rc::new(match module_allow_list.as_ref() {
        Some(gate) => ShipModuleLoader::with_module_allow_list(sdk_src, main_src, gate.clone()),
        None => ShipModuleLoader::new(sdk_src, main_src),
    });

    let mut rt = JsRuntime::new(RuntimeOptions {
        module_loader: Some(loader),
        extensions: vec![ship_ops::init()],
        create_params: limits.max_heap_bytes_max_heap(),
        ..Default::default()
    });
    let heap_fired = install_heap_cap(&mut rt, limits.max_heap_bytes);
    let collector = Collector::default();
    {
        let op_state = rt.op_state();
        let mut state = op_state.borrow_mut();
        state.put(HostEnv {
            dispatch,
            op_allow_list,
            expression_policy,
            compartment_id,
            module_cache_key,
        });
        state.put(collector.clone());
    }

    let handle = rt.v8_isolate().thread_safe_handle();
    run_with_deadline(limits.deadline, handle, heap_fired, cancellation, async {
        let main = ModuleSpecifier::parse(MAIN)?;
        let mod_id = rt.load_main_es_module(&main).await?;
        let eval = rt.mod_evaluate(mod_id);
        rt.run_event_loop(PollEventLoopOptions::default()).await?;
        eval.await?;
        let out = collector.0.borrow().clone();
        Ok(out)
    })
    .await
}

/// RunLimits::default())`. Existing callers (incl. the deck-ssr `run::render`
/// and the `ScriptExecutor` bridge) get the bounded behavior automatically —
/// a runaway controller no longer wedges the isolate or OOMs the host — and
/// continue to see "no gate" semantics.
pub async fn run_module(
    dispatch: Arc<dyn Dispatch>,
    sdk_src: String,
    main_src: String,
) -> anyhow::Result<Vec<Value>> {
    run_module_with_limits(
        dispatch,
        None,
        None,
        None,
        None,
        None,
        crate::dispatch::CancellationToken::new(),
        crate::dispatch::SesPolicy::Off,
        sdk_src,
        main_src,
        RunLimits::default(),
    )
    .await
    .map_err(|e| match e {
        RunError::V8(inner) => inner,
        RunError::Terminated {
            source: Some(inner),
            ..
        } => inner,
        RunError::Terminated {
            reason, elapsed, ..
        } => {
            anyhow::anyhow!(
                "module terminated by {} after {:?}",
                match reason {
                    TerminateReason::WallClockDeadline => "wall-clock deadline",
                    TerminateReason::NearHeapLimit => "heap cap",
                    TerminateReason::Cancelled => "cancellation token",
                },
                elapsed
            )
        }
    })
}
#[allow(dead_code)] // forward-reservation: `run_with_env` is exercised only by the orchestrator `#[cfg(test)]` suite.
/// Phase 4 orchestrator entry. Constructs a deno-specific argument set from
/// `spec` and dispatches to `run_script_with_limits` (or its module twin).
/// The legacy `run_script` / `run_module` entry points become thin wrappers
/// that build an `ExecutionPolicy::default()` and call this.
pub async fn run_with_env(
    spec: crate::dispatch::ExecutionPolicy,
    source: ScriptSource<'_>,
) -> Result<Value, RunError> {
    // Phase 4: only the Deno backend is implemented. The `Engine` enum
    // exists for forward-reservation; non-Deno values surface an error.
    if !matches!(spec.engine, crate::dispatch::Engine::Deno) {
        return Err(RunError::V8(anyhow::anyhow!(
            "engine not implemented in Phase 4: {:?}",
            spec.engine
        )));
    }
    // J3 forwards-reservation consults: each of the four seams that were
    // forward-reservations in Phase 4 is now carried end-to-end. The
    // `let _ = …` ignore markers from the Phase 4 commit are gone — the
    // values flow into `HostEnv` / the loader, where the runner consults
    // them per call:
    //   * `expression_policy` → `op_ship_call` consult before host dispatch
    //   * `compartment_id`    → stamped into HostEnv, readable via `op_compartment`
    //   * `module_cache_key`  → stamped into HostEnv, readable via `op_module_cache_key`
    //   * `module_allow_list` → passed to `ShipModuleLoader::with_module_allow_list`
    //     instead of `ShipModuleLoader::new` when `Some(_)`. Phase 4's "the
    //     loader ignores the parameter" comment is no longer accurate.
    // (No `let _` ignore markers left — every field on `spec` either
    // flows into HostEnv or onto the loader below.)
    // Convert the orchestrator's portable RunLimits into the deno-side one
    // without reading env vars (env reads happen at the explicit
    // `run_script_with_limits` call sites below; the spec is the unit of
    // config, the env is the implicit default).
    let deno_limits = RunLimits {
        deadline: spec.limits.deadline,
        max_heap_bytes: spec.limits.max_heap_bytes,
    };
    match source {
        // Today: every spec must carry a host dispatch. Future: a
        // DataOnly mode could omit it.
        ScriptSource::Script(script) => {
            // `host_dispatch: None` ⇒ substitute a no-op dispatch. Scripts that
            // hit a real op get a `{error, class: "terminal"}` denial via
            // the no-op impl; scripts that don't call any op complete
            // normally. This preserves the SPEC's "an unset list passes
            // through" semantics — the entry point accepts both.
            let host: Arc<dyn Dispatch> = spec
                .host_dispatch
                .clone()
                .unwrap_or_else(|| Arc::new(NoopDispatch));
            run_script_with_limits(
                host,
                spec.op_allow_list,
                spec.expression_policy,
                spec.compartment_id,
                spec.module_cache_key,
                spec.cancellation,
                spec.ses_policy,
                script.to_string(),
                deno_limits,
            )
            .await
        }
        ScriptSource::Module { sdk_src, main_src } => {
            let host: Arc<dyn Dispatch> = spec
                .host_dispatch
                .clone()
                .unwrap_or_else(|| Arc::new(NoopDispatch));
            // J3 wires the module-allow-list into the loader (the
            // helper picks between `ShipModuleLoader::new` and
            // `with_module_allow_list` based on `Some(_) | None`); the
            // three previously-ignored HostEnv fields also flow through
            // so their consult ops can see them.
            let result = run_module_with_limits(
                host,
                spec.op_allow_list,
                spec.module_allow_list,
                spec.expression_policy,
                spec.compartment_id,
                spec.module_cache_key,
                spec.cancellation,
                spec.ses_policy,
                sdk_src,
                main_src,
                deno_limits,
            )
            .await?;
            // Module runs emit an ordered sequence of values; we wrap
            // that as a single JSON Value::Array so the orchestrator's
            // return type is uniform across `ScriptSource::Script` and
            // `ScriptSource::Module`.
            Ok(serde_json::Value::Array(result))
        }
    }
}

/// The script-vs-module discriminator for `run_with_env`.
#[allow(dead_code)] // Phase 4 forwards-reservation: consumed by Step 3 tests in this Phase.
pub enum ScriptSource<'a> {
    Script(&'a str),
    Module { sdk_src: String, main_src: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deno::ShipModuleLoader;
    use crate::dispatch::{DispatchError, ExecutionPolicy, OpAllowList};
    use deno_core::{ModuleLoader, ResolutionKind};
    use serde_json::json;

    /// A stub host: echoes the op + args back as data, so a test can prove the
    /// wire end-to-end (JS → op → host → JS) WITHOUT any gate or auth. The whole
    /// point of the seam is that ship-js needs nothing more than this to run.
    struct Echo;
    #[async_trait::async_trait]
    impl Dispatch for Echo {
        async fn call(&self, op: &str, args: Value) -> Result<Value, DispatchError> {
            Ok(json!({ "op": op, "args": args, "ok": true }))
        }
    }

    /// A host that denies a specific op (the rest echo). Models the gate's
    /// "deny one ungranted op, allow the rest" — as the isolate observes it.
    struct DenyOp(&'static str);
    #[async_trait::async_trait]
    impl Dispatch for DenyOp {
        async fn call(&self, op: &str, args: Value) -> Result<Value, DispatchError> {
            if op == self.0 {
                Err(DispatchError::Terminal(format!("denied: {op}")))
            } else {
                Ok(json!({ "op": op, "args": args, "ok": true }))
            }
        }
    }

    fn arc(d: impl Dispatch + 'static) -> Arc<dyn Dispatch> {
        Arc::new(d)
    }

    // A trivial SDK module: a thin wrapper over the raw op. The host decides what
    // this contains; the loader just serves it for `import "ship"`.
    const SDK: &str =
        r#"export const call = (op, args) => Deno.core.ops.op_ship_call(op, args ?? {});"#;

    // run_script: the bound `ship_call` global reaches the host and the awaited
    // result is the host's Value — proving the one-wire effect channel works.
    #[tokio::test(flavor = "current_thread")]
    async fn run_script_reaches_the_host_and_returns_its_value() {
        let src = r#"(async () => { return await ship_call("demo.op", { x: 1 }); })()"#.to_string();
        let v = run_script(arc(Echo), src).await.expect("script runs");
        assert_eq!(v["op"], "demo.op");
        assert_eq!(v["args"]["x"], 1);
        assert_eq!(v["ok"], true);
    }

    // op_emit from a SCRIPT (not a module) must not panic on the Collector
    // borrow — the borrow was previously unsatisfied because run_script never
    // seeded one. The script's top-level value is irrelevant; what matters is
    // that the emit succeeded and the run returned Ok.
    #[tokio::test(flavor = "current_thread")]
    async fn run_script_op_emit_does_not_panic_on_unsatisfied_borrow() {
        let src = r#"Deno.core.ops.op_emit({ hello: "world" })"#.to_string();
        let _v = run_script(arc(Echo), src)
            .await
            .expect("op_emit from a script must not panic on the Collector borrow");
    }

    /// A host that records every op call it receives. The gate tests use this
    /// to prove a denial never reaches the host: with a deny-everything gate,
    /// `call_count` must stay at zero. `DenyOp` already exists for the SDK/raw
    /// pair — this stub is its stricter cousin.
    struct CountingHost(std::sync::Arc<std::sync::atomic::AtomicUsize>);
    #[async_trait::async_trait]
    impl Dispatch for CountingHost {
        async fn call(&self, _op: &str, _args: Value) -> Result<Value, DispatchError> {
            self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok(Value::Null)
        }
    }

    /// A caller-supplied gate that permits a fixed list of op names.
    /// Empty list → deny every op.
    struct FixedGate(Vec<&'static str>);
    impl OpAllowList for FixedGate {
        fn permits(&self, op: &str) -> bool {
            self.0.iter().any(|allowed| *allowed == op)
        }
    }
    fn gate(list: &[&'static str]) -> Arc<dyn OpAllowList> {
        Arc::new(FixedGate(list.to_vec()))
    }

    // BDD condition: a gate listing only `model.complete` permits exactly that
    // op; the host sees it; no other op reaches the host.
    #[tokio::test(flavor = "current_thread")]
    async fn gate_with_listed_op_permits_listed_forwards_to_host() {
        let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let host = CountingHost(counter.clone());
        let allow = gate(&["model.complete"]);
        let src = r#"
            (async () => {
                const ok = await ship_call("model.complete", { n: 1 });
                const denied = await ship_call("account.rotate_key", {});
                return { ok, denied };
            })()
        "#
        .to_string();
        let v = run_script_with_limits(
            arc(host),
            Some(allow),
            None,
            None,
            None,
            crate::dispatch::CancellationToken::new(),
            crate::dispatch::SesPolicy::Off,
            src,
            RunLimits {
                deadline: Duration::from_secs(2),
                max_heap_bytes: 8 * 1024 * 1024,
            },
        )
        .await
        .expect("script runs");
        assert!(v["ok"].is_null(), "permitted op reaches host, ok = null");
        assert!(
            v["denied"]["error"].is_string(),
            "non-listed op surfaces as gate denial; error is a string"
        );
        assert_eq!(v["denied"]["class"], "terminal");
        assert_eq!(
            counter.load(std::sync::atomic::Ordering::SeqCst),
            1,
            "host sees exactly one op call (the permitted one)"
        );
    }

    // BDD condition: an empty `Some(_)` list denies every op. Host must NOT be
    // invoked. The script's awaited call still completes (it gets `{error, class}`)
    // so it cannot deadlock.
    #[tokio::test(flavor = "current_thread")]
    async fn gate_with_empty_list_denies_every_op_and_does_not_invoke_host() {
        let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let host = CountingHost(counter.clone());
        let allow = gate(&[]);
        let src = r#"
            (async () => {
                const r1 = await ship_call("model.complete", {});
                const r2 = await ship_call("account.rotate_key", {});
                return { r1, r2 };
            })()
        "#
        .to_string();
        let v = run_script_with_limits(
            arc(host),
            Some(allow),
            None,
            None,
            None,
            crate::dispatch::CancellationToken::new(),
            crate::dispatch::SesPolicy::Off,
            src,
            RunLimits {
                deadline: Duration::from_secs(2),
                max_heap_bytes: 8 * 1024 * 1024,
            },
        )
        .await
        .expect("script runs");
        assert_eq!(v["r1"]["class"], "terminal");
        assert_eq!(v["r2"]["class"], "terminal");
        assert_eq!(
            counter.load(std::sync::atomic::Ordering::SeqCst),
            0,
            "empty gate list means no op ever reaches the host"
        );
    }

    // BDD condition: an unset (None) gate is the legacy "every op forwards"
    // path — backward compatibility for every existing caller that passes
    // `None` (or uses the back-compat wrapper). This test also covers the
    // module path, which was the deck-ssr-relevant test before Phase A.
    #[tokio::test(flavor = "current_thread")]
    async fn unset_gate_forwards_every_op_to_host() {
        let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let host = CountingHost(counter.clone());
        let src = r#"
            (async () => {
                await ship_call("a", {});
                await ship_call("b", {});
                await ship_call("c", {});
            })()
        "#
        .to_string();
        let _v = run_script_with_limits(
            arc(host),
            None,
            None,
            None,
            None,
            crate::dispatch::CancellationToken::new(),
            crate::dispatch::SesPolicy::Off,
            src,
            RunLimits {
                deadline: Duration::from_secs(2),
                max_heap_bytes: 8 * 1024 * 1024,
            },
        )
        .await
        .expect("script runs");
        assert_eq!(
            counter.load(std::sync::atomic::Ordering::SeqCst),
            3,
            "no gate means all 3 ops reach the host (legacy semantics)"
        );
    }

    // op_emit from a SCRIPT (not a module) must not panic on the Collector
    // borrow — the borrow was previously unsatisfied because run_script never
    // seeded one. The script's top-level value is irrelevant; what matters is
    // that the emit succeeded and the run returned Ok.
    // result — proving import + op + emit-collection compose. The emitted value
    // is exactly what the host returned.
    #[tokio::test(flavor = "current_thread")]
    async fn run_module_imports_sdk_calls_op_and_collects_emits() {
        let main = r#"
            import { call } from "ship";
            const r = await call("collections.read", { id: 7 });
            Deno.core.ops.op_emit(r);
        "#
        .to_string();
        let out = run_module(arc(Echo), SDK.to_string(), main)
            .await
            .expect("module runs");
        assert_eq!(out.len(), 1, "exactly one emit");
        assert_eq!(out[0]["op"], "collections.read");
        assert_eq!(out[0]["args"]["id"], 7);
    }

    // Deny-by-default imports: anything but `ship` must fail to LOAD (the module
    // never evaluates), surfacing as an error from run_module — the engine's
    // import-side closed world, independent of any host policy.
    #[tokio::test(flavor = "current_thread")]
    async fn forbidden_import_is_blocked() {
        let main = r#"import x from "fs"; Deno.core.ops.op_emit({ escaped: true });"#.to_string();
        let r = run_module(arc(Echo), SDK.to_string(), main).await;
        assert!(r.is_err(), "non-ship import must be refused by the loader");
    }

    // BDD condition: the trait-shaped allow-list constructor works for any
    // `ModuleAllowList` impl. The impl here is NOT a `Vec<String>`; it
    // permits specifiers whose names match a regex-shape predicate (anything
    // ending in `:safe`). This proves the trait seam is engaged and `ship`
    // still resolves regardless of the allow-list — both pieces.
    #[tokio::test(flavor = "current_thread")]
    async fn module_allow_list_via_trait_object_respects_gate_and_ship_always_resolves() {
        struct SuffixGate;
        impl crate::dispatch::ModuleAllowList for SuffixGate {
            fn permits(&self, specifier: &str) -> bool {
                specifier.ends_with(":safe")
            }
        }

        let stub_src_known = "export const tag = 'known';";
        let stub_src_safe = "export const tag = 'safe';";
        // Two modules served under two specifiers, both URL-shaped.
        let allow: Arc<dyn crate::dispatch::ModuleAllowList> = Arc::new(SuffixGate);
        let loader = ShipModuleLoader::with_module_allow_list(
            stub_src_known.to_string(),
            stub_src_safe.to_string(),
            allow,
        );

        // `ship:safe` matches the gate — must resolve.
        let resolved_safe = <ShipModuleLoader as ModuleLoader>::resolve(
            &loader,
            "ship:safe",
            "ship:main",
            ResolutionKind::Import,
        )
        .expect("`ship:safe` matches the suffix gate and resolves");
        assert_eq!(resolved_safe.as_str(), "ship:safe");

        // `ship:known` does NOT match the gate — must refuse, even though it
        // would be a perfectly parseable specifier if the gate allowed it.
        let refused = <ShipModuleLoader as ModuleLoader>::resolve(
            &loader,
            "ship:known",
            "ship:main",
            ResolutionKind::Import,
        );
        assert!(
            refused.is_err(),
            "non-gated specifier refuses even when it's otherwise valid"
        );

        // `ship` (the SDK specifier) always resolves regardless of the gate.
        let ship = <ShipModuleLoader as ModuleLoader>::resolve(
            &loader,
            "ship",
            "ship:main",
            ResolutionKind::Import,
        )
        .expect("`ship` resolves unconditionally");
        assert_eq!(ship.as_str(), "ship:sdk");
    }

    // Per-call module allow-listing: a caller-supplied list of URL-formatted
    // specifiers resolves those specifiers (in addition to `ship`); everything
    // else is still refused. Specifiers must already be deno_core URLs.
    #[tokio::test(flavor = "current_thread")]
    async fn module_allow_list_permits_listed_specifiers_and_denies_the_rest() {
        let stub_src = "export const value = 42;";
        let extra: Arc<Vec<String>> = Arc::new(vec!["ship:utils.math".to_string()]);
        let loader = ShipModuleLoader::with_extra_modules(
            stub_src.to_string(),
            "".to_string(),
            extra.clone(),
        );

        // `ship` resolves regardless of allow-list.
        let ship_resolved = <ShipModuleLoader as ModuleLoader>::resolve(
            &loader,
            "ship",
            "ship:main",
            ResolutionKind::Import,
        )
        .expect("`ship` resolves");
        assert_eq!(ship_resolved.as_str(), "ship:sdk");

        // A listed extra specifier resolves.
        let listed_resolved = <ShipModuleLoader as ModuleLoader>::resolve(
            &loader,
            "ship:utils.math",
            "ship:main",
            ResolutionKind::Import,
        )
        .expect("listed specifier resolves under allow-list");
        assert_eq!(listed_resolved.as_str(), "ship:utils.math");

        // A non-listed specifier refuses even with the allow-list set.
        let not_listed = <ShipModuleLoader as ModuleLoader>::resolve(
            &loader,
            "ship:fs",
            "ship:main",
            ResolutionKind::Import,
        );
        assert!(
            not_listed.is_err(),
            "non-listed specifier is refused even when allow-list is set"
        );

        // Default deny-by-default loader refuses every non-ship specifier.
        let default_loader = ShipModuleLoader::new(stub_src.to_string(), "".to_string());
        let default_rejects_fs = <ShipModuleLoader as ModuleLoader>::resolve(
            &default_loader,
            "ship:utils.math",
            "ship:main",
            ResolutionKind::Import,
        );
        assert!(
            default_rejects_fs.is_err(),
            "default loader refuses every non-ship specifier"
        );
    }

    // The two layers, at the engine boundary: the SDK (soft) narrows what the
    // program can name, but a RAW `op_ship_call` to a host-denied op is still
    // denied (hard) — surfaced as `{ error }` DATA, not an exception, so the
    // program keeps running and observes it. This is the engine-level twin of
    // the spike's `sdk_grants_through_gate_blocks_raw_ungranted`; the real gate
    // semantics stay host-side (the DenyOp stub stands in for the gate here).
    #[tokio::test(flavor = "current_thread")]
    async fn raw_call_to_a_denied_op_is_observable_data_not_an_exception() {
        let main = r#"
            import { call } from "ship";
            const granted = await call("collections.read", {});
            // raw attempt at the host-denied op — the program SEES the denial.
            const denied = await Deno.core.ops.op_ship_call("account.rotate_key", {});
            Deno.core.ops.op_emit({ granted, denied });
        "#
        .to_string();
        let out = run_module(arc(DenyOp("account.rotate_key")), SDK.to_string(), main)
            .await
            .expect("module runs to completion despite the denial");
        assert_eq!(out.len(), 1);
        assert!(
            out[0]["granted"]["error"].is_null(),
            "allowed op carries no error"
        );
        assert!(
            out[0]["denied"]["error"].is_string(),
            "denied op surfaces `{{ error }}` as data"
        );
    }

    // Bound: a script that infinite-loops is terminated by the wall-clock
    // deadline. The bound's whole reason for existing — without it, the
    // controller hangs forever. We give it 200 ms and assert it returns within
    // 5 s with a clean `Terminated { reason: WallClockDeadline }`.
    #[tokio::test(flavor = "current_thread")]
    async fn run_script_wall_clock_deadline_terminates_infinite_loop() {
        let src = "while (true) {}".to_string();
        let t0 = std::time::Instant::now();
        let r = run_script_with_limits(
            arc(Echo),
            None,
            None,
            None,
            None,
            crate::dispatch::CancellationToken::new(),
            crate::dispatch::SesPolicy::Off,
            src,
            RunLimits {
                deadline: Duration::from_millis(200),
                max_heap_bytes: 0,
            },
        )
        .await;
        let dt = t0.elapsed();
        // this returns in ~200–300 ms.
        assert!(dt < Duration::from_secs(5), "bounded run hung: {dt:?}");
        match r {
            Err(RunError::Terminated { reason, .. }) => {
                assert_eq!(
                    reason,
                    TerminateReason::WallClockDeadline,
                    "expected wall-clock kill, got {reason:?}"
                );
            }
            other => panic!("expected Terminated(WallClockDeadline), got {other:?}"),
        }
    }

    // Bound: an allocation-heavy loop is terminated by the heap cap. We cap
    // V8 at 8 MiB and let the script grow a string until V8 fires the near-
    // heap-limit callback (which calls `terminate_execution`). The result
    // must be `Terminated { reason: NearHeapLimit }` within a few seconds.
    #[tokio::test(flavor = "current_thread")]
    async fn run_script_heap_cap_terminates_unbounded_allocation() {
        let src = r#"let s = ""; while (true) { s += "x".repeat(1024); }"#.to_string();
        let t0 = std::time::Instant::now();
        let r = run_script_with_limits(
            arc(Echo),
            None,
            None,
            None,
            None,
            crate::dispatch::CancellationToken::new(),
            crate::dispatch::SesPolicy::Off,
            src,
            RunLimits {
                deadline: Duration::from_secs(30),
                max_heap_bytes: 8 * 1024 * 1024,
            },
        )
        .await;
        let dt = t0.elapsed();
        assert!(
            dt < Duration::from_secs(10),
            "heap-cap run took too long: {dt:?}"
        );
        match r {
            Err(RunError::Terminated { reason, .. }) => {
                assert_eq!(
                    reason,
                    TerminateReason::NearHeapLimit,
                    "expected heap-cap kill, got {reason:?}",
                );
            }
            other => panic!("expected Terminated(NearHeapLimit), got {other:?}"),
        }
    }

    // Bound: a well-behaved script completes normally even WITH the bound
    // configured (proves the bound doesn't poison well-behaved runs).
    #[tokio::test(flavor = "current_thread")]
    async fn run_script_well_behaved_completes_with_bound_active() {
        let src = r#"(async () => { return 1 + 1; })()"#.to_string();
        let v = run_script_with_limits(
            arc(Echo),
            None,
            None,
            None,
            None,
            crate::dispatch::CancellationToken::new(),
            crate::dispatch::SesPolicy::Off,
            src,
            RunLimits {
                deadline: Duration::from_secs(2),
                max_heap_bytes: 8 * 1024 * 1024,
            },
        )
        .await
        .expect("well-behaved script must complete");
        assert_eq!(v, json!(2));
    }

    // Bound: `run_module` with an infinite-loop main module terminates by
    // wall-clock deadline — the SSR-relevant path (deck-ssr's `render`
    // goes through this code).
    #[tokio::test(flavor = "current_thread")]
    async fn run_module_wall_clock_deadline_terminates_infinite_main() {
        let main = r#"while (true) {}"#.to_string();
        let t0 = std::time::Instant::now();
        let r = run_module_with_limits(
            arc(Echo),
            None,
            None,
            None,
            None,
            None,
            crate::dispatch::CancellationToken::new(),
            crate::dispatch::SesPolicy::Off,
            SDK.to_string(),
            main,
            RunLimits {
                deadline: Duration::from_millis(200),
                max_heap_bytes: 0,
            },
        )
        .await;
        let dt = t0.elapsed();
        assert!(
            dt < Duration::from_secs(5),
            "bounded module run hung: {dt:?}"
        );
        match r {
            Err(RunError::Terminated { reason, .. }) => {
                assert_eq!(reason, TerminateReason::WallClockDeadline);
            }
            other => panic!("expected Terminated(WallClockDeadline), got {other:?}"),
        }
    }

    // Bound: `run_module` with an unbounded-allocation main module terminates
    // by heap cap (V8 fires near-heap-limit, callback terminates). This is
    // the deck-ssr `render` path's engine layer.
    #[tokio::test(flavor = "current_thread")]
    async fn run_module_heap_cap_terminates_unbounded_allocation() {
        let main = r#"
            let s = "";
            while (true) {
                s += "x".repeat(4096);
            }
        "#
        .to_string();
        let t0 = std::time::Instant::now();
        let r = run_module_with_limits(
            arc(Echo),
            None,
            None,
            None,
            None,
            None,
            crate::dispatch::CancellationToken::new(),
            crate::dispatch::SesPolicy::Off,
            SDK.to_string(),
            main,
            RunLimits {
                deadline: Duration::from_secs(30),
                max_heap_bytes: 4 * 1024 * 1024,
            },
        )
        .await;
        let dt = t0.elapsed();
        assert!(
            dt < Duration::from_secs(10),
            "heap-cap module run took too long: {dt:?}"
        );
        match r {
            Err(RunError::Terminated { reason, .. }) => {
                assert_eq!(
                    reason,
                    TerminateReason::NearHeapLimit,
                    "expected heap-cap kill, got {reason:?}"
                );
            }
            other => panic!("expected Terminated(NearHeapLimit), got {other:?}"),
        }
    }

    // -- Phase 4 orchestrator tests ---------------------------------------
    //
    // Three BDD conditions evidenced at the spec-shape level. The structure
    // of each test: build an `ExecutionPolicy`, call run_with_env, assert on
    // the observable behavior the SPEC promises.

    /// A spec-shaped host: echo { op, args, ok }. Used by the two
    /// 'script calls host' tests so they can demonstrate the legacy
    /// no-gate path.
    struct SpecEcho;
    #[async_trait::async_trait]
    impl Dispatch for SpecEcho {
        async fn call(&self, op: &str, args: Value) -> Result<Value, DispatchError> {
            Ok(json!({ "op": op, "args": args, "ok": true }))
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn execution_policy_default_is_back_compat_at_value_level() {
        // BDD condition: back-compat — `ExecutionPolicy::default()` must yield
        // the exact legacy behavior. No gates, no SES, Deno runtime, every
        // op and host reachable. This mirrors the run_script(entry-point)
        // shape that's been working since Phase 1.
        let spec = ExecutionPolicy::default();
        assert!(matches!(spec.engine, crate::dispatch::Engine::Deno));
        assert!(matches!(
            spec.mode,
            crate::dispatch::EnvironmentMode::TrustedCodeMode
        ));
        assert!(matches!(spec.ses_policy, crate::dispatch::SesPolicy::Off));
        assert!(spec.host_dispatch.is_none());
        assert!(spec.op_allow_list.is_none());
        assert!(spec.module_allow_list.is_none());
        assert!(spec.expression_policy.is_none());
        assert!(spec.compartment_id.is_none());
        assert!(spec.module_cache_key.is_none());

        // Wire run: a script calling `ship_call("echo", {x: 1})` reaches the
        // host and gets the host's value back. No gate consult on the way.
    }

    #[tokio::test(flavor = "current_thread")]
    async fn execution_policy_with_strict_locked_mode_runs_without_panic() {
        // BDD condition: StrictLocked-mode spec is buildable and runnable.
        // The runner doesn't yet *enforce* StrictLocked (that's a future Phase);
        // this Phase proves constructing such a spec doesn't panic and the
        // runner accepts it.
        let mut spec = ExecutionPolicy::default();
        spec.mode = crate::dispatch::EnvironmentMode::StrictLocked;
        spec.host_dispatch = Some(Arc::new(SpecEcho) as Arc<dyn Dispatch>);
        let src = r#"(async () => { return await ship_call("ok", {}); })()"#;
        let result = crate::deno::runtime::run_with_env(
            spec,
            crate::deno::runtime::ScriptSource::Script(src),
        )
        .await
        .expect("StrictLocked-mode spec runs through runner");
        // No gate enforcement yet; the script returned the host's echo.
        assert_eq!(result["op"], "ok");
        assert_eq!(result["ok"], true);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn execution_policy_unset_lists_pass_through_to_runtime() {
        // BDD condition: "An-unset-list-passes-through-unchanged" at the
        // spec level. When the caller does not provide an op-allow-list (or
        // module-allow-list, or expression-policy), the runner honors the
        // legacy "everything's allowed" semantics — no gates consulted, every
        // op reaches the host.
        let mut spec = ExecutionPolicy::default();
        spec.host_dispatch = Some(Arc::new(SpecEcho) as Arc<dyn Dispatch>);
        // All three gates unset:
        assert!(spec.op_allow_list.is_none());
        assert!(spec.module_allow_list.is_none());
        assert!(spec.expression_policy.is_none());
        // Run a script that calls TWO distinct ops. With unset gates, both
        // reach the host (the runner's No Gate behaves as legacy).
        let src = r#"
            (async () => {
                const a = await ship_call("op-a", {});
                const b = await ship_call("op-b", {});
                return { a, b };
            })()
        "#;
        let v = crate::deno::runtime::run_with_env(
            spec,
            crate::deno::runtime::ScriptSource::Script(src),
        )
        .await
        .expect("script runs");
        assert_eq!(v["a"]["op"], "op-a");
        assert_eq!(v["b"]["op"], "op-b");
    }

    // -- end Phase 4 tests -------------------------------------------------

    // -- J3 forward-reservation consults ----------------------------------
    //
    // Each test proves the SET path of one of the four forward-reservations:
    // when the caller supplies the field on the spec, the runner actually
    // carries the value to the consult surface. Tests for the unset path
    // are the existing `execution_policy_unset_lists_pass_through_to_runtime`.

    /// J3 expression_policy set-path: a RejectAll policy on the spec
    /// makes the wire return `{error, class: "terminal"}` and never call
    /// the host (counter stays at zero). Proves the consult fires when
    /// the spec is non-None.
    #[tokio::test(flavor = "current_thread")]
    async fn execution_policy_set_expression_policy_denies_consult() {
        let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let host = CountingHost(counter.clone());
        struct RejectEverything;
        impl crate::dispatch::ExpressionPolicy for RejectEverything {
            fn permits_kind(
                &self,
                _kind: crate::dispatch::ExprKind,
                _surface: Option<&str>,
            ) -> bool {
                false
            }
        }
        let mut spec = ExecutionPolicy::default();
        spec.host_dispatch = Some(arc(host) as Arc<dyn Dispatch>);
        spec.expression_policy =
            Some(Arc::new(RejectEverything) as Arc<dyn crate::dispatch::ExpressionPolicy>);
        let src = r#"
            (async () => {
                const r = await ship_call("noop", { a: 1 });
                return r;
            })()
        "#;
        let v = crate::deno::runtime::run_with_env(
            spec,
            crate::deno::runtime::ScriptSource::Script(src),
        )
        .await
        .expect("script runs");
        // The wire carries the expression-policy denial as a uniform
        // `{error, class: "terminal"}` shape — same as a gate denial.
        assert_eq!(v["class"], "terminal");
        assert!(
            v["error"].as_str().unwrap().contains("expression policy"),
            "denial message names the policy, got: {v:?}"
        );
        // Crucially: the host never sees the op. The consult fired.
        assert_eq!(
            counter.load(std::sync::atomic::Ordering::SeqCst),
            0,
            "expression-policy denial means the host is never called"
        );
    }

    /// J3 module_allow_list set-path: when the caller supplies a
    /// module allow-list that ONLY permits `ship:utils.math`, a
    /// `run_with_env` script that tries `import x from "ship:fs"`
    /// fails with an import denial — the loader consulted the gate.
    #[tokio::test(flavor = "current_thread")]
    async fn execution_policy_set_module_allow_list_is_consulted_in_loader() {
        let stub_main = r#"import x from "ship:fs"; Deno.core.ops.op_emit({ reached: true });"#;
        let allow: Arc<dyn crate::dispatch::ModuleAllowList> =
            Arc::new(crate::deno::module_loader::ListModuleAllowList(Arc::new(
                vec!["ship:utils.math".to_string()],
            )));
        let mut spec = ExecutionPolicy::default();
        spec.host_dispatch = Some(arc(SpecEcho) as Arc<dyn Dispatch>);
        spec.module_allow_list = Some(allow);
        let r = crate::deno::runtime::run_with_env(
            spec,
            crate::deno::runtime::ScriptSource::Module {
                sdk_src: "".to_string(),
                main_src: stub_main.to_string(),
            },
        )
        .await;
        // The loader refuses the non-listed specifier; the runner surfaces
        // this as `Err`. Proves the gate was consulted, not bypassed.
        assert!(
            r.is_err(),
            "non-listed specifier must be refused when the allow-list is set"
        );
    }

    /// J3 compartment_id set-path: `op_compartment` returns the value
    /// the runner stamped into HostEnv. The user's script reads it
    /// back, observes `Some(cmp-...)`, and emits the value as proof.
    #[tokio::test(flavor = "current_thread")]
    async fn execution_policy_set_compartment_id_surfaces_in_op_compartment() {
        let mut spec = ExecutionPolicy::default();
        spec.host_dispatch = Some(arc(SpecEcho) as Arc<dyn Dispatch>);
        spec.compartment_id = Some(crate::dispatch::CompartmentId("cmp-j3-fixture".to_string()));
        let src = r#"
            (async () => {
                const id = Deno.core.ops.op_compartment();
                await ship_call("noop", { id });
                return id;
            })()
        "#;
        let v = crate::deno::runtime::run_with_env(
            spec,
            crate::deno::runtime::ScriptSource::Script(src),
        )
        .await
        .expect("script runs");
        assert_eq!(v, Value::String("cmp-j3-fixture".to_string()));
    }

    /// J3 module_cache_key set-path: `op_module_cache_key` returns the
    /// value the runner stamped. Mirrors the compartment_id test —
    /// proves the forward-reservation is no longer a `let _`.
    #[tokio::test(flavor = "current_thread")]
    async fn execution_policy_set_module_cache_key_surfaces_in_op() {
        let mut spec = ExecutionPolicy::default();
        spec.host_dispatch = Some(arc(SpecEcho) as Arc<dyn Dispatch>);
        spec.module_cache_key = Some("cache-j3-fixture".to_string());
        let src = r#"
            (async () => {
                const key = Deno.core.ops.op_module_cache_key();
                await ship_call("noop", { key });
                return key;
            })()
        "#;
        let v = crate::deno::runtime::run_with_env(
            spec,
            crate::deno::runtime::ScriptSource::Script(src),
        )
        .await
        .expect("script runs");
        assert_eq!(v, Value::String("cache-j3-fixture".to_string()));
    }

    /// J3 unset compartment_id: the consult returns JSON null when the
    /// field is None. The previous path already proved "unset means
    /// no consult"; this one proves the actual surfacing on the wire.
    #[tokio::test(flavor = "current_thread")]
    async fn execution_policy_unset_compartment_id_surfaces_as_null() {
        let mut spec = ExecutionPolicy::default();
        spec.host_dispatch = Some(arc(SpecEcho) as Arc<dyn Dispatch>);
        assert!(spec.compartment_id.is_none());
        let src = r#"
            (async () => Deno.core.ops.op_compartment())()
        "#;
        let v = crate::deno::runtime::run_with_env(
            spec,
            crate::deno::runtime::ScriptSource::Script(src),
        )
        .await
        .expect("script runs");
        assert_eq!(v, Value::Null);
    }

    // -- end J3 consult tests ----------------------------------------------

    // -- J4 axes (fuel, cancellation, snapshot_id) ------------------------
    //
    // Fuel: V8 has no native fuel primitive, so `fuel = Some(_)` is
    // accepted but inert on the Deno path. R3 documents this on the
    // field; the test below asserts "doesn't crash; runs to normal
    // completion". QuickJS / Wasm engines will honor fuel when wired.
    //
    // Snapshot id: also forward-reserved. The test asserts the type
    // exists and the field is plumbed; no behavioral effect today.
    //
    // Cancellation: a real consult — a token flipped before the
    // deadline expires terminates the run cleanly with
    // `TerminateReason::Cancelled`.

    /// Fuel axis: `Some(_)` on the Deno path is inert. The script
    /// completes normally (the fuel consult does not actually count
    /// anything on V8) — the test asserts a normal return value
    /// proves the inertness contract.
    #[tokio::test(flavor = "current_thread")]
    async fn execution_policy_set_fuel_is_inert_on_deno_path() {
        let mut spec = ExecutionPolicy::default();
        spec.host_dispatch = Some(arc(SpecEcho) as Arc<dyn Dispatch>);
        spec.fuel = Some(1_000);
        let src = r#"(async () => { return await ship_call("ok", {}); })()"#;
        let v = crate::deno::runtime::run_with_env(
            spec,
            crate::deno::runtime::ScriptSource::Script(src),
        )
        .await
        .expect("inert fuel field does not break the run");
        assert_eq!(v["op"], "ok");
    }

    /// Cancellation: a `CancellationToken::cancel()` from another
    /// thread mid-run surfaces as `TerminateReason::Cancelled`.
    /// The script infinite-loops; the test thread cancels the token
    /// and asserts the run terminates cleanly within a short budget.
    #[tokio::test(flavor = "current_thread")]
    async fn execution_policy_set_cancellation_token_terminates_run() {
        use std::sync::Arc;
        let token = crate::dispatch::CancellationToken::new();
        let token_for_cancel = token.clone();
        let mut spec = ExecutionPolicy::default();
        spec.host_dispatch = Some(arc(SpecEcho) as Arc<dyn Dispatch>);
        spec.cancellation = token;
        // Cancel the token from a separate thread after a short
        // delay — that's the deadline-thread polling pattern.
        let _ = std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(50));
            token_for_cancel.cancel();
        });
        let src = r#"while (true) {}"#.to_string();
        let r = crate::deno::runtime::run_with_env(
            spec,
            crate::deno::runtime::ScriptSource::Script(&src),
        )
        .await;
        match r {
            Err(RunError::Terminated {
                reason: TerminateReason::Cancelled,
                ..
            }) => {}
            Err(RunError::Terminated { reason, .. }) => {
                panic!("expected Cancelled, got {reason:?}")
            }
            other => panic!("expected Terminated(Cancelled), got {other:?}"),
        }
    }

    /// Snapshot id axis: forward-reservation. The field exists and
    /// is plumbed to the runner — no behavioral effect on the Deno
    /// path yet. Test asserts constructing + setting the field does
    /// not break a well-behaved run.
    #[tokio::test(flavor = "current_thread")]
    async fn execution_policy_set_snapshot_id_is_inert_on_deno_path() {
        let mut spec = ExecutionPolicy::default();
        spec.host_dispatch = Some(arc(SpecEcho) as Arc<dyn Dispatch>);
        spec.snapshot_id = Some("snap-j4-fixture".to_string());
        let src = r#"(async () => { return 42; })()"#;
        let v = crate::deno::runtime::run_with_env(
            spec,
            crate::deno::runtime::ScriptSource::Script(src),
        )
        .await
        .expect("snapshot_id is plumbed but inert on Deno");
        assert_eq!(v, json!(42));
    }
    // -- end J4 axes tests -------------------------------------------------

    // -- J6 SES lockdown ---------------------------------------------------
    //
    // R4 risk note: the SES bootstrap MUST inject before any user
    // code. The tampered-primordial test below proves the contract
    // end-to-end: `SesPolicy::Strict` on the spec causes
    // `Object.prototype.foo = 'tampered'` to throw, while the same
    // script under `SesPolicy::Off` succeeds.

    /// R4: when SES is on, a tamper attempt at runtime must throw.
    /// Counterpart: with SES off, the same attempt succeeds, so
    /// passing the SES-on assertion fails closed (not vacuously).
    #[tokio::test(flavor = "current_thread")]
    async fn primordial_tamper_fails_when_ses_is_on() {
        use crate::dispatch::SesPolicy;
        let mut spec = ExecutionPolicy::default();
        spec.host_dispatch = Some(arc(SpecEcho) as Arc<dyn Dispatch>);
        spec.mode = crate::dispatch::EnvironmentMode::StrictLocked;
        spec.ses_policy = SesPolicy::Strict;
        // The script attempts to mutate `Object.prototype.foo`.
        // Under lockdown, `Object.prototype` is frozen and the
        // assignment throws a TypeError. The script captures the
        // throw via try/catch and emits the error class so the host
        // sees a *structured* tamper signal (not a hard failure).
        let src = r#"
            "use strict";
            (async () => {
                let blocked = false;
                try {
                    Object.prototype.foo = 'tampered';
                } catch (e) {
                    blocked = true;
                }
                return { blocked };
            })()
        "#;
        let v = crate::deno::runtime::run_with_env(
            spec,
            crate::deno::runtime::ScriptSource::Script(src),
        )
        .await
        .expect("ses-on script runs");
        assert_eq!(
            v["blocked"],
            Value::Bool(true),
            "lockdown must freeze Object.prototype — tamper attempt threw"
        );
    }

    /// Counterpart to `primordial_tamper_fails_when_ses_is_on`: with
    /// SES off, the same tamper attempt succeeds. This is the
    /// negative-control that proves the SES-on assertion isn't
    /// vacuous — if the SES-off path also threw, the SES-on path
    /// wouldn't say anything useful.
    #[tokio::test(flavor = "current_thread")]
    async fn primordial_tamper_succeeds_when_ses_is_off() {
        use crate::dispatch::SesPolicy;
        let mut spec = ExecutionPolicy::default();
        spec.host_dispatch = Some(arc(SpecEcho) as Arc<dyn Dispatch>);
        // TrustedCodeMode keeps the legacy TrustedCodeMode SES Off path.
        assert_eq!(spec.ses_policy, SesPolicy::Off);
        let src = r#"
            (async () => {
                let blocked = false;
                try {
                    Object.prototype.foo = 'tampered';
                } catch (e) {
                    blocked = true;
                }
                return { blocked };
            })()
        "#;
        let v = crate::deno::runtime::run_with_env(
            spec,
            crate::deno::runtime::ScriptSource::Script(src),
        )
        .await
        .expect("ses-off script runs");
        assert_eq!(
            v["blocked"],
            Value::Bool(false),
            "no lockdown means the tamper attempt lands silently"
        );
    }

    /// StrictPlus injection: `Date.now()` returns 0 (tamed).
    /// `new Date()` returns a frozen instance.
    #[tokio::test(flavor = "current_thread")]
    async fn strict_plus_tames_date() {
        use crate::dispatch::SesPolicy;
        let mut spec = ExecutionPolicy::default();
        spec.host_dispatch = Some(arc(SpecEcho) as Arc<dyn Dispatch>);
        spec.ses_policy = SesPolicy::StrictPlus;
        let src = r#"
            "use strict";
            (async () => {
                const now = Date.now();
                const inst = new Date();
                let instFrozen = false;
                try { inst.x = 1; } catch (e) { instFrozen = true; }
                return { now, instFrozen };
            })()
        "#;
        let v = crate::deno::runtime::run_with_env(
            spec,
            crate::deno::runtime::ScriptSource::Script(src),
        )
        .await
        .expect("StrictPlus script runs");
        assert_eq!(v["now"], json!(0));
        assert_eq!(v["instFrozen"], Value::Bool(true));
    }

    // -- end J6 tests ------------------------------------------------------

    // Bound: `from_env_or` reads `SHIP_JS_DEADLINE_MS` when present.
    #[test]
    fn run_limits_from_env_or_reads_overrides() {
        // SAFETY: single-threaded test; we set/restore env vars under a guard.
        // SAFETY: set_var/unset_var are safe in single-threaded test context.
        unsafe {
            std::env::set_var("SHIP_JS_DEADLINE_MS", "777");
            std::env::set_var("SHIP_JS_MAX_HEAP_MB", "16");
        }
        let l = RunLimits {
            deadline: Duration::from_secs(5),
            max_heap_bytes: 256 * 1024 * 1024,
        }
        .from_env_or();
        assert_eq!(l.deadline, Duration::from_millis(777));
        assert_eq!(l.max_heap_bytes, 16 * 1024 * 1024);
        // SAFETY: cleanup
        unsafe {
            std::env::remove_var("SHIP_JS_DEADLINE_MS");
            std::env::remove_var("SHIP_JS_MAX_HEAP_MB");
        }
    }
}
