// queried — native-backed implementation of the swap boundary.
//
// Contract: `queried()` forwards to `nativeQueried()` (native-cache.ts) with
// the same signature and five-clause contract as the public `query()` door.
// Not exported from index.ts or the package `exports` map — `query()`
// (query.ts) is the only public entry. See query.ts for the five-clause
// CONTRACT. Historical note: a prior path wrapped @tanstack/query-core's
// QueryObserver; the engine layer is native-only now.

import type { Signal } from './signal'
import type { LiveState } from './live-state'
import { nativeQueried, type QueryKey, type NativeCacheOpts } from './native-cache'

export type QueriedOpts = NativeCacheOpts

/**
 * Cache-backed pull seam under `query()`. `key` is the explicit cache
 * identity — no inference from `fn`. `fn` is the fetcher; its return
 * value becomes the `ok` state's `value`.
 *
 * Thin forwarder onto `nativeQueried()` — kept as its own module (not
 * inlined into `query.ts`) so the swap boundary stays a single,
 * clearly-named file per the seam contract's own convention.
 */
export function queried<T>(
  key: QueryKey,
  fn: (signal: AbortSignal) => Promise<T>,
  opts?: QueriedOpts,
): Signal<LiveState<T>> {
  return nativeQueried(key, fn, opts)
}
