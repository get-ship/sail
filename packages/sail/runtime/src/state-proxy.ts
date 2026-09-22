//! The one `MutableState` proxy. Both mount paths build their writable state
//! through `createMountState` — there is no second implementation, and there
//! is no per-mount mode flag.
//!
//! Reads of declared signal keys return the UNWRAPPED value (the ergonomic
//! `state.count++`, `state.label = 'hi'`). Writes route through `signal.set`
//! via the set-trap. The raw signal is reached via `state.$signals.x` for the
//! cases the assignment-style can't express (handing a `Signal<T>` to a
//! callback, calling `.set(updater)` with a function).
//!
//! Non-signal scope members (methods, plain refs, `derived` getters that
//! arrived as callable views) fall through unchanged — `state.togglePanel`
//! is the bound method, `state.foo` is whatever the factory returned.
//!
//! The unwrap is INTENTIONAL sugar for IMPERATIVE contexts (`on:` handlers,
//! `mount`, `unmount`) where reactive tracking doesn't apply — those are
//! one-shot reads, not subscriptions. Reactive contexts (`derived`, `effect`,
//! templates) receive the raw scope, not this proxy, so they always work
//! against callable signals.
//!
//! History: the proxy used to take an `unwrap: boolean` option — component-
//! mount passed `unwrap: false`, leaving `state.x` as the raw signal. That
//! created a type/runtime lie (`MutableState<Scope>` unwraps in TS, but the
//! component runtime didn't), which silently broke casts like
//! `state.canvasRef as HTMLCanvasElement`. The fork retired in favor of one
//! shape; the 58 component sites that called `state.x()` became `state.x`.

import type { Signal } from './signal'
import { isSignal } from './signal'

/** The read-only accessor key exposing the raw signal map: `state.$signals.x`. */
export const SIGNALS_KEY = '$signals'

/**
 * Build the canonical writable-state proxy over `scope`. `signals` is the
 * writable-signal subset of `scope` (drives `$signals`, set-routing, and key
 * enumeration). Reads of signal keys return the unwrapped value; writes
 * route through `signal.set`.
 */
export function createMountState(
  scope: Record<string, unknown>,
  signals: Record<string, Signal<unknown>>,
): Record<string, unknown> {
  return new Proxy(scope, {
    get(t, k: string | symbol) {
      if (k === SIGNALS_KEY) return signals
      // Unwrap DECLARED signal keys (the ergonomic read). Non-signal members
      // fall through — methods, refs that arrived as functions, etc.
      if ((k as string) in signals) return signals[k as string]!()
      return (t as Record<string | symbol, unknown>)[k]
    },
    set(_t, k: string | symbol, value: unknown) {
      if (k === SIGNALS_KEY) {
        console.warn(
          `[sail] ${SIGNALS_KEY} is the read-only signal-map accessor; the assignment was ignored.`,
        )
        return true
      }
      const sig = signals[k as string]
      if (sig) {
        // signal.set treats a function arg as an updater; a `state.foo = () =>
        // {…}` assignment means STORE the function, so wrap it as a constant
        // updater.
        if (typeof value === 'function') sig.set(() => value)
        else sig.set(value)
        return true
      }
      console.warn(
        `[sail] s.${String(k)} = … was ignored — ${String(k)} is not a ` +
          `writable signal. Declare it as a signal in the controller's ` +
          `setup: factory and return it (\`${String(k)}: signal(…)\`).`,
      )
      // Return true so strict-mode module assignments don't throw — the write
      // is dropped, the warning is the signal.
      return true
    },
    has(t, k) {
      return k === SIGNALS_KEY || k in (t as object)
    },
    ownKeys() {
      return [SIGNALS_KEY, ...Object.keys(signals)]
    },
    getOwnPropertyDescriptor(_t, k: string | symbol) {
      if (k === SIGNALS_KEY) {
        return { configurable: true, enumerable: true, writable: false, value: signals }
      }
      const sig = signals[k as string]
      if (!sig) return undefined
      return {
        configurable: true,
        enumerable: true,
        writable: true,
        value: sig(),
      }
    },
  }) as Record<string, unknown>
}

/** The writable-signal subset of a scope object. */
export function pickSignals(scope: Record<string, unknown>): Record<string, Signal<unknown>> {
  const out: Record<string, Signal<unknown>> = {}
  for (const [k, v] of Object.entries(scope)) {
    if (isSignal(v)) out[k] = v
  }
  return out
}

/**
 * Component-mount state. The compiler emits `componentMountState($scope)` in
 * place of a hand-rolled proxy. Behavior matches route-mount — one shape,
 * one builder, no flag.
 */
export function componentMountState(scope: Record<string, unknown>): Record<string, unknown> {
  return createMountState(scope, pickSignals(scope))
}
