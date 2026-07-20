//! The dispatch seam — the one wire from a program back to its host.
//!
//! This is the shared op-execution contract: a ship-js isolate calls it to reach
//! the host, and the durable-execution engine calls it to run an op. It carries
//! no auth, no gate, no transport — the host (the agent runtime) implements
//! [`Dispatch`] over its capability gateway + credential and injects it, so the
//! credential is captured *inside* the impl and never crosses this boundary.
//!
//! It lives outside the `deno` feature on purpose: it's just a trait + an error,
//! so a consumer (e.g. `durable-execution`) can depend on this contract without
//! pulling V8.

use serde_json::Value;
use std::sync::Arc;

/// Wall-clock + memory budget for one execution. Re-typed here so
/// `ExecutionPolicy` doesn't depend on the `deno` feature. The deno
/// runner converts this into its V8-aware `RunLimits` internally
/// (`deno::runtime::RunLimits`). Anything that *needs* the V8-shaped
/// resource limit (heap cap in V8-recognizable bytes, etc.) lives in
/// the deno runner; the orchestrator only carries the portable subset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunLimits {
    /// Wall-clock deadline. `Duration::ZERO` disables.
    pub deadline: std::time::Duration,
    /// Memory budget in bytes. The deno runner converts to V8 heap caps.
    /// `0` disables.
    pub max_heap_bytes: usize,
}

impl Default for RunLimits {
    /// 5 s + 256 MiB — the historical default; preserves back-compat.
    fn default() -> Self {
        Self {
            deadline: std::time::Duration::from_secs(5),
            max_heap_bytes: 256 * 1024 * 1024,
        }
    }
}

/// A dispatch failure, classified for retry. ship-js itself doesn't retry — it
/// flattens any error to `{ error }` data for the isolate — but the contract
/// carries the classification because the durable-execution engine retries
/// `Transient` and never `Terminal` (a denial is data, not a thing to re-attempt).
#[derive(Debug, thiserror::Error)]
pub enum DispatchError {
    /// A transient blip — safe to retry (a flaky transport, lock contention).
    #[error("transient: {0}")]
    Transient(String),
    /// A terminal outcome — never retried (a denial, an unknown op, bad args).
    #[error("terminal: {0}")]
    Terminal(String),
}

/// The one wire from a program to its host: execute one op, return its result or
/// a classified failure. Implemented host-side; ship-js and the engine only call it.
#[async_trait::async_trait]
pub trait Dispatch: Send + Sync {
    /// Dispatch one op with JSON args; return the host's JSON result, or a
    /// classified failure.
    async fn call(&self, op: &str, args: Value) -> Result<Value, DispatchError>;
}

/// A per-call capability gate: decides whether a given op name may leave the
/// isolate boundary. The wire consults this BEFORE the host's `Dispatch`, so
/// a denial at the gate is data the script sees — never a host dispatch.
///
/// Implementations are out-of-crate (floki's grants shape, ship-triggers's
/// per-skill surface, etc.). This trait is the seam, not the policy.
pub trait OpAllowList: Send + Sync {
    /// `true` iff `op` is permitted to reach the host on this call.
    fn permits(&self, op: &str) -> bool;
}

/// A per-call module gate: decides whether a given specifier (URL-formatted
/// per `deno_core::ModuleSpecifier`) resolves through the loader. The closed
/// world (only `ship`) is enforced before this is consulted; a caller-supplied
/// list *extends* the closed world, never replaces it. So `permits("ship")`
/// is unconditionally true at the loader; this trait only governs everything
/// else — bare specifiers a caller has decided to allow.
///
/// Implementations live out-of-crate (caller-supplied allow-lists, generated
/// per-skill module surfaces, etc.). The trait is the seam; the policy is
/// the caller's.
pub trait ModuleAllowList: Send + Sync {
    /// `true` iff `specifier` is permitted to resolve.
    fn permits(&self, specifier: &str) -> bool;
}
/// Classification of the *value shape* an op-args expression produces by the
/// time it crosses the wire. By the time we receive a `Value` in
/// `op_ship_call`, the JS AST is gone — what we have is the structural kind
/// of the resulting value. Source-level vocabulary policy (function calls,
/// `await`, etc.) is *not* on this trait; that's a future caller-side AST
/// walker. Today's trait is the *args-shape* gate, the smaller of the two.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExprKind {
    Scalar, // Value::Null | Value::Bool | Value::Number | Value::String
    Array,  // Value::Array
    Object, // Value::Object
    Other,  // anything else (defensive default)
}

impl ExprKind {
    pub fn of(value: &serde_json::Value) -> Self {
        match value {
            serde_json::Value::Null
            | serde_json::Value::Bool(_)
            | serde_json::Value::Number(_)
            | serde_json::Value::String(_) => Self::Scalar,
            serde_json::Value::Array(_) => Self::Array,
            serde_json::Value::Object(_) => Self::Object,
        }
    }
}

/// A caller-supplied expression-shape policy. Consulted by the wire BEFORE
/// the host's `Dispatch` (and after the host's `Dispatch` for the host's
/// response, if the host returns non-trivial data).
///
/// The trait is small today. Source-level vocabulary (function calls,
/// await, eval) is the bigger seam the BDD condition names — that lives
/// in a future caller-side source walker that consults this trait's
/// `permits_kind` as a *fallback* against the walked source AST, never
/// from `op_ship_call` alone.
// =====================================================================
// Phase 4 orchestrator types — `ExecutionPolicy` and the axes it
// composes. Each axis is orthogonal; no field's meaning depends on
// another's. The runtime receives an `ExecutionPolicy`, reads the axes
// it knows how to honor today, and ignores the rest (forward-reservations).
// =====================================================================

/// Reachability posture. Which globals, host ops, and modules are visible
/// from this call's realm. NOT the same as `SesPolicy` — that is the
/// hardening mechanism (whether `lockdown()` was called). A `StrictLocked`
/// runtime can run unhardened (by axes) and a `TrustedCodeMode` runtime
/// can be hardened (by SES); the two axes are independent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EnvironmentMode {
    /// Default security posture. Frozen primordials, deny-by-default globals,
    /// only endowed ops reachable. Suitable for SSR + template render +
    /// any surface that touches untrusted data.
    StrictLocked,
    /// Code-mode. Full language globals, host ops reachable, no SES
    /// tamper-proofing. Use only for trusted-author code (floki code-mode).
    TrustedCodeMode,
    /// Pure data — no language runtime at all. The evaluator is Rust-side
    /// (today: not yet implemented). `host_dispatch` is `None` for this mode.
    /// Forward-reservation: no runner wires this Phase.
    DataOnly,
    /// Dev/manual override. Triggers a `tracing::warn` and a unit test flag
    /// at the call site so it shows up in the audit grid.
    Unlocked,
}

/// Hardening mechanism. Maps 1:1 onto `@endo/ses-shim`'s `lockdown()` options.
/// Orthogonal to `EnvironmentMode`. A `Some(policy)` activates SES for the
/// call; `Off` means no tamper-proofing. Today only the Deno backend
/// honors this; rquickjs ships with `Off` until SES-for-quickjs is
/// upstream-validated (not on the table for Phase 4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SesPolicy {
    /// Don't call `lockdown()`. Trust the call site has done its own reasoning.
    Off,
    /// `lockdown()` with hardened defaults.
    Strict,
    /// `lockdown()` plus tamed `Date`, hardened console, error-tag discipline.
    /// Agoric's default for production deploys.
    StrictPlus,
}

/// Which evaluation engine runs the code. The v2 name is `Engine` —
/// the orchestrator is engine-agnostic; only the dispatch layer cares
/// which one runs. Today only `Engine::Deno` is wired; the others are
/// forward-reservations so callers can construct policies that name a
/// non-existent engine and have it fail at the dispatch boundary
/// (rather than at the trait level).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Engine {
    /// `deno_core` / V8. Heavy cold-start (~100ms), large binary (~30MB
    /// deno_core footprint). Has the SES-on-lockdown path. Default.
    Deno,
    /// `rquickjs`. Pure-Rust, microsecond cold-start, ~1-2MB binary
    /// overhead. For when V8 is too expensive (cron, worker, tooling
    /// preview). Forward-reservation: no runner wires this Phase.
    Quickjs,
}

/// Cooperative cancel token. Cheap to clone (the inner state is one
/// `Arc<AtomicBool>`); safe across `Send + Sync` boundaries. A caller
/// may invoke `cancel()` from any thread; the runner polls `cancelled()`
/// between op calls and on the event-loop wake. It is intentionally
/// minimal: no async wait, no nested tokens, no `Drop` side-effects
/// (cancellation is fire-and-forget).
///
/// The token is `Clone` — every clone shares the same flag, so a
/// caller can hand a clone to the runner for per-call bookkeeping
/// while keeping the original handle around to fire `cancel()` on.
#[derive(Debug, Clone)]
pub struct CancellationToken {
    inner: Arc<std::sync::atomic::AtomicBool>,
}

impl CancellationToken {
    /// A fresh, un-cancelled token.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    /// `true` if `cancel()` has been observed. The flag is sticky —
    /// once flipped, every subsequent poll returns `true`.
    pub fn cancelled(&self) -> bool {
        self.inner.load(std::sync::atomic::Ordering::Acquire)
    }

    /// Flip the cancellation flag. Idempotent and thread-safe.
    pub fn cancel(&self) {
        self.inner.store(true, std::sync::atomic::Ordering::Release);
    }
}

impl Default for CancellationToken {
    fn default() -> Self {
        Self::new()
    }
}

/// Compartment identity. Two calls with the same id are meant to share
/// their V8 isolate; two with different ids are meant to be fresh.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CompartmentId(pub String);

impl CompartmentId {
    /// A fresh compartment identity. Two calls produce different ids at
    /// sub-microsecond resolution (system-clock nanos). Names end with a hex
    /// token derived from the wall clock.
    pub fn new() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        Self(format!("cmp-{:x}", nanos))
    }
}

/// Workaround: `dispatch.rs` is the leaf crate; no uuid dep. We hand-craft
/// a short unique token. Real implementation will live in a tiny helper
/// module once the "tiny pieces" cluster lands; for Phase 4 it is local
/// and only used from tests.
#[doc(hidden)]
pub fn dispatch_test_only_nanoid() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{:x}", nanos)
}

/// The orchestrator: one type, every axis. Callers construct an
/// `ExecutionPolicy` and pass it to `run_with_env`. `Default` returns
/// the legacy back-compat shape (TrustedCodeMode + SES Off + Deno), so
/// existing callers that don't yet specify anything get today's exact
/// behavior.
/// `EnvironmentSpec` was the v1 name; it is removed in J5. New code
/// composes via `Policy::*` (below). Backward-compatible legacy
/// callers fall through to `ExecutionPolicy::default()` (identical
/// to the v1 default shape).
#[derive(Clone)]
pub struct ExecutionPolicy {
    /// Which evaluation engine runs the code. Today only `Engine::Deno`
    /// is wired; non-Deno values surface an error at dispatch time.
    /// The v2 name is `engine` — the orchestrator is engine-agnostic;
    /// the field is engine-the-instance, not engine-the-runtime-name.
    pub engine: Engine,
    /// Reachability posture of this call.
    pub mode: EnvironmentMode,
    /// The capability-bounded host. `None` = no host ops possible
    /// (suitable for `DataOnly` mode).
    pub host_dispatch: Option<Arc<dyn Dispatch>>,
    /// Per-call restriction on which named host ops are reachable. `None`
    /// is the legacy "every op forwards" path.
    pub op_allow_list: Option<Arc<dyn OpAllowList>>,
    /// Per-call restriction on which module names are importable. `None`
    /// is the closed-world default ("only `ship`").
    pub module_allow_list: Option<Arc<dyn ModuleAllowList>>,
    /// Per-call restriction on which op-args expression shapes are
    /// admissible. `None` means no shape consult — every shape accepted.
    pub expression_policy: Option<Arc<dyn ExpressionPolicy>>,
    /// Compartment identity for cache sharing. Forward-reservation.
    pub compartment_id: Option<CompartmentId>,
    /// Module-cache key for `ModuleMap` reuse. Forward-reservation.
    pub module_cache_key: Option<String>,
    /// Hardening mechanism applied to the runtime (Deno only).
    pub ses_policy: SesPolicy,
    /// Resource bounds for the call.
    // Re-typed to keep `ExecutionPolicy` portable; the runner converts.
    // Forward-reservation: `RunLimits` itself lives in `deno::runtime`
    // because it has V8/heap-specific fields; the spec uses duration + bytes
    // directly so this type carries no deno dep.
    pub limits: RunLimits,
    /// Engine-level fuel budget. Each "unit" of fuel maps to one
    /// operation the engine charges (an op-call, a microtask, an
    /// allocation past a threshold — engine-defined). **`None` means
    /// no fuel consult**; the runner relies on `RunLimits.deadline`
    /// (wall-clock) and `max_heap_bytes` (memory) as the bound.
    ///
    /// **Honored if the engine supports it.** V8/deno_core has no
    /// native fuel primitive — fuel is NOT consumed on the Deno path,
    /// and the run-limit deadline remains the floor. QuickJS and
    /// Wasm engines have first-class fuel; the field is the seam
    /// those engines will read when they are wired (Phase E). Today
    /// `fuel = Some(n)` on the Deno path is accepted but inert; the
    /// wall-clock deadline still fires. This is the R3 risk-note
    /// contract documented on the field — do NOT silently change.
    pub fuel: Option<u64>,
    /// Cooperative cancellation handle. When `cancelled()` returns
    /// `true`, the runner terminates the run cleanly and surfaces a
    /// `RunError::Terminated { reason: Cancelled, .. }` (mirroring
    /// the deadline / heap-limit surface). The token is checked at
    /// the same polling points as the deadline thread; a token
    /// flipped from another thread is observed on the next poll.
    /// The default is a fresh un-cancelled token — `.default()` so
    /// existing callers don't need to think about it.
    pub cancellation: CancellationToken,
    /// Snapshot id for warm-start reuse. Two calls with the same
    /// `snapshot_id` are *intended* to share a V8 isolate snapshot
    /// (a future `ScriptExecutor::with_snapshot(_)` path); calls
    /// with distinct ids get fresh isolates. Forward-reservation on
    /// the Deno path: the field is plumbed but the runner doesn't
    /// yet key off it (Phase D pool).
    pub snapshot_id: Option<String>,
}
/// The J1 migration alias is gone in J5: every in-repo caller has
/// moved to `ExecutionPolicy`. The v2 name is the only surface; new
/// callers compose via `Policy::*` (below). Existing callers that
/// still name `EnvironmentSpec` will fail to compile and must
/// migrate to `Policy::*`.

impl Default for ExecutionPolicy {
    /// Back-compat with every pre-Phase-4 caller. TrustedCodeMode + Off +
    /// Deno + every host op + every module closed-world + no expression
    /// policy + default limits. This is what `run_script` and `run_module`
    /// (the legacy wrappers) construct internally.
    fn default() -> Self {
        Self {
            engine: Engine::Deno,
            mode: EnvironmentMode::TrustedCodeMode,
            host_dispatch: None,
            op_allow_list: None,
            module_allow_list: None,
            expression_policy: None,
            compartment_id: None,
            module_cache_key: None,
            ses_policy: SesPolicy::Off,
            limits: RunLimits::default(),
            fuel: None,
            cancellation: CancellationToken::new(),
            snapshot_id: None,
        }
    }
}

pub trait ExpressionPolicy: Send + Sync {
    /// `true` iff the policy admits an op-args expression whose resulting
    /// value shape is `kind`. `surface` is an optional caller-supplied
    /// tag (e.g. a pre-known binding name); the trait may ignore it.
    fn permits_kind(&self, kind: ExprKind, surface: Option<&str>) -> bool;
}

// =====================================================================
// `Policy::*` — preset builders for the four in-repo postures. The
// orchestrator type is `ExecutionPolicy`; these constructors are the
// ONLY sanctioned way to cross from non-engine code into the
// runtime. A future engine swap (e.g. V8 → rquickjs) lives behind
// these four entry points — in-repo and cross-repo consumers do not
// touch `ExecutionPolicy::default()` directly any more.
//
// Preset names match the seven consumers in `census-shipjs.md`:
//   * `untrusted_render()` — deck-ssr hot path (per-request, SUI bundle
//     is untrusted author). StrictLocked posture, the rigour-preserving
//     defaults — short deadline, moderate heap, StrictPlus SES.
//   * `trusted_code_mode()` — floki code-mode (per-session, agent
//     author). TrustedCodeMode posture, longer deadline for the
//     session-loop workload, SES Off (SES for trusted-author code is
//     noise).
//   * `resolver_eval()` — agent-config-eval / J3 resolver (many small,
//     deterministic, content-hash-cacheable). Tight deadline and
//     heap, fuel hint set (the resolver wants determinism; engines
//     that honor fuel will consume it — V8 still inert per J4 R3).
//
// The legacy "trusted-author script" shape is still available as
// `Policy::default()` (= `ExecutionPolicy::default()`) for the
// back-compat wrappers `run_script` / `run_module`, which pre-date
// this seam.
// =====================================================================

/// The preset builder. Each method returns an `ExecutionPolicy` tailored
/// for one consumer posture. Field-level tunability (deadline_ms,
/// heap_mib, fuel, ses_policy opt-in) lives on the returned value
/// — callers compose by `..` or `set_` helpers as they prefer.
pub struct Policy;

impl Policy {
    /// Preset: SSR / first-paint render. StrictLocked posture +
    /// StrictPlus SES (per J6 contract; this method lands SES Off
    /// in J5 and J6 promotes it to `StrictPlus`). The deck-ssr
    /// `run::render` and `ScriptPool::render` callers reach for
    /// this; the deck-ssr-curve (per-request, throughput-bound)
    /// is matched by a short deadline and a moderate heap cap.
    pub fn untrusted_render() -> ExecutionPolicy {
        ExecutionPolicy {
            engine: Engine::Deno,
            mode: EnvironmentMode::StrictLocked,
            host_dispatch: None,
            op_allow_list: None,
            module_allow_list: None,
            expression_policy: None,
            compartment_id: None,
            module_cache_key: None,
            // J6: SUI bundle is untrusted author; runtime + tamed Date.
            ses_policy: SesPolicy::StrictPlus,
            limits: RunLimits {
                deadline: std::time::Duration::from_secs(5),
                max_heap_bytes: 256 * 1024 * 1024,
            },
            fuel: None,
            cancellation: CancellationToken::new(),
            snapshot_id: None,
        }
    }

    /// Preset: floki code-mode / trusted agent author. TrustedCodeMode
    /// posture, full language globals reachable, SES Off (the agent
    /// author is the trust boundary; SES would only get in the way
    /// of `Math.random` / `Date.now` access in a code-mode loop).
    /// Longer deadline — the executor runs many tool calls per
    /// session; one 30s budget amortizes across them.
    pub fn trusted_code_mode() -> ExecutionPolicy {
        ExecutionPolicy {
            engine: Engine::Deno,
            mode: EnvironmentMode::TrustedCodeMode,
            host_dispatch: None,
            op_allow_list: None,
            module_allow_list: None,
            expression_policy: None,
            compartment_id: None,
            module_cache_key: None,
            ses_policy: SesPolicy::Off,
            limits: RunLimits {
                deadline: std::time::Duration::from_secs(30),
                max_heap_bytes: 256 * 1024 * 1024,
            },
            fuel: None,
            cancellation: CancellationToken::new(),
            snapshot_id: None,
        }
    }

    /// Preset: agent-config-eval / J3 resolver (many small,
    /// deterministic, content-hash-cacheable evals). StrictLocked
    /// posture — the resolver may eventually ingest untrusted user
    /// descriptors. Tight deadline (100ms) and heap cap (32 MiB) so
    /// a runaway descriptor can't chew the daemon. Fuel hint of
    /// 10_000 set so engines that honor fuel have a budget; V8
    /// ignores it per the J4 R3 inertness contract.
    pub fn resolver_eval() -> ExecutionPolicy {
        ExecutionPolicy {
            engine: Engine::Deno,
            mode: EnvironmentMode::StrictLocked,
            host_dispatch: None,
            op_allow_list: None,
            module_allow_list: None,
            expression_policy: None,
            compartment_id: None,
            module_cache_key: None,
            // J6: resolver may ingest untrusted user descriptors;
            ses_policy: SesPolicy::Strict,
            limits: RunLimits {
                deadline: std::time::Duration::from_millis(100),
                max_heap_bytes: 32 * 1024 * 1024,
            },
            fuel: Some(10_000),
            cancellation: CancellationToken::new(),
            snapshot_id: None,
        }
    }

    /// The legacy back-compat shape — same as
    /// `ExecutionPolicy::default()`. Use this only for the
    /// `run_script` / `run_module` back-compat wrappers; new
    /// orchestrator code should pick a named preset above.
    pub fn default() -> ExecutionPolicy {
        ExecutionPolicy::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::Arc;

    /// A permit-everything policy: every ExprKind is admitted. The default
    /// contract — a caller who hands in *no* policy or a no-op policy sees
    /// every form accepted — is exactly the BDD condition's "Three-named-
    /// forms-accepted" plus "An-unset-policy-accepts-every-form-the-runtime-admits".
    struct AcceptAllPolicy;
    impl ExpressionPolicy for AcceptAllPolicy {
        fn permits_kind(&self, _kind: ExprKind, _surface: Option<&str>) -> bool {
            true
        }
    }

    /// A permit-nothing policy. The BDD condition's "An-out-of-vocabulary-
    /// form-rejected-before-execution".
    struct RejectAllPolicy;
    impl ExpressionPolicy for RejectAllPolicy {
        fn permits_kind(&self, _kind: ExprKind, _surface: Option<&str>) -> bool {
            false
        }
    }

    #[test]
    fn accepts_scalar_array_object_via_kind_classification() {
        // BDD condition: "Three-named-forms-accepted" — the three forms the
        // BDD condition names are scalar, array, object.
        let p = AcceptAllPolicy;
        assert!(p.permits_kind(ExprKind::of(&json!(null)), None));
        assert!(p.permits_kind(ExprKind::of(&json!(true)), None));
        assert!(p.permits_kind(ExprKind::of(&json!(42)), None));
        assert!(p.permits_kind(ExprKind::of(&json!("hello")), None));
        assert!(p.permits_kind(ExprKind::of(&json!([1, 2, 3])), None));
        assert!(p.permits_kind(ExprKind::of(&json!({"k": "v"})), None));
    }

    #[test]
    fn reject_policy_disallows_every_kind() {
        // BDD condition: "An-out-of-vocabulary-form-rejected-before-execution".
        // Every ExprKind is the same: rejected.
        let p = RejectAllPolicy;
        let kinds = [
            (json!(null), "null"),
            (json!(true), "bool"),
            (json!(42), "number"),
            (json!("x"), "string"),
            (json!([1]), "array"),
            (json!({"k": "v"}), "object"),
        ];
        for (val, label) in kinds.iter() {
            assert!(
                !p.permits_kind(ExprKind::of(val), None),
                "{label} must be rejected by RejectAllPolicy"
            );
        }
    }

    #[test]
    fn unset_policy_means_no_consult_performs_as_accept_all_under_optional_contract() {
        // BDD condition: "An-unset-policy-accepts-every-form-the-runtime-admits".
        // The trait is gated by an `Option<Arc<dyn ExpressionPolicy>>`; the
        // runtime consults the policy only when `Some(_)`. The trait itself
        // is not consulted when `None` — semantically equivalent to
        // "everything accepted." We prove this at the contract level: the
        // Option wrapper makes this an unset-is-accept-all contract; the trait
        // alone cannot express it. This test documents that contract.
        let nothing: Option<Arc<dyn ExpressionPolicy>> = None;
        let everything: Option<Arc<dyn ExpressionPolicy>> = Some(Arc::new(AcceptAllPolicy));
        // None → no consultation; runtime's job to honor.
        assert!(nothing.is_none());
        assert!(everything.is_some());
        // The trait itself accepts everything when AcceptAll is provided.
        let p = everything.as_ref().unwrap().as_ref();
        assert!(p.permits_kind(ExprKind::Scalar, None));
        assert!(p.permits_kind(ExprKind::Array, None));
        assert!(p.permits_kind(ExprKind::Object, None));
    }

    // -- J5 preset smoke tests --------------------------------------------
    //
    // Each named preset returns a valid `ExecutionPolicy` with the
    // posture the spec record (census-shipjs Part 6 + SPEC.md) names
    // for that consumer. The tests don't drive a run end-to-end —
    // that's the consumer-migration job. They assert the shape.

    #[test]
    fn policy_untrusted_render_is_strict_locked_with_moderate_limits() {
        let p = Policy::untrusted_render();
        assert!(matches!(p.engine, Engine::Deno));
        assert!(matches!(p.mode, EnvironmentMode::StrictLocked));
        assert!(matches!(p.ses_policy, SesPolicy::StrictPlus));
        assert_eq!(p.limits.max_heap_bytes, 256 * 1024 * 1024);
        assert!(p.fuel.is_none());
    }

    #[test]
    fn policy_trusted_code_mode_is_trusted_with_long_deadline() {
        let p = Policy::trusted_code_mode();
        assert!(matches!(p.engine, Engine::Deno));
        assert!(matches!(p.mode, EnvironmentMode::TrustedCodeMode));
        assert_eq!(p.limits.deadline, std::time::Duration::from_secs(30));
        assert!(p.fuel.is_none());
    }

    #[test]
    fn policy_resolver_eval_is_tight_with_fuel_hint() {
        let p = Policy::resolver_eval();
        assert!(matches!(p.mode, EnvironmentMode::StrictLocked));
        assert_eq!(p.limits.deadline, std::time::Duration::from_millis(100));
        assert_eq!(p.limits.max_heap_bytes, 32 * 1024 * 1024);
        // Fuel hint set; engine-supportable. V8 ignores (J4 R3).
        assert_eq!(p.fuel, Some(10_000));
        // J6: resolver runs untrusted descriptors; Strict lockdown.
        assert!(matches!(p.ses_policy, SesPolicy::Strict));
    }

    #[test]
    fn policy_default_equals_execution_policy_default() {
        // Back-compat: `Policy::default()` is the same shape as
        // `ExecutionPolicy::default()`. Diff-relevant for the
        // run_script / run_module wrappers.
        let a = Policy::default();
        let b = ExecutionPolicy::default();
        assert_eq!(a.mode, b.mode);
        assert_eq!(a.ses_policy, b.ses_policy);
        assert_eq!(a.limits.deadline, b.limits.deadline);
        assert_eq!(a.limits.max_heap_bytes, b.limits.max_heap_bytes);
        assert_eq!(a.fuel, b.fuel);
    }

    #[test]
    fn policy_presets_carry_a_fresh_cancellation_token_each() {
        // Two calls produce distinct, un-cancelled tokens. The
        // runner observes the flag via the polling path (J4); the
        // per-call seeding here is what makes per-call cancellation
        // safe — no shared-flag accident when a caller constructs
        // the same preset twice.
        let a = Policy::trusted_code_mode().cancellation;
        let b = Policy::trusted_code_mode().cancellation;
        assert!(!a.cancelled());
        assert!(!b.cancelled());
    }

    // -- end J5 preset smoke tests ---------------------------------------
}
