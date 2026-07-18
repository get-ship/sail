// queried — the swap boundary's CURRENT implementation, native-backed.
//
// ENGINE RULED (owner, 2026-07-17): NATIVE — see
// helm/data/missions/m-deck-standalone/scratch/data-arch-seed.md
// ("CACHE = BUILD... supersedes 'QUERY = BUY' at the ENGINE layer
// only") and native-cache-torture-report.md (22/22 green, 6/6
// differential scenarios matched byte-for-byte before this swap
// landed). `queried()` used to wrap @tanstack/query-core's
// QueryObserver; it now forwards straight to `nativeQueried()`
// (native-cache.ts) — same signature, same five-clause contract,
// zero query-core dependency in the real path. The retired
// query-core-backed implementation survives ONLY as
// `queried-querycore.ts`'s differential-test twin (see that file's
// header) until the `@tanstack/query-core` dependency itself is
// dropped from package.json.
//
// INTERNAL/UNDOCUMENTED. Not exported from index.ts or the package's
// public `exports` map — `query()` (query.ts) is the ONLY public
// door to this engine (see query.ts's module header for the
// five-clause CONTRACT, moved there from this file now that `query()`
// is the public primitive `live()` used to be).

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
