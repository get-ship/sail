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
//! `import { signal } from '@ship/sail-runtime'` to references resolved from the
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

// ── network-capability fetch wrapper ────────────────────────────────────────
//
// The static lint (`sui-controller-no-ambient-authority`) checks a fetch
// call's STRING-LITERAL target against a package's resolved `sail.config.json`
// `"network"` allowlist at author time. A dynamic target (a variable, a
// template with interpolation) can't be verified statically — this wrapper is
// the ACTUAL runtime wall for that case, and the one enforcement point full
// stop for anything the Compartment endows. Never endow the bare global
// `fetch`; endow this instead (`view-runner.ts`'s `buildViewEndowments` does).

/** Extract the `scheme://host[:port]` authority from an absolute URL-ish
 *  string. `null` for a relative/schemeless string (`/api/x`, `./x`) — those
 *  are same-origin, matched against `"self"` instead. Mirrors
 *  `crates/sui/lint/src/rules/controller_no_ambient_authority.rs`'s
 *  `authority_of` so the static lint and this runtime wall agree on what
 *  "the same origin" means. */
function authorityOf(url: string): string | null {
  const m = /^([a-zA-Z][a-zA-Z0-9+.-]*):\/\/([^/?#]+)/.exec(url)
  return m ? `${m[1]}://${m[2]}` : null
}

/** Whether `target` clears the `network` per-origin allowlist — same rule as
 *  the lint's `literal_target_allowed`: an absolute target must authority-
 *  match an allowlist entry exactly; a relative target is allowed only when
 *  `"self"` is in the list. */
function targetAllowed(target: string, network: readonly string[]): boolean {
  const authority = authorityOf(target)
  if (authority === null) return network.includes('self')
  return network.some((entry) => authorityOf(entry) === authority)
}

/** The URL string a `fetch` call's first argument targets, across all three
 *  shapes `fetch` accepts. */
function requestTarget(input: RequestInfo | URL): string {
  if (typeof input === 'string') return input
  if (input instanceof URL) return input.href
  return input.url
}

/**
 * Build a confined `fetch` — a WRAPPER, never the bare global — that checks
 * each call's target origin against `network` (the package's resolved
 * `sail.config.json` `"network"` allowlist) and throws for anything outside
 * it, then delegates to `realFetch`. `network` empty/absent means no grant:
 * the caller should simply not endow `fetch` at all in that case (today's
 * default, unchanged) rather than call this with an empty list — an always-
 * throwing endowed `fetch` is observably different from an undefined one.
 */
export function createConfinedFetch(
  network: readonly string[],
  realFetch: typeof fetch = (globalThis as { fetch?: typeof fetch }).fetch as typeof fetch,
): typeof fetch {
  // `async` so a refusal is a REJECTED PROMISE (matching `fetch`'s own
  // contract — callers `await`/`.catch()` it), not a synchronous throw that
  // would escape an un-awaited call site differently from a real fetch failure.
  const confined = (async (input: RequestInfo | URL, init?: RequestInit) => {
    const target = requestTarget(input)
    if (!targetAllowed(target, network)) {
      throw new Error(
        `confined fetch: "${target}" is not on this package's sail.config.json "network" allowlist`,
      )
    }
    return realFetch(input, init)
  }) as typeof fetch
  return confined
}
