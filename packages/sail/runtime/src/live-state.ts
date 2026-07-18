// live-state — the loading/ok/error lifecycle union and the PUSH source
// shape, plus the two tiny runtime helpers that operate on them.
//
// OWNERSHIP: these shapes are `query()`'s public face — `query.ts`
// re-exports every one of them, and `query()` is where the author-facing
// contract is documented. They live in their OWN module only to break a
// would-be `query.ts` ↔ `native-cache.ts` import cycle: `native-cache.ts`
// needs `LiveState` (the entry signal's value) and `isLiveSource` (the
// pull-vs-push routing seam), while `query.ts` needs `nativeQuery` from
// `native-cache.ts`. A leaf module both can import keeps that acyclic.
//
// This file used to be `live.ts` (which also defined the now-deleted
// `live()` primitive). `live()` was two capabilities bolted together —
// a PULL (`Promise`) and a PUSH (`LiveSource` subscribe). PULL is
// `query()`; PUSH folded into `query()` by type-directed routing at the
// runtime seam (`isLiveSource` below). See
// `helm/data/missions/m-deck-standalone/scratch/data-arch-seed.md`,
// "Owner RULINGS 2026-07-18".

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
 * Source shape for a PUSH data source (SSE / wss / an SDK live store).
 * `subscribe` takes ownership of the live signal: it transitions to `ok`
 * on the first push, applies subsequent pushes, and may transition to
 * `error` on transport failure. The returned disposer closes the
 * connection.
 *
 * This is the shape a client METHOD returns when it streams
 * (`ship.agents.detail(): LiveSource<Agent>`). `query()` detects it at
 * the runtime type-resolved seam (`isLiveSource`) and wires it to the
 * cache's push path instead of the one-shot pull path — the return type
 * is the routing signal, no author ceremony.
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

import type { Signal } from './signal'

export const toError = (err: unknown): Error =>
  err instanceof Error ? err : new Error(String(err))

/** The pull-vs-push routing seam: a value is a PUSH source iff it carries
 *  a `subscribe` method. A `Promise` (or any plain value) is a PULL. This
 *  is the runtime realization of the ruling's "type-directed routing" —
 *  the returned value's shape faithfully reflects the client method's
 *  compile-time return type (`Promise<T>` vs `LiveSource<T>`). */
export const isLiveSource = <T>(v: unknown): v is LiveSource<T> =>
  typeof v === 'object' &&
  v !== null &&
  typeof (v as { subscribe?: unknown }).subscribe === 'function'
