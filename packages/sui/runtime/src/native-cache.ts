// native-cache — SPIKE. A signals-native replacement for the
// query-core-backed `queried()` (see `queried.ts`'s CONTRACT header,
// which this file reproduces). NOT wired in: no app site, no
// index.ts export. Exists to answer one question with numbers — see
// `helm/data/missions/m-deck-standalone/scratch/signals-cache-spike-report.md`.
//
// The bet: a cache entry IS a signal homed on the owner tree. A pull
// (`nativeQueried`) fills the entry once and lets it go stale; a push
// source (`nativePush`) keeps writing into the same keyed entry. Both
// share one `CacheEntry<T>` — there is no adapter layer translating
// between a "query" vocabulary and a "subscription" vocabulary, so
// interleaving pull and push on one key falls out for free (see the
// `interleaved pull+push` test) instead of needing a second primitive
// query-core has no room for.
//
// Reproduces the FIVE-CLAUSE CONTRACT from queried.ts:
//   1. SUBSCRIBE — the returned Signal IS the entry's signal; every
//      transition lands via `.set()` synchronously, no microtask hop
//      added by this file. A remount that finds a live entry reflects
//      its current value synchronously (nothing to do — same signal).
//   2. READ — exactly the frozen `LiveState<T>` union; a background
//      revalidate (§ stale-while-revalidate) never flips ok→loading.
//   3. INVALIDATE — staleTime/gcTime/retry are the only knobs; forwarded
//      unmodified from `NativeCacheOpts`, interpreted only by this file.
//   4. OWNER-TREE HOMING — one `NativeCache` per chain root, resolved by
//      walking `.parent` to `parent === null` (mirrors `query-client.ts`
//      exactly), never a module singleton. Disposed with the root.
//   5. UNSUBSCRIBE — registered via `onCleanup` on the CALLING scope.

import type { QueryKey } from '@tanstack/query-core'
import { signal, type Signal } from './signal'
import { onCleanup, currentScope, createScope, type EffectScope } from './effect-scope'
import type { LiveState } from './live'

// ---------------------------------------------------------------------------
// Cache entry
// ---------------------------------------------------------------------------

interface CacheEntry<T> {
  key: QueryKey
  sig: Signal<LiveState<T>>
  status: 'idle' | 'fetching'
  /** 0 until the first successful fill (pull or push). */
  dataUpdatedAt: number
  observers: number
  gcTimer: ReturnType<typeof setTimeout> | null
  staleTime: number
  gcTime: number
  retry: number | boolean
  retryCount: number
  retryTimer: ReturnType<typeof setTimeout> | null
  /** Bumped on every fetch start; a resolution whose token has gone
   *  stale (dispose during flight, or a newer fetch started) is
   *  discarded. Mirrors `live.ts`'s token guard. */
  fetchToken: number
  /** Set the first time a push source attaches; lets a second
   *  `nativePush` call on the same key reuse the one subscription
   *  instead of opening a duplicate. */
  pushUnsubscribe: (() => void) | null
}

const toError = (err: unknown): Error => (err instanceof Error ? err : new Error(String(err)))

function hashKey(key: QueryKey): string {
  return JSON.stringify(key)
}

function retryDelay(attempt: number): number {
  return Math.min(1000 * 2 ** (attempt - 1), 30_000)
}

// ---------------------------------------------------------------------------
// NativeCache — one per owner-tree chain root
// ---------------------------------------------------------------------------

export class NativeCache {
  private entries = new Map<string, CacheEntry<unknown>>()

  private getOrCreate<T>(key: QueryKey, opts?: NativeCacheOpts): CacheEntry<T> {
    const hashed = hashKey(key)
    let entry = this.entries.get(hashed) as CacheEntry<T> | undefined
    if (!entry) {
      entry = {
        key,
        sig: signal<LiveState<T>>({ kind: 'loading' }),
        status: 'idle',
        dataUpdatedAt: 0,
        observers: 0,
        gcTimer: null,
        staleTime: opts?.staleTime ?? 0,
        gcTime: opts?.gcTime ?? 5 * 60_000,
        retry: opts?.retry ?? 3,
        retryCount: 0,
        retryTimer: null,
        fetchToken: 0,
        pushUnsubscribe: null,
      }
      this.entries.set(hashed, entry as CacheEntry<unknown>)
    }
    return entry
  }

  private cancelGc(entry: CacheEntry<unknown>): void {
    if (entry.gcTimer !== null) {
      clearTimeout(entry.gcTimer)
      entry.gcTimer = null
    }
  }

  private scheduleGc(hashed: string, entry: CacheEntry<unknown>): void {
    entry.gcTimer = setTimeout(() => {
      entry.pushUnsubscribe?.()
      if (entry.retryTimer !== null) clearTimeout(entry.retryTimer)
      this.entries.delete(hashed)
    }, entry.gcTime)
  }

  /** Attach an observer to `key`'s entry (creating it if needed),
   *  cancelling any pending GC and bumping the refcount. Returns the
   *  entry plus a release callback the caller registers on cleanup. */
  attach<T>(key: QueryKey, opts?: NativeCacheOpts): { entry: CacheEntry<T>; release: () => void } {
    const entry = this.getOrCreate<T>(key, opts)
    // Unlike creation-only defaults, staleTime/gcTime/retry are applied on
    // EVERY attach — an entry created by `hydrate` (no opts) must still pick
    // up the staleTime a subsequent `nativeQueried(..., opts)` call passes,
    // or a hydrated entry can never be judged fresh.
    if (opts?.staleTime !== undefined) entry.staleTime = opts.staleTime
    if (opts?.gcTime !== undefined) entry.gcTime = opts.gcTime
    if (opts?.retry !== undefined) entry.retry = opts.retry
    this.cancelGc(entry as CacheEntry<unknown>)
    entry.observers++
    const hashed = hashKey(key)
    return {
      entry,
      release: () => {
        entry.observers--
        if (entry.observers === 0) this.scheduleGc(hashed, entry as CacheEntry<unknown>)
      },
    }
  }

  /** Start (or join) a fetch for `entry`. A background revalidate of
   *  already-`ok` data never flips the signal back to `loading` — only
   *  a genuinely empty entry starts in `loading`. */
  fetch<T>(entry: CacheEntry<T>, fn: () => Promise<T>): void {
    if (entry.status === 'fetching') return
    entry.status = 'fetching'
    const myToken = ++entry.fetchToken
    if (entry.dataUpdatedAt === 0) entry.sig.set({ kind: 'loading' })

    const run = (): void => {
      fn().then(
        (value) => {
          if (myToken !== entry.fetchToken) return
          entry.status = 'idle'
          entry.dataUpdatedAt = Date.now()
          entry.retryCount = 0
          entry.sig.set({ kind: 'ok', value })
        },
        (err) => {
          if (myToken !== entry.fetchToken) return
          const maxRetries = entry.retry === false ? 0 : entry.retry === true ? Infinity : entry.retry
          if (entry.retryCount < maxRetries) {
            entry.retryCount++
            entry.retryTimer = setTimeout(() => {
              if (myToken !== entry.fetchToken) return
              run()
            }, retryDelay(entry.retryCount))
            return
          }
          entry.status = 'idle'
          entry.sig.set({ kind: 'error', error: toError(err) })
        },
      )
    }
    run()
  }

  /** Whether `key` is fresh enough that a new observer should skip a
   *  refetch (mirrors query-core's `staleTime` semantics). */
  isFresh(entry: CacheEntry<unknown>): boolean {
    return entry.dataUpdatedAt !== 0 && Date.now() - entry.dataUpdatedAt < entry.staleTime
  }

  /** Serialize every entry currently holding `ok` data. Loading/error
   *  entries are not persisted — same as query-core's default
   *  `shouldDehydrateQuery`. */
  dehydrate(): DehydratedEntry[] {
    const out: DehydratedEntry[] = []
    for (const entry of this.entries.values()) {
      const state = entry.sig.__peek()
      if (state.kind === 'ok') {
        out.push({ key: entry.key, value: state.value, dataUpdatedAt: entry.dataUpdatedAt })
      }
    }
    return out
  }

  /** Seed entries from a prior `dehydrate()` call — typically on a
   *  fresh client-side root after SSR. */
  hydrate(dehydrated: DehydratedEntry[]): void {
    for (const d of dehydrated) {
      const entry = this.getOrCreate(d.key)
      entry.sig.set({ kind: 'ok', value: d.value })
      entry.dataUpdatedAt = d.dataUpdatedAt
    }
  }

  /** Torn down with the owning root: stop every timer and push
   *  subscription so nothing outlives the scope that created it. */
  dispose(): void {
    for (const entry of this.entries.values()) {
      this.cancelGc(entry)
      if (entry.retryTimer !== null) clearTimeout(entry.retryTimer)
      entry.pushUnsubscribe?.()
    }
    this.entries.clear()
  }

  /** Test/debug only — the live observer count for `key`'s entry, or
   *  `undefined` if nothing has ever touched that key. */
  debugObserverCount(key: QueryKey): number | undefined {
    return this.entries.get(hashKey(key))?.observers
  }

  /** Test/debug only — whether `key`'s entry is still resident. */
  debugHasEntry(key: QueryKey): boolean {
    return this.entries.has(hashKey(key))
  }

  /** Test/debug only — a live reference to `key`'s entry, so a caller
   *  can read `.observers` AFTER the entry has been evicted from the
   *  map (e.g. by the owning root disposing) — the object itself is
   *  mutated in place by `release()` before eviction ever runs. */
  debugEntryRef(key: QueryKey): { readonly observers: number } | undefined {
    return this.entries.get(hashKey(key))
  }
}

export interface DehydratedEntry {
  key: QueryKey
  value: unknown
  dataUpdatedAt: number
}

// ---------------------------------------------------------------------------
// Owner-tree homing — mirrors query-client.ts exactly
// ---------------------------------------------------------------------------

const idKey = Symbol.for('sui.nativeCache')

function rootOf(scope: EffectScope): EffectScope {
  let cur: EffectScope = scope
  while (cur.parent) cur = cur.parent
  return cur
}

let ANON_ROOT: EffectScope | null = null
function anonRoot(): EffectScope {
  if (!ANON_ROOT || ANON_ROOT.disposed) ANON_ROOT = createScope()
  return ANON_ROOT
}

/**
 * Resolve the `NativeCache` for the current owner-tree chain root. The
 * first call for a given root allocates a fresh cache and registers
 * `.dispose()` as a cleanup on that root; descendant scopes share it.
 * NEVER a module singleton — mirrors `useQueryClient`.
 */
export function useNativeCache(): NativeCache {
  const current = currentScope()
  const root = rootOf(current ?? anonRoot())
  const existing = root.contexts.get(idKey) as NativeCache | undefined
  if (existing) return existing

  const cache = new NativeCache()
  root.contexts.set(idKey, cache)
  root.cleanup(() => cache.dispose())
  return cache
}

// ---------------------------------------------------------------------------
// Public primitives
// ---------------------------------------------------------------------------

/** Options honored by both `nativeQueried` and `nativePush`. Only
 *  `staleTime`/`gcTime`/`retry` — see CONTRACT §3 above. `retry` is
 *  meaningless for a push source (nothing to retry) and ignored there. */
export interface NativeCacheOpts {
  /** Milliseconds data is considered fresh — a remount within this
   *  window paints the cached value with no refetch. Default: 0. */
  staleTime?: number
  /** Milliseconds an unobserved entry survives before GC. Default:
   *  5 minutes. */
  gcTime?: number
  /** Retry count on fetch failure, or `false`/`true` for none/infinite.
   *  Default: 3, exponential backoff capped at 30s. */
  retry?: boolean | number
}

/**
 * Cache-backed sibling of `live()` — the native-core counterpart to
 * `queried()`. `key` is the explicit cache identity; `fn` is the
 * fetcher. Resolves one `NativeCache` per owner-tree chain root, so
 * two components under the same root sharing `key` share one fetch
 * and one entry; two roots never see each other's cache.
 */
export function nativeQueried<T>(
  key: QueryKey,
  fn: () => Promise<T>,
  opts?: NativeCacheOpts,
): Signal<LiveState<T>> {
  const cache = useNativeCache()
  const { entry, release } = cache.attach<T>(key, opts)

  if (!cache.isFresh(entry as CacheEntry<unknown>)) cache.fetch(entry, fn)

  onCleanup(release)
  return entry.sig
}

/** A push source: takes ownership of pushing values/errors into the
 *  cache entry until the returned disposer is called. */
export interface PushSource<T> {
  (push: (value: T) => void, fail: (err: unknown) => void): () => void
}

/**
 * The one thing `queried()` cannot express: a keyed entry fed by a
 * PUSH source (wss/sse/SDK store) instead of a one-shot fetch — loading
 * until the first push, `ok(latest)` per push thereafter, `error` if
 * the source reports failure. Shares `CacheEntry<T>` with
 * `nativeQueried`: a pull and a push on the SAME key write the same
 * signal, so interleaving them is not a special case (see
 * `interleaved pull+push` in the test file) — it is just two callers
 * of `.attach()`.
 */
export function nativePush<T>(
  key: QueryKey,
  subscribe: PushSource<T>,
  opts?: Omit<NativeCacheOpts, 'retry'>,
): Signal<LiveState<T>> {
  const cache = useNativeCache()
  const { entry, release } = cache.attach<T>(key, opts)

  if (entry.pushUnsubscribe === null) {
    if (entry.dataUpdatedAt === 0) entry.sig.set({ kind: 'loading' })
    const unsub = subscribe(
      (value) => {
        entry.dataUpdatedAt = Date.now()
        entry.sig.set({ kind: 'ok', value })
      },
      (err) => {
        entry.sig.set({ kind: 'error', error: toError(err) })
      },
    )
    entry.pushUnsubscribe = unsub
  }

  onCleanup(release)
  return entry.sig
}

/** SSR: serialize every `ok` entry on `cache`'s root. Pair with
 *  `hydrateNative` on a fresh root to seed it without a client fetch. */
export function dehydrateNative(cache: NativeCache): DehydratedEntry[] {
  return cache.dehydrate()
}

/** SSR: seed `cache` from a prior `dehydrateNative()` payload. */
export function hydrateNative(cache: NativeCache, dehydrated: DehydratedEntry[]): void {
  cache.hydrate(dehydrated)
}
