// queried — adapter from @tanstack/query-core's QueryObserver onto the
// LiveState three-state union.
//
// `queried(key, fn, opts?)` is the query-core-backed sibling of
// `live()`: same LiveSignal<T> face, but backed by a cache with
// dedup, staleness, and GC instead of a bare re-run-on-dep-change
// effect. `live()` itself is UNCHANGED by this module — a separate
// primitive, so cutting query-core in doesn't touch the SSE/patch
// path `live()` also serves.
//
// INTERNAL/UNDOCUMENTED. Not exported from index.ts or the package's
// public `exports` map. `live()` is the only user-facing async
// primitive today; `queried`'s public naming and explicit-key-vs-
// inferred-key identity story belong to a future data-architecture
// decision, not this module. No app site, tutorial, or snippet may
// import this.
//
// ---------------------------------------------------------------
// CONTRACT — the swap boundary.
//
// `queried` is deliberately the ONLY file in the runtime that knows
// @tanstack/query-core exists. Everything above this file — `live()`
// callers, components, the compiler's emitted code — sees nothing
// but `Signal<LiveState<T>>`. Replacing query-core with a different
// cache engine (or a hand-rolled one) means rewriting this file and
// `query-client.ts` alone; the interface below is what the
// replacement must reproduce, unchanged, for that swap to be free.
//
// 1. SUBSCRIBE. `queried(key, fn, opts)` returns a `Signal<LiveState<T>>`
//    that is live for the lifetime of the calling effect scope: every
//    state transition the engine produces for `key` is pushed onto
//    the signal via `.set()`, synchronously with the engine's own
//    notification (no extra microtask hop introduced by this file).
//    A fresh call with a key already cached by an ancestor observer
//    must synchronously reflect the cached value on first read (see
//    REMOUNT below) — no signal ever starts in `loading` if the
//    engine already holds fresh data for that key.
//
// 2. READ. The signal's current value is always one of exactly three
//    `LiveState<T>` shapes — `{kind:'loading'}`, `{kind:'ok',value}`,
//    `{kind:'error',error}` — never a fourth shape, and never
//    `undefined`. The mapping from the engine's native result shape
//    to these three is this file's job; nothing downstream is
//    allowed to see the engine's richer status vocabulary
//    (`isFetching`, `isStale`, `isPaused`, placeholder data,
//    background-refetch-in-flight, etc.) — LiveState has no slot for
//    it, and a background revalidation of already-`ok` data must NOT
//    flip the signal back to `loading`. Only a genuine status
//    transition (`pending` → `success`/`error`, or a NEW key
//    starting fresh) may move the signal off its current kind.
//
// 3. INVALIDATE. Cache invalidation, GC, and retry policy are the
//    engine's job end-to-end — this file exposes none of it as its
//    own surface (see `QueriedOpts` below: `staleTime`/`gcTime`/
//    `retry` pass straight through, unmodified, uninterpreted). A
//    replacement engine must offer equivalents for these three knobs
//    or the adapter has nothing to forward.
//
// 4. OWNER-TREE HOMING. `queried` never allocates or references a
//    cache directly — it resolves one shared cache handle per call
//    via `useQueryClient()` (see `query-client.ts`), which homes that
//    handle on the CURRENT EFFECT SCOPE'S CHAIN ROOT (walking
//    `.parent` to `parent === null`), never a module singleton. Two
//    components under the same root sharing `key` share one fetch;
//    two roots (two parallel server requests, or two independent
//    mounts) never see each other's cache. A replacement engine's
//    client/cache object must be constructible fresh per root and
//    disposable when the root scope disposes (`query-client.ts`
//    registers `.unmount()` + cache clear as a root cleanup) — no
//    replacement may leak state across roots or across the app's
//    lifetime.
//
// 5. UNSUBSCRIBE. `queried` registers its own teardown via
//    `onCleanup` on the CALLING scope (not the root) — when that
//    scope disposes, the underlying observer unsubscribes and stops
//    receiving notifications. Mirrors `live()`'s disposal contract:
//    no signal update after the owning scope is gone.
// ---------------------------------------------------------------

import type { QueryKey } from '@tanstack/query-core'
import { QueryObserver } from '@tanstack/query-core'
import { signal, type Signal } from './signal'
import { onCleanup } from './effect-scope'
import type { LiveState } from './live'
import { useQueryClient } from './query-client'

/** Options passed through to the underlying `QueryObserver`. Only the
 *  subset the adapter currently forwards is surfaced here; the rest
 *  of query-core's `QueryObserverOptions` is available to widen as
 *  the internal call sites need it — see CONTRACT §3 above. */
export interface QueriedOpts {
  /** Milliseconds data is considered fresh — a remount within this
   *  window paints the cached value with no refetch. Default: 0
   *  (query-core's default — always stale, always refetches). */
  staleTime?: number
  /** Milliseconds unused cache data survives before GC. Default:
   *  query-core's default (5 minutes). */
  gcTime?: number
  /** Retry count/policy on fetch failure. Default: query-core's
   *  default (3 retries, exponential backoff). */
  retry?: boolean | number
}

/**
 * Cache-backed sibling of `live()`. `key` is the explicit cache
 * identity (query-core's `queryKey`) — no inference from `fn`. `fn`
 * is the fetcher; its return value becomes the `ok` state's `value`.
 *
 * Resolves one `QueryClient` per owner-tree chain root (via
 * `useQueryClient` — see `query-client.ts`), so two components under
 * the same root reading the same `key` share one fetch and one cache
 * entry; two roots never see each other's cache.
 *
 * Unsubscribes the underlying `QueryObserver` via `onCleanup` on the
 * calling scope — mirrors `live()`'s disposal contract.
 */
export function queried<T>(
  key: QueryKey,
  fn: () => Promise<T>,
  opts?: QueriedOpts,
): Signal<LiveState<T>> {
  const client = useQueryClient()
  const observer = new QueryObserver<T, Error, T, T>(client, {
    queryKey: key,
    queryFn: fn,
    ...(opts?.staleTime !== undefined && { staleTime: opts.staleTime }),
    ...(opts?.gcTime !== undefined && { gcTime: opts.gcTime }),
    ...(opts?.retry !== undefined && { retry: opts.retry }),
  })

  const toLiveState = (result: ReturnType<typeof observer.getCurrentResult>): LiveState<T> => {
    if (result.status === 'pending') return { kind: 'loading' }
    if (result.status === 'error') return { kind: 'error', error: result.error }
    return { kind: 'ok', value: result.data }
  }

  const sig = signal<LiveState<T>>(toLiveState(observer.getCurrentResult()))

  const unsubscribe = observer.subscribe((result) => {
    sig.set(toLiveState(result))
  })

  onCleanup(unsubscribe)

  return sig
}
