// native-cache-torture — hammer the native core on exactly the paths
// where young caches die: races, invalidate-during-flight, unmount
// mid-everything, retry backoff, push+pull interleave, dehydrate
// under fire, a differential against query-core, and staleTime clock-
// skew exactness. See
// helm/data/missions/m-deck-standalone/scratch/native-cache-torture-omp.md
// for the brief and
// helm/data/missions/m-deck-standalone/scratch/native-cache-torture-report.md
// for the kill list.
//
// One real bug was found and fixed here (see the UNMOUNT MID-EVERYTHING
// section): neither `NativeCache.dispose()` nor GC eviction bumped
// `fetchToken` or otherwise marked the entry dead, so a fetch (or a
// push source that ignores its own unsubscribe) already in flight at
// teardown time would still write into the torn-down entry's signal
// after the fact. Fixed with an `entry.disposed` flag checked in the
// fetch resolve/reject handlers and the push callbacks, set by both
// `dispose()` and GC eviction.

import { describe, expect, it, vi } from 'vitest'
import { createScope, withScope, type EffectScope } from '../src/effect-scope'
import type { LiveState } from '../src/live'
import {
  nativeQueried,
  nativePush,
  useNativeCache,
  dehydrateNative,
  hydrateNative,
  type PushSource,
} from '../src/native-cache'
import { queried } from '../src/queried'

const tick = () => new Promise((r) => setTimeout(r, 0))

function childOf(root: EffectScope): EffectScope {
  const child = createScope()
  Object.defineProperty(child, 'parent', { value: root, writable: false })
  return child
}

function deferred<T>(): { promise: Promise<T>; resolve: (v: T) => void; reject: (e: unknown) => void } {
  let resolve!: (v: T) => void
  let reject!: (e: unknown) => void
  const promise = new Promise<T>((res, rej) => {
    resolve = res
    reject = rej
  })
  return { promise, resolve, reject }
}

// ---------------------------------------------------------------------------
// 1. RACES
// ---------------------------------------------------------------------------

describe('1. RACES', () => {
  it('two subscribers, same key, second arrives mid-flight — exactly one fetch, both settle identically', async () => {
    const scope = createScope()
    const key = ['race-mid-flight']
    const d = deferred<string>()
    const fetcher1 = vi.fn(() => d.promise)
    const fetcher2 = vi.fn(() => Promise.resolve('SHOULD NEVER RUN'))

    const s1 = withScope(scope, () => nativeQueried(key, fetcher1))
    expect(s1()).toEqual({ kind: 'loading' })

    // second reader arrives WHILE the first fetch is in flight
    const s2 = withScope(scope, () => nativeQueried(key, fetcher2))
    expect(s2()).toEqual({ kind: 'loading' })
    expect(fetcher2).not.toHaveBeenCalled()

    d.resolve('settled')
    await tick()
    await tick()

    expect(fetcher1).toHaveBeenCalledOnce()
    expect(fetcher2).not.toHaveBeenCalled()
    expect(s1).toBe(s2)
    expect(s1()).toEqual({ kind: 'ok', value: 'settled' })
    expect(s2()).toEqual({ kind: 'ok', value: 'settled' })
  })

  it('rapid re-key: a stale key resolving AFTER a newer key can never leak into the newer key\'s entry (separate CacheEntry per key — structurally impossible, unlike a naive shared-signal port)', async () => {
    const scope = createScope()
    const dOld = deferred<string>()
    const dNew = deferred<string>()

    const sOld = withScope(scope, () => nativeQueried(['item', 1], () => dOld.promise))
    const sNew = withScope(scope, () => nativeQueried(['item', 2], () => dNew.promise))

    // resolve the NEW key first...
    dNew.resolve('new-value')
    await tick()
    await tick()
    expect(sNew()).toEqual({ kind: 'ok', value: 'new-value' })

    // ...then resolve the OLD key's fetch AFTER — the newer key's entry
    // must still hold its own value, untouched.
    dOld.resolve('stale-value')
    await tick()
    await tick()

    expect(sNew()).toEqual({ kind: 'ok', value: 'new-value' })
    expect(sOld()).toEqual({ kind: 'ok', value: 'stale-value' }) // its OWN entry — correct, isolated
    expect(sOld).not.toBe(sNew)
  })
})

// ---------------------------------------------------------------------------
// 2. INVALIDATE-DURING-FLIGHT
//
// FINDING: NativeCache has no public `invalidate()` — nothing analogous
// to query-core's `client.invalidateQueries()`. The only way to force a
// refetch is to reach into the cache (`useNativeCache()` + `cache.attach()`
// + `cache.fetch()`) directly; `nativeQueried` itself exposes no such
// call. The tests below use that internal path since it's the only one
// that exists, and the gap itself is called out in the report.
// ---------------------------------------------------------------------------

describe('2. INVALIDATE-DURING-FLIGHT (via the only mechanism the core exposes)', () => {
  it('invalidating a currently-fetching entry does not start a second fetch, and does not double-settle', async () => {
    const scope = createScope()
    const cache = withScope(scope, () => useNativeCache())
    const key = ['invalidate-in-flight']
    const d = deferred<string>()
    const fetcher = vi.fn(() => d.promise)

    const s = withScope(scope, () => nativeQueried(key, fetcher))
    expect(s()).toEqual({ kind: 'loading' })

    // "invalidate" attempt while in flight — the status guard absorbs it.
    const { entry } = cache.attach(key)
    cache.fetch(entry, fetcher)
    expect(fetcher).toHaveBeenCalledOnce()

    d.resolve('settled-once')
    await tick()
    await tick()

    expect(fetcher).toHaveBeenCalledOnce()
    expect(s()).toEqual({ kind: 'ok', value: 'settled-once' })
  })

  it('invalidating an already-ok entry forces exactly one background refetch, never flashing ok back to loading', async () => {
    const scope = createScope()
    const cache = withScope(scope, () => useNativeCache())
    const key = ['invalidate-fresh']
    let call = 0
    const fetcher = vi.fn(() => Promise.resolve(`v${++call}`))

    const s = withScope(scope, () => nativeQueried(key, fetcher, { staleTime: 60_000 }))
    await tick()
    await tick()
    expect(s()).toEqual({ kind: 'ok', value: 'v1' })

    const { entry } = cache.attach(key)
    cache.fetch(entry, fetcher)
    // no ok -> loading flash from the doomed... er, the SUCCESSFUL revalidate
    expect(s()).toEqual({ kind: 'ok', value: 'v1' })

    await tick()
    await tick()
    expect(s()).toEqual({ kind: 'ok', value: 'v2' })
    expect(fetcher).toHaveBeenCalledTimes(2)
  })
})

// ---------------------------------------------------------------------------
// 3. UNMOUNT MID-EVERYTHING
// ---------------------------------------------------------------------------

describe('3. UNMOUNT MID-EVERYTHING', () => {
  it('KILL: root dispose mid-fetch — a late-resolving promise must NOT write into the disposed entry\'s signal (poisoned set trap)', async () => {
    const scope = createScope() // scope IS the cache's root here
    const d = deferred<string>()
    const s = withScope(scope, () => nativeQueried(['unmount-mid-fetch'], () => d.promise))
    expect(s()).toEqual({ kind: 'loading' })

    const setSpy = vi.spyOn(s, 'set')
    scope.dispose() // tears down the NativeCache itself (same root)

    d.resolve('too-late')
    await tick()
    await tick()

    // Pre-fix: this fired anyway, because dispose() never bumped
    // fetchToken/marked the entry dead — the resolve handler's token
    // check (`myToken !== entry.fetchToken`) was still satisfied.
    expect(setSpy).not.toHaveBeenCalled()
    expect(s()).toEqual({ kind: 'loading' })
  })

  it('KILL: root dispose mid-retry-backoff — the pending retry timer is cancelled, no late retry fires', async () => {
    vi.useFakeTimers()
    try {
      const scope = createScope()
      const fetcher = vi.fn(() => Promise.reject(new Error('down')))
      const s = withScope(scope, () => nativeQueried(['unmount-mid-retry'], fetcher, { retry: 3 }))

      await vi.advanceTimersByTimeAsync(0) // let the first rejection land, schedule retry #1
      expect(fetcher).toHaveBeenCalledOnce()
      expect(vi.getTimerCount()).toBeGreaterThan(0)

      const setSpy = vi.spyOn(s, 'set')
      scope.dispose()

      await vi.advanceTimersByTimeAsync(60_000) // past every possible backoff step
      expect(fetcher).toHaveBeenCalledOnce() // no retry ever fired after dispose
      expect(setSpy).not.toHaveBeenCalled()
      expect(vi.getTimerCount()).toBe(0)
    } finally {
      vi.useRealTimers()
    }
  })

  it('KILL: root dispose mid-push-stream — a source that keeps delivering after its own unsubscribe must not write into the disposed entry', () => {
    const scope = createScope()
    let pushFn: ((v: number) => void) | null = null
    let unsubscribed = false
    const source: PushSource<number> = (push) => {
      pushFn = push
      push(1)
      return () => {
        unsubscribed = true
      }
    }
    const s = withScope(scope, () => nativePush(['unmount-mid-push'], source))
    expect(s()).toEqual({ kind: 'ok', value: 1 })

    const setSpy = vi.spyOn(s, 'set')
    scope.dispose()
    expect(unsubscribed).toBe(true)

    // Naughty source: fires one more push despite having been told to stop
    // (a real-world race — a message already in flight when close() runs).
    pushFn!(2)
    expect(setSpy).not.toHaveBeenCalled()
    expect(s()).toEqual({ kind: 'ok', value: 1 })
  })

  it('no leaks: an unobserved entry is GC\'d at exactly gcTime, tearing down its push subscription and timers', () => {
    vi.useFakeTimers()
    try {
      const root = createScope()
      const child = childOf(root)
      const unsubscribe = vi.fn()
      const source: PushSource<number> = (push) => {
        push(1)
        return unsubscribe
      }
      const cache = withScope(root, () => useNativeCache())
      withScope(child, () => nativePush(['gc-exact'], source, { gcTime: 5000 }))
      child.dispose() // observer released, GC scheduled

      vi.advanceTimersByTime(4999)
      expect(cache.debugHasEntry(['gc-exact'])).toBe(true)
      expect(unsubscribe).not.toHaveBeenCalled()

      vi.advanceTimersByTime(1)
      expect(cache.debugHasEntry(['gc-exact'])).toBe(false)
      expect(unsubscribe).toHaveBeenCalledOnce()
      expect(vi.getTimerCount()).toBe(0)
    } finally {
      vi.useRealTimers()
    }
  })
})

// ---------------------------------------------------------------------------
// 4. RETRY TORTURE (fake timers)
// ---------------------------------------------------------------------------

describe('4. RETRY TORTURE', () => {
  it('backoff schedule is exactly min(1000*2^n, 30s), boundary-exact', async () => {
    vi.useFakeTimers()
    try {
      const scope = createScope()
      const fetcher = vi.fn(() => Promise.reject(new Error('down')))
      withScope(scope, () => nativeQueried(['retry-schedule'], fetcher, { retry: 6 }))

      await vi.advanceTimersByTimeAsync(0)
      expect(fetcher).toHaveBeenCalledTimes(1) // initial attempt

      const expectedDelays = [1000, 2000, 4000, 8000, 16000, 30000] // capped at 30s
      for (let i = 0; i < expectedDelays.length; i++) {
        const delay = expectedDelays[i]!
        await vi.advanceTimersByTimeAsync(delay - 1)
        expect(fetcher).toHaveBeenCalledTimes(i + 1) // not yet — boundary-exact
        await vi.advanceTimersByTimeAsync(1)
        expect(fetcher).toHaveBeenCalledTimes(i + 2)
      }
    } finally {
      vi.useRealTimers()
    }
  })

  it('success mid-schedule cancels every remaining scheduled attempt', async () => {
    vi.useFakeTimers()
    try {
      const scope = createScope()
      let call = 0
      const fetcher = vi.fn(() => {
        call++
        if (call < 3) return Promise.reject(new Error('down'))
        return Promise.resolve('recovered')
      })
      const s = withScope(scope, () => nativeQueried(['retry-recovers'], fetcher, { retry: 10 }))

      await vi.advanceTimersByTimeAsync(0) // attempt 1: fails
      await vi.advanceTimersByTimeAsync(1000) // attempt 2: fails
      await vi.advanceTimersByTimeAsync(2000) // attempt 3: succeeds

      expect(s()).toEqual({ kind: 'ok', value: 'recovered' })
      expect(vi.getTimerCount()).toBe(0) // no dangling retry timer after recovery

      await vi.advanceTimersByTimeAsync(60_000)
      expect(fetcher).toHaveBeenCalledTimes(3) // nothing further ever runs
    } finally {
      vi.useRealTimers()
    }
  })

  it('unmount cancels the timer chain (covered from the disposal angle in §3, re-asserted here for the retry-specific invariant: zero pending timers, retryCount frozen)', async () => {
    vi.useFakeTimers()
    try {
      const scope = createScope()
      const fetcher = vi.fn(() => Promise.reject(new Error('down')))
      withScope(scope, () => nativeQueried(['retry-unmount'], fetcher, { retry: 5 }))
      await vi.advanceTimersByTimeAsync(0)
      expect(fetcher).toHaveBeenCalledOnce()

      scope.dispose()
      await vi.advanceTimersByTimeAsync(120_000)
      expect(fetcher).toHaveBeenCalledOnce()
      expect(vi.getTimerCount()).toBe(0)
    } finally {
      vi.useRealTimers()
    }
  })
})

// ---------------------------------------------------------------------------
// 5. PUSH+PULL INTERLEAVE
// ---------------------------------------------------------------------------

describe('5. PUSH+PULL INTERLEAVE', () => {
  it('fetch resolves, push arrives, invalidate-refetch runs while pushes continue — LiveState never regresses ok -> loading', async () => {
    const scope = createScope()
    const cache = withScope(scope, () => useNativeCache())
    const key = ['interleave-no-regress']

    const s1 = withScope(scope, () => nativeQueried(key, () => Promise.resolve('pulled')))
    await tick()
    await tick()
    expect(s1()).toEqual({ kind: 'ok', value: 'pulled' })

    let pushFn: ((v: string) => void) | null = null
    const source: PushSource<string> = (push) => {
      pushFn = push
      return () => {}
    }
    withScope(scope, () => nativePush(key, source))
    pushFn!('pushed-1')
    expect(s1()).toEqual({ kind: 'ok', value: 'pushed-1' })

    // "invalidate" refetch while push is live — must not flash back to loading.
    const { entry } = cache.attach(key)
    const dRefetch = deferred<string>()
    cache.fetch(entry, () => dRefetch.promise)
    expect(s1()).toEqual({ kind: 'ok', value: 'pushed-1' }) // no regression

    pushFn!('pushed-2') // pushes keep flowing during the refetch
    expect(s1()).toEqual({ kind: 'ok', value: 'pushed-2' })

    dRefetch.resolve('refetched')
    await tick()
    await tick()
    // Documented semantics: last SETTLED write wins. The refetch settles
    // after the last push here, so it wins — deterministic by completion
    // order, not by wall-clock arrival order. No LiveState kind ever
    // regressed to 'loading' throughout.
    expect(s1()).toEqual({ kind: 'ok', value: 'refetched' })
  })

  it('DOCUMENTED: a slow pull started BEFORE a push can still clobber a push that arrived and settled EARLIER, because the pull applies no timestamp reconciliation against push writes — last-SETTLED wins, not last-arrived', async () => {
    const scope = createScope()
    const key = ['interleave-clobber']
    const dPull = deferred<string>()
    withScope(scope, () => nativeQueried(key, () => dPull.promise))

    let pushFn: ((v: string) => void) | null = null
    const source: PushSource<string> = (push) => {
      pushFn = push
      return () => {}
    }
    const s = withScope(scope, () => nativePush(key, source))

    // push arrives and settles WHILE the pull is still in flight...
    pushFn!('push-value')
    expect(s()).toEqual({ kind: 'ok', value: 'push-value' })

    // ...then the OLDER pull settles LAST and overwrites it. This is the
    // native core's actual, current behavior — flagged, not silently
    // patched: fixing it would mean threading dataUpdatedAt comparisons
    // through the fetch-resolve path, a design decision for the owner,
    // not a bug this torture pass unilaterally fixes.
    dPull.resolve('pull-value')
    await tick()
    await tick()
    expect(s()).toEqual({ kind: 'ok', value: 'pull-value' })
  })
})

// ---------------------------------------------------------------------------
// 6. DEHYDRATE UNDER FIRE
// ---------------------------------------------------------------------------

describe('6. DEHYDRATE UNDER FIRE', () => {
  it('dehydrating while a fetch is in-flight and a push is live: only settled (ok) entries serialize; hydrate reproduces exactly those; no dangling write lands in the new root', async () => {
    const server = createScope()
    const serverCache = withScope(server, () => useNativeCache())

    // one entry mid-flight (loading) — must NOT appear in the dehydrated payload
    const dInFlight = deferred<string>()
    withScope(server, () => nativeQueried(['dehydrate-in-flight'], () => dInFlight.promise))

    // one entry settled via push — must appear
    const source: PushSource<string> = (push) => {
      push('live-value')
      return () => {}
    }
    withScope(server, () => nativePush(['dehydrate-push'], source))

    const dehydrated = dehydrateNative(serverCache)
    expect(dehydrated).toHaveLength(1)
    expect(dehydrated[0]!.key).toEqual(['dehydrate-push'])
    expect(dehydrated[0]!.value).toBe('live-value')

    server.dispose()

    const client_ = createScope()
    const clientCache = withScope(client_, () => useNativeCache())
    hydrateNative(clientCache, dehydrated)

    expect(clientCache.debugHasEntry(['dehydrate-push'])).toBe(true)
    expect(clientCache.debugHasEntry(['dehydrate-in-flight'])).toBe(false)

    // the abandoned in-flight fetch resolving AFTER dehydrate/hydrate must
    // not touch the fresh root at all — separate NativeCache instance,
    // separate CacheEntry object entirely.
    dInFlight.resolve('too-late-for-a-dead-root')
    await tick()
    await tick()
    expect(clientCache.debugHasEntry(['dehydrate-in-flight'])).toBe(false)
  })
})

// ---------------------------------------------------------------------------
// 7. DIFFERENTIAL — nativeQueried vs queried (query-core), identical scripts
// ---------------------------------------------------------------------------

function snapshot(sig: { (): LiveState<unknown> }): unknown {
  const v = sig()
  if (v.kind === 'error') return { kind: 'error', message: v.error.message }
  return v
}

async function scenarioSuccess(query: typeof nativeQueried | typeof queried) {
  const scope = createScope()
  const s = withScope(scope, () => query(['diff-success'], () => Promise.resolve(42)))
  const before = snapshot(s)
  await tick()
  await tick()
  return [before, snapshot(s)]
}

async function scenarioError(query: typeof nativeQueried | typeof queried) {
  const scope = createScope()
  const s = withScope(scope, () =>
    query(['diff-error'], () => Promise.reject(new Error('boom')), { retry: false }),
  )
  await tick()
  await tick()
  return [snapshot(s)]
}

async function scenarioDedup(query: typeof nativeQueried | typeof queried) {
  const scope = createScope()
  const fetcher = vi.fn(() => Promise.resolve('shared'))
  const [a, b] = withScope(scope, () => [
    query(['diff-dedup'], fetcher),
    query(['diff-dedup'], fetcher),
  ])
  await tick()
  await tick()
  return [snapshot(a), snapshot(b), fetcher.mock.calls.length]
}

async function scenarioRemountFresh(query: typeof nativeQueried | typeof queried) {
  const root = createScope()
  const fetcher = vi.fn(() => Promise.resolve('cached'))
  const first = childOf(root)
  const s1 = withScope(first, () => query(['diff-remount'], fetcher, { staleTime: 60_000 }))
  await tick()
  await tick()
  const snap1 = snapshot(s1)
  first.dispose()
  const second = childOf(root)
  const s2 = withScope(second, () => query(['diff-remount'], fetcher, { staleTime: 60_000 }))
  return [snap1, snapshot(s2), fetcher.mock.calls.length]
}

async function scenarioStaleRevalidate(query: typeof nativeQueried | typeof queried) {
  const root = createScope()
  let call = 0
  const fetcher = vi.fn(() => Promise.resolve(`v${++call}`))
  const first = childOf(root)
  const s1 = withScope(first, () => query(['diff-stale'], fetcher, { staleTime: 0 }))
  await tick()
  await tick()
  const snapFirst = snapshot(s1)
  first.dispose()
  const second = childOf(root)
  const s2 = withScope(second, () => query(['diff-stale'], fetcher, { staleTime: 0 }))
  const snapImmediate = snapshot(s2)
  await tick()
  await tick()
  return [snapFirst, snapImmediate, snapshot(s2), fetcher.mock.calls.length]
}

async function scenarioIsolatedRoots(query: typeof nativeQueried | typeof queried) {
  const rootA = createScope()
  const rootB = createScope()
  const fetcherA = vi.fn(() => Promise.resolve('A'))
  const fetcherB = vi.fn(() => Promise.resolve('B'))
  const sA = withScope(rootA, () => query(['diff-iso'], fetcherA))
  const sB = withScope(rootB, () => query(['diff-iso'], fetcherB))
  await tick()
  await tick()
  return [snapshot(sA), snapshot(sB)]
}

describe('7. DIFFERENTIAL — nativeQueried vs queried, identical scripted sources', () => {
  const scenarios: Array<[string, (q: typeof nativeQueried | typeof queried) => Promise<unknown>]> = [
    ['success', scenarioSuccess],
    ['error', scenarioError],
    ['dedup', scenarioDedup],
    ['remount-fresh', scenarioRemountFresh],
    ['stale-then-revalidate', scenarioStaleRevalidate],
    ['isolated-roots', scenarioIsolatedRoots],
  ]

  for (const [name, scenario] of scenarios) {
    it(`${name}: native and query-core produce the same state sequence`, async () => {
      const nativeResult = await scenario(nativeQueried)
      const queryResult = await scenario(queried)
      expect(nativeResult).toEqual(queryResult)
    })
  }
})

// ---------------------------------------------------------------------------
// 8. TWO-ROOTS + CLOCK SKEW
// ---------------------------------------------------------------------------

describe('8. TWO-ROOTS + CLOCK SKEW', () => {
  it('staleTime boundary is exact: fresh at staleTime-1, stale at staleTime and staleTime+1', async () => {
    vi.useFakeTimers()
    try {
      const root = createScope()
      const fetcher = vi.fn(() => Promise.resolve('v'))
      const first = childOf(root)
      withScope(first, () => nativeQueried(['clock-skew'], fetcher, { staleTime: 1000 }))
      await vi.advanceTimersByTimeAsync(0)
      expect(fetcher).toHaveBeenCalledOnce()
      first.dispose()

      // t = staleTime - 1: fresh, no refetch
      await vi.advanceTimersByTimeAsync(999)
      const atMinus1 = childOf(root)
      withScope(atMinus1, () => nativeQueried(['clock-skew'], fetcher, { staleTime: 1000 }))
      await vi.advanceTimersByTimeAsync(0)
      expect(fetcher).toHaveBeenCalledOnce() // still just the one
      atMinus1.dispose()

      // t = staleTime exactly: stale (isFresh is strict `<`), refetches
      await vi.advanceTimersByTimeAsync(1)
      const atExact = childOf(root)
      withScope(atExact, () => nativeQueried(['clock-skew'], fetcher, { staleTime: 1000 }))
      await vi.advanceTimersByTimeAsync(0)
      expect(fetcher).toHaveBeenCalledTimes(2)
      atExact.dispose()

      // t = staleTime + 1: still stale, refetches again
      await vi.advanceTimersByTimeAsync(1000)
      const atPlus1 = childOf(root)
      withScope(atPlus1, () => nativeQueried(['clock-skew'], fetcher, { staleTime: 1000 }))
      await vi.advanceTimersByTimeAsync(0)
      expect(fetcher).toHaveBeenCalledTimes(3)
    } finally {
      vi.useRealTimers()
    }
  })

  it('two roots never share entries even under identical keys and interleaved GC timers', () => {
    vi.useFakeTimers()
    try {
      const rootA = createScope()
      const rootB = createScope()
      const cacheA = withScope(rootA, () => useNativeCache())
      const cacheB = withScope(rootB, () => useNativeCache())
      expect(cacheA).not.toBe(cacheB)

      const childA = childOf(rootA)
      const childB = childOf(rootB)
      withScope(childA, () => nativeQueried(['skew-key'], () => Promise.resolve('A'), { gcTime: 1000 }))
      withScope(childB, () => nativeQueried(['skew-key'], () => Promise.resolve('B'), { gcTime: 5000 }))

      childA.dispose() // schedules A's GC at t+1000
      vi.advanceTimersByTime(500)
      childB.dispose() // schedules B's GC at t+500+5000

      vi.advanceTimersByTime(500) // t = 1000: A's GC fires
      expect(cacheA.debugHasEntry(['skew-key'])).toBe(false)
      expect(cacheB.debugHasEntry(['skew-key'])).toBe(true) // untouched by A's eviction

      vi.advanceTimersByTime(4500) // t = 5500: B's GC fires
      expect(cacheB.debugHasEntry(['skew-key'])).toBe(false)
    } finally {
      vi.useRealTimers()
    }
  })
})
