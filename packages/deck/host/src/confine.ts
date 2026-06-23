//! SES Compartment runner — confined execution of view-controller code.
//!
//! Deck runs views, so Deck owns secure execution (the SUI runtime provides the
//! hardened endowment surface; the author-time `sui-controller-no-ambient-
//! authority` lint is the static first line). A controller is untrusted code;
//! the Compartment is the capability boundary. We endow ONLY the runtime
//! reactive surface + the view's policy-gated `ship.*` ops, and deny everything
//! else (`fetch`, `XMLHttpRequest`, `window`, `process`, intrinsic tampering).
//!
//! Import model: per the SES spike, the compiler's *confined build mode* lowers
//! `import { signal } from '@ship/sui-runtime'` to references resolved from the
//! Compartment's endowed globals — so the controller source we evaluate here
//! reads its runtime from endowments, NOT an ESM import. We never use the
//! Compartment module loader (proven friction under ses@1.x).

import 'ses'

// `lockdown` / `Compartment` / `harden` are installed as realm globals by
// `import 'ses'`. ses ships ambient global types, but declare the shape we use
// so this module type-checks without DOM-lib lockdown typings.
declare const lockdown: (opts?: Record<string, unknown>) => void
declare const Compartment: new (endowments?: Record<string, unknown>) => {
  evaluate(source: string): unknown
}
declare function harden<T>(value: T): T

let realmLocked = false

/**
 * Harden the realm's intrinsics (freezes `Object`/`Array`/… prototypes so
 * confined code can't tamper with shared objects). One-time, process-wide;
 * idempotent. The host calls this once at boot, before confining any
 * controller.
 *
 * NOTE: lockdown is global — it affects ALL code in the realm, including
 * libraries that monkey-patch intrinsics. Roll out behind a flag and verify
 * the editor/3rd-party deps survive. `errorTaming: 'unsafe-debug'` keeps
 * readable stacks in dev; callers may override.
 */
export function lockdownRealm(opts: Record<string, unknown> = { errorTaming: 'unsafe-debug' }): void {
  if (realmLocked) return
  lockdown(opts)
  realmLocked = true
}

/** Whether {@link lockdownRealm} has run. */
export function isRealmLocked(): boolean {
  return realmLocked
}

/**
 * Evaluate confined-built controller `source` in a Compartment whose only
 * reachable bindings are `endowments` — the hardened runtime reactive surface
 * plus the view's policy-gated op set. Ambient authority is unreachable;
 * intrinsics are frozen (when {@link lockdownRealm} has run). Returns the
 * controller's evaluation result (its setup return / default export value).
 *
 * Endowments are `harden`-ed so the controller cannot mutate shared host state
 * through them. Signals created via an endowed `signal` factory live in the
 * host's reactive graph — one graph across the boundary (verified in the spike).
 */
export function confineController<T = unknown>(
  source: string,
  endowments: Record<string, unknown>,
): T {
  const compartment = new Compartment(harden({ ...endowments }))
  return compartment.evaluate(source) as T
}
