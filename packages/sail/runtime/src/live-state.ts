// LiveState / LiveSource types + helpers for query() (breaks query ↔ native-cache cycle).

/** The lifecycle of a live/queried value — the frozen three-state union
 *  `query()` reads out, whether the entry is fed by a one-shot pull or a
 *  streaming push. Never a fourth shape. */
export type LiveState<T> =
  | { kind: 'loading' }
  | { kind: 'error'; error: Error }
  | { kind: 'ok'; value: T }

/** A signal that carries the `LiveState<T>` lifecycle. */
export type LiveSignal<T> = Signal<LiveState<T>>

/**
 * The freeze-locked optimistic write surface (SPEC-HARDENING-1 WI3 / S8).
 *
 * A mutation may expose this handle so authors can optimistically patch a
 * related `LiveSignal` (or a local list) before the network settles.
 * Implementation may land any time; the TYPE is the freeze artifact:
 *
 * - `set` — same shape as `Signal.set`: next value, or an updater of the
 *   previous **payload** (`T | undefined` when the live value is not yet
 *   `ok`). Callers never invent a fourth `LiveState` kind.
 * - `rollback` — restore the `LiveState` captured when the optimistic write
 *   was applied (or a no-op if nothing was applied). Called on mutation
 *   failure or explicit cancel.
 *
 * No second optimistic API is introduced later without a freeze revision.
 */
export interface OptimisticHandle<T> {
  set(value: T | ((prev: T | undefined) => T)): void
  rollback(): void
}

/**
 * PUSH data source (SSE / wss / live store).
 * `subscribe` owns the signal: first push → `ok`, may later → `error`;
 * returned disposer closes the connection. `query()` routes via `isLiveSource`.
 */
export interface LiveSource<T> {
  subscribe: (sig: LiveSignal<T>) => () => void
}

/**
 * The value a client method call yields, unwrapping the push/pull union:
 * a `LiveSource<U>` yields `U`, a `Promise<U>` yields `U`, else the value
 * itself. The compiler-generated `Scope` uses this to type a `query()`
 * signal's payload from its client method's declared return — e.g.
 * `feed.clock(): LiveSource<Tick>` ⇒ `LiveValueOf<ReturnType<…>> = Tick`.
 */
export type LiveValueOf<R> = R extends LiveSource<infer U> ? U : Awaited<R>

import type { Signal } from '@ship/sail-runtime'

export const toError = (err: unknown): Error =>
  err instanceof Error ? err : new Error(String(err))

/** SSR/older isolates (ship-js's server environment) may not define
 *  `AbortController`. A pull fetch must still RUN there — cancellation is a
 *  browser concern — so fall back to a no-op controller whose `signal`
 *  satisfies the `AbortSignal`-typed fetcher arg. `abort()` flips
 *  `aborted` but wires no events (nothing to cancel server-side). */
export function makeAbortController(): AbortController {
  if (typeof AbortController !== 'undefined') return new AbortController()
  const signal = {
    aborted: false,
    reason: undefined,
    onabort: null,
    throwIfAborted() {},
    addEventListener() {},
    removeEventListener() {},
    dispatchEvent() {
      return false
    },
  } as unknown as AbortSignal
  return {
    signal,
    abort() {
      ;(signal as { aborted: boolean }).aborted = true
    },
  } as unknown as AbortController
}

/** The pull-vs-push routing seam: a value is a PUSH source iff it carries
 *  a `subscribe` method. A `Promise` (or any plain value) is a PULL. This
 *  is the runtime realization of the ruling's "type-directed routing" —
 *  the returned value's shape faithfully reflects the client method's
 *  compile-time return type (`Promise<T>` vs `LiveSource<T>`). */
export const isLiveSource = <T>(v: unknown): v is LiveSource<T> =>
  typeof v === 'object' &&
  v !== null &&
  typeof (v as { subscribe?: unknown }).subscribe === 'function'
