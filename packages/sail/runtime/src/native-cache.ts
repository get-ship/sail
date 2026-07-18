// native-cache — the signals-native cache engine. WIRED IN as of the
// query-slice job: `queried()` (queried.ts) and the public `query()`/
// `mutation()` primitives (query.ts) sit directly on top of this
// file. Originated as a spike answering one question with numbers —
// see `helm/data/missions/m-deck-standalone/scratch/signals-cache-spike-report.md`
// — then ruled the engine in `data-arch-seed.md` ("CACHE = BUILD").
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
// Implements the FIVE-CLAUSE CONTRACT — see `query.ts`'s module
// header for the canonical, numbered text (moved there now that
// `query()` is the public door; this file is implementation detail
// underneath it). One-line-per-clause recap: (1) SUBSCRIBE — the
// returned Signal IS the entry's signal, synchronous `.set()`; (2)
// READ — exactly `LiveState<T>`, never flips ok→loading on a
// background revalidate; (3) INVALIDATE — staleTime/gcTime/retry are
// the only knobs, `invalidate(prefix)` (method below) is the shared
// mechanism, every fetch attempt owns a real `AbortController`; (4)
// OWNER-TREE HOMING — one `NativeCache` per chain root via
// `useNativeCache()`, never a module singleton; (5) UNSUBSCRIBE —
// `onCleanup` on the CALLING scope.

import { signal, type Signal } from './signal'
import { onCleanup, currentScope, createScope, type EffectScope } from './effect-scope'
import { isLiveSource, type LiveState, type LiveSignal, type LiveSource } from './live-state'

/** A cache identity: client-root + static member-path segments +
 *  (optionally) a tail of call-arg values. Compiler-derived, never
 *  handwritten by an author — see query.ts's module header. Owned
 *  here rather than imported from `@tanstack/query-core` so the real
 *  engine has zero dependency on that package; only the retired
 *  differential-test twin (`queried-querycore.ts`) still imports
 *  query-core's own `QueryKey`, which is structurally the same
 *  shape (`readonly unknown[]`). */
export type QueryKey = readonly unknown[]

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
  /** The fetcher last passed to `fetch()`. Retained so `invalidate()`
   *  can trigger a background refetch of an actively-observed entry
   *  without the caller re-supplying `fn` (see INVALIDATE contract). */
  fetchFn: ((signal: AbortSignal) => Promise<unknown>) | null
  /** The controller for whichever attempt (initial or retry) is
   *  currently in flight; `null` when nothing is fetching. Aborted on
   *  dispose, GC eviction, and invalidate-while-fetching — real
   *  network work stops, it does not just get its result discarded. */
  abortController: AbortController | null
  /** Set the first time a push source attaches; lets a second
   *  `nativePush` call on the same key reuse the one subscription
   *  instead of opening a duplicate. */
  pushUnsubscribe: (() => void) | null
  /** Set once this entry is torn down (root `dispose()` or GC
   *  eviction). A fetch or push callback closed over this entry
   *  checks this BEFORE `fetchToken` — neither `dispose()` nor GC
   *  eviction bumps `fetchToken`, so without this flag a promise
   *  already in flight (or a push source that keeps delivering after
   *  its own unsubscribe) writes into a torn-down entry's signal. */
  disposed: boolean
}

const toError = (err: unknown): Error => (err instanceof Error ? err : new Error(String(err)))

function hashKey(key: QueryKey): string {
  return JSON.stringify(key)
}

function retryDelay(attempt: number): number {
  return Math.min(1000 * 2 ** (attempt - 1), 30_000)
}

/** Whether `key` starts with every segment of `prefix`, value-compared
 *  (not reference-compared, since args are often fresh objects/arrays
 *  per call). `invalidate(crm)` → `['crm']`, `invalidate(crm.deals)` →
 *  `['crm','deals']`, `invalidate(crm.deals.list)` →
 *  `['crm','deals','list']` — no args segment, so it matches every
 *  cached call to that method regardless of arguments. */
function isKeyPrefixedBy(key: QueryKey, prefix: QueryKey): boolean {
  if (prefix.length > key.length) return false
  for (let i = 0; i < prefix.length; i++) {
    if (JSON.stringify(key[i]) !== JSON.stringify(prefix[i])) return false
  }
  return true
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
        fetchFn: null,
        abortController: null,
        pushUnsubscribe: null,
        disposed: false,
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
      entry.disposed = true
      entry.pushUnsubscribe?.()
      entry.abortController?.abort()
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
   *  a genuinely empty entry starts in `loading`. `fn` receives the
   *  in-flight attempt's `AbortSignal`; dispose, GC eviction, and a
   *  forced invalidate-refetch all abort it for real, not just discard
   *  its eventual result. */
  fetch<T>(
    entry: CacheEntry<T>,
    fn: (signal: AbortSignal) => Promise<T> | LiveSource<T>,
  ): void {
    if (entry.status === 'fetching') return
    entry.status = 'fetching'
    const myToken = ++entry.fetchToken
    entry.fetchFn = fn as (signal: AbortSignal) => Promise<unknown>
    // Only a genuinely empty/errored entry flashes to loading. An
    // invalidate-triggered background revalidate of `ok` data zeroes
    // `dataUpdatedAt` (staleness bookkeeping) but must NOT flip the
    // signal — checking the signal's own kind, not `dataUpdatedAt`,
    // keeps those two concerns from colliding.
    if (entry.sig.__peek().kind !== 'ok') entry.sig.set({ kind: 'loading' })

    const onError = (err: unknown): void => {
      if (entry.disposed || myToken !== entry.fetchToken) return
      entry.abortController = null
      const maxRetries = entry.retry === false ? 0 : entry.retry === true ? Infinity : entry.retry
      if (entry.retryCount < maxRetries) {
        entry.retryCount++
        entry.retryTimer = setTimeout(() => {
          if (entry.disposed || myToken !== entry.fetchToken) return
          run()
        }, retryDelay(entry.retryCount))
        return
      }
      entry.status = 'idle'
      entry.sig.set({ kind: 'error', error: toError(err) })
    }

    const run = (): void => {
      const controller = new AbortController()
      entry.abortController = controller

      // TYPE-DIRECTED ROUTING at the runtime type-resolved seam: the
      // fetcher's return value shape reflects the client method's
      // compile-time return type. A `LiveSource<T>` (a `subscribe`
      // method) is a PUSH source — hand the entry over to it; a
      // `Promise<T>` is a PULL — await it with the retry/token machinery
      // below. Same `CacheEntry`, same `LiveState` face out.
      let result: Promise<T> | LiveSource<T>
      try {
        result = fn(controller.signal)
      } catch (err) {
        onError(err)
        return
      }

      if (isLiveSource<T>(result)) {
        // The sync `subscribe` is the whole "fetch" — no promise to
        // await, no retry, nothing to abort via the controller (the
        // source owns its own teardown through the returned disposer).
        entry.status = 'idle'
        entry.abortController = null
        entry.retryCount = 0
        entry.pushUnsubscribe = result.subscribe(this.pushProxy(entry))
        return
      }

      result.then((value) => {
        if (entry.disposed || myToken !== entry.fetchToken) return
        entry.status = 'idle'
        entry.abortController = null
        entry.dataUpdatedAt = Date.now()
        entry.retryCount = 0
        entry.sig.set({ kind: 'ok', value })
      }, onError)
    }
    run()
  }

  /** Adapt the cache entry's signal into the guarded `LiveSignal` a
   *  `LiveSource.subscribe` writes into: every push checks the entry
   *  isn't torn down / superseded, stamps `dataUpdatedAt` on `ok`, then
   *  forwards to the real entry signal. Reads pass straight through, so a
   *  source that accumulates (reads `sig()` before its next `set`) sees
   *  the live value. Mirrors `nativePush`'s inline guards. */
  private pushProxy<T>(entry: CacheEntry<T>): LiveSignal<T> {
    const proxy = (() => entry.sig()) as LiveSignal<T>
    // Guard on `disposed` ONLY (mirrors `nativePush`): a push subscription
    // is long-lived and independent of fetch tokens — a concurrent pull
    // refetch bumping `fetchToken` must NOT silence the still-live stream
    // (the PUSH+PULL interleave contract).
    proxy.set = (next: LiveState<T> | ((prev: LiveState<T>) => LiveState<T>)): void => {
      if (entry.disposed) return
      const s = typeof next === 'function' ? next(entry.sig.__peek()) : next
      if (s.kind === 'ok') entry.dataUpdatedAt = Date.now()
      entry.sig.set(s)
    }
    proxy.__peek = () => entry.sig.__peek()
    proxy.__subs = entry.sig.__subs
    return proxy
  }

  /** Invalidate every entry whose key is prefixed by `prefix` (the
   *  compiler-resolved surface: `invalidate(crm.deals.list)`,
   *  `invalidate(crm.deals)`, `invalidate(crm)` all compile down to a
   *  prefix here). Same mechanism serves mutation auto-invalidation
   *  and the public `invalidate()` grammar — one implementation, two
   *  callers. See invalidate-design-proposal.md for the full contract;
   *  summary of the five rules applied per matched entry:
   *   1. mark stale (`dataUpdatedAt = 0`),
   *   2. observed entries refetch in the background — `ok` never
   *      flips to `loading` mid-revalidate,
   *   3. unobserved-but-resident entries are just marked stale — no
   *      wasted fetch; the next `attach()` catches it via `isFresh()`,
   *   4. push-fed entries are exempt, skipped silently,
   *   5. an in-flight fetch is aborted and its token bumped so the
   *      stale resolution can never land, then a fresh fetch starts. */
  invalidate(prefix: QueryKey): void {
    for (const entry of this.entries.values()) {
      if (entry.disposed) continue
      if (!isKeyPrefixedBy(entry.key, prefix)) continue

      entry.dataUpdatedAt = 0
      if (entry.pushUnsubscribe !== null) continue // rule 4: push-fed, exempt
      if (entry.observers === 0) continue // rule 3: unobserved, just marked stale above

      // rule 5: discard whatever the in-flight attempt was about to write
      if (entry.status === 'fetching') {
        entry.abortController?.abort()
        entry.abortController = null
        entry.fetchToken++
      }
      if (entry.retryTimer !== null) {
        clearTimeout(entry.retryTimer)
        entry.retryTimer = null
      }
      entry.retryCount = 0
      entry.status = 'idle'
      if (entry.fetchFn) this.fetch(entry, entry.fetchFn)
    }
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
      entry.disposed = true
      this.cancelGc(entry)
      if (entry.retryTimer !== null) clearTimeout(entry.retryTimer)
      entry.pushUnsubscribe?.()
      entry.abortController?.abort()
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

const idKey = Symbol.for('sail.nativeCache')

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
 * Cache-backed pull seam under `query()` — the native-core counterpart to
 * `queried()`. `key` is the explicit cache identity; `fn` is the
 * fetcher. Resolves one `NativeCache` per owner-tree chain root, so
 * two components under the same root sharing `key` share one fetch
 * and one entry; two roots never see each other's cache.
 */
export function nativeQueried<T>(
  key: QueryKey,
  fn: (signal: AbortSignal) => Promise<T>,
  opts?: NativeCacheOpts,
): Signal<LiveState<T>> {
  const cache = useNativeCache()
  const { entry, release } = cache.attach<T>(key, opts)

  if (!cache.isFresh(entry as CacheEntry<unknown>)) cache.fetch(entry, fn)

  onCleanup(release)
  return entry.sig
}

/**
 * The unified read seam `query()` sits on: one keyed entry fed by a
 * fetcher that returns EITHER a `Promise<T>` (PULL — one-shot fill,
 * staleness, retry) OR a `LiveSource<T>` (PUSH — a `subscribe` that keeps
 * writing the entry). The pull-vs-push decision is `fetch`'s
 * `isLiveSource` branch — the return value's shape IS the routing signal,
 * so there is no second primitive and no author marker. Shares
 * `CacheEntry<T>` with `nativeQueried`/`nativePush`: a pull and a push on
 * the same key write the same signal.
 */
export function nativeQuery<T>(
  key: QueryKey,
  fn: (signal: AbortSignal) => Promise<T> | LiveSource<T>,
  opts?: NativeCacheOpts,
): Signal<LiveState<T>> {
  const cache = useNativeCache()
  const { entry, release } = cache.attach<T>(key, opts)

  // A push-fed entry is owned by its subscription (never stale, never
  // refetched) — a later observer just shares it. A pull entry refetches
  // only when not fresh. `fetch()` itself no-ops on an already-push-fed
  // or in-flight entry, so this is just the pull-side freshness gate.
  if (entry.pushUnsubscribe === null && !cache.isFresh(entry as CacheEntry<unknown>)) {
    cache.fetch(entry, fn)
  }

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
        if (entry.disposed) return
        entry.dataUpdatedAt = Date.now()
        entry.sig.set({ kind: 'ok', value })
      },
      (err) => {
        if (entry.disposed) return
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
