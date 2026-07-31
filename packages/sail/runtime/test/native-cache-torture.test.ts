// native-cache-torture — hammer the native core on paths where young caches
// die: races, invalidate-during-flight, unmount mid-everything, retry
// backoff, push+pull interleave, dehydrate under fire, and staleTime
// clock-skew exactness.
//
// Contract: dispose/GC must mark entries dead so in-flight fetch/push
// cannot write after teardown (`entry.disposed`). `invalidate(prefix)`
// and AbortSignal wiring abort real in-flight work, not just discard results.

import { describe, expect, it, vi } from 'vitest'
import { createScope, withScope, type EffectScope } from '../src/effect-scope'
import type { LiveState } from '../src/live-state'
import {
  nativeQueried,
  nativePush,
  useNativeCache,
  dehydrateNative,
  hydrateNative,
  type PushSource,
} from '../src/native-cache'

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
// 2. INVALIDATE
// ---------------------------------------------------------------------------

describe('2. INVALIDATE', () => {
  it('invalidating a currently-fetching entry aborts it, discards its result, and starts exactly one fresh fetch', async () => {
    const scope = createScope()
    const cache = withScope(scope, () => useNativeCache())
    const key = ['invalidate-in-flight']
    const d1 = deferred<string>()
    const d2 = deferred<string>()
    let call = 0
    const fetcher = vi.fn((signal: AbortSignal) => {
      call++
      signal.addEventListener('abort', () => {})
      return call === 1 ? d1.promise : d2.promise
    })

    const s = withScope(scope, () => nativeQueried(key, fetcher))
    expect(s()).toEqual({ kind: 'loading' })
    expect(fetcher).toHaveBeenCalledOnce()
    const firstSignal = fetcher.mock.calls[0]![0]!
    expect(firstSignal.aborted).toBe(false)

    cache.invalidate(key)
    expect(firstSignal.aborted).toBe(true) // real abort, not just a discarded result
    expect(fetcher).toHaveBeenCalledTimes(2) // fresh fetch started immediately

    d1.resolve('too-late') // the aborted attempt's eventual resolution must never land
    await tick()
    await tick()
    expect(s()).toEqual({ kind: 'loading' }) // still waiting on attempt #2

    d2.resolve('settled')
    await tick()
    await tick()
    expect(s()).toEqual({ kind: 'ok', value: 'settled' })
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

    cache.invalidate(key)
    // no ok -> loading flash from the background revalidate
    expect(s()).toEqual({ kind: 'ok', value: 'v1' })

    await tick()
    await tick()
    expect(s()).toEqual({ kind: 'ok', value: 'v2' })
    expect(fetcher).toHaveBeenCalledTimes(2)
  })

  it('invalidating an unobserved-but-resident entry marks it stale but fetches nothing until the next attach', async () => {
    const root = createScope()
    const cache = withScope(root, () => useNativeCache())
    const key = ['invalidate-unobserved']
    let call = 0
    const fetcher = vi.fn(() => Promise.resolve(`v${++call}`))

    const child = childOf(root)
    withScope(child, () => nativeQueried(key, fetcher, { staleTime: 60_000 }))
    await tick()
    await tick()
    child.dispose() // observer released; entry resident (gcTime default), zero observers

    cache.invalidate(key)
    await tick()
    await tick()
    expect(fetcher).toHaveBeenCalledOnce() // invalidate itself fetched nothing

    const second = childOf(root)
    const s2 = withScope(second, () => nativeQueried(key, fetcher, { staleTime: 60_000 }))
    await tick()
    await tick()
    expect(fetcher).toHaveBeenCalledTimes(2) // the NEXT attach sees isFresh() === false and refetches
    expect(s2()).toEqual({ kind: 'ok', value: 'v2' })
  })

  it('invalidating a push-fed entry leaves it untouched — no refetch, stream keeps writing', async () => {
    const scope = createScope()
    const cache = withScope(scope, () => useNativeCache())
    const key = ['invalidate-push']
    let pushFn: ((v: string) => void) | null = null
    const source: PushSource<string> = (push) => {
      pushFn = push
      push('initial')
      return () => {}
    }
    const s = withScope(scope, () => nativePush(key, source))
    expect(s()).toEqual({ kind: 'ok', value: 'initial' })

    cache.invalidate(key)
    expect(s()).toEqual({ kind: 'ok', value: 'initial' }) // untouched

    pushFn!('still-flowing')
    expect(s()).toEqual({ kind: 'ok', value: 'still-flowing' }) // stream unaffected
  })

  it('invalidating under a disposed root is a no-op — no timer leak, no thrown error', () => {
    const scope = createScope()
    const cache = withScope(scope, () => useNativeCache())
    withScope(scope, () => nativeQueried(['invalidate-disposed'], () => Promise.resolve('v')))
    scope.dispose()

    expect(() => cache.invalidate(['invalidate-disposed'])).not.toThrow()
  })

  it('invalidate(prefix) matches by namespace: a shorter prefix invalidates every deeper key, a sibling namespace is untouched', async () => {
    const scope = createScope()
    const cache = withScope(scope, () => useNativeCache())
    let listCalls = 0
    let detailCalls = 0
    const sList = withScope(scope, () =>
      nativeQueried(['crm', 'deals', 'list'], () => Promise.resolve(`list-${++listCalls}`), {
        staleTime: 60_000,
      }),
    )
    const sDetail = withScope(scope, () =>
      nativeQueried(['crm', 'deals', 'detail', 1], () => Promise.resolve(`detail-${++detailCalls}`), {
        staleTime: 60_000,
      }),
    )
    const sBilling = withScope(scope, () =>
      nativeQueried(['billing', 'summary'], () => Promise.resolve('billing-untouched'), {
        staleTime: 60_000,
      }),
    )
    await tick()
    await tick()

    cache.invalidate(['crm', 'deals']) // namespace prefix — matches both list and detail
    await tick()
    await tick()

    expect(listCalls).toBe(2)
    expect(detailCalls).toBe(2)
    expect(sBilling()).toEqual({ kind: 'ok', value: 'billing-untouched' }) // sibling namespace, untouched
    expect(sList()).toEqual({ kind: 'ok', value: 'list-2' })
    expect(sDetail()).toEqual({ kind: 'ok', value: 'detail-2' })
  })

  // A differential test proving this matches query-core's
  // `invalidateQueries` transition sequence (before -> during-revalidate,
  // no loading flash -> settled) ran here pre-cutover and passed byte-
  // for-byte; see query-slice-report.md. Removed along with the
  // @tanstack/query-core dependency itself (section 7 below explains the
  // same retirement for the broader differential suite).
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
// 7. DIFFERENTIAL — RETIRED. Ran nativeQueried vs the query-core-backed twin
// through 6 identical scripted scenarios (success, error, dedup, remount-
// fresh, stale-then-revalidate, isolated-roots) and asserted byte-identical
// state sequences — all 6 passed before this cutover (see
// query-slice-report.md). Removed along with @tanstack/query-core and
// queried-querycore.ts once that parity was proven; nothing left to diff
// against post-swap.
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// 9. ABORTSIGNAL — dispose/GC/supersede abort REAL network work, not just
// the eventual result. The cache owns one AbortController per fetch
// attempt (initial or retry) and passes its `.signal` to the fetcher;
// section 2 above already covers the invalidate-triggered abort.
// ---------------------------------------------------------------------------

describe('9. ABORTSIGNAL', () => {
  it('root dispose mid-fetch aborts the signal the fetcher was given', () => {
    const scope = createScope()
    let capturedSignal: AbortSignal | null = null
    const d = deferred<string>()
    withScope(scope, () =>
      nativeQueried(['abort-on-dispose'], (signal) => {
        capturedSignal = signal
        return d.promise
      }),
    )
    expect(capturedSignal!.aborted).toBe(false)

    scope.dispose()
    expect(capturedSignal!.aborted).toBe(true)
  })

  it('GC eviction of an unobserved entry aborts its in-flight fetch signal', () => {
    vi.useFakeTimers()
    try {
      const root = createScope()
      const child = childOf(root)
      let capturedSignal: AbortSignal | null = null
      const d = deferred<string>()
      withScope(child, () =>
        nativeQueried(['abort-on-gc'], (signal) => {
          capturedSignal = signal
          return d.promise
        }, { gcTime: 1000 }),
      )
      child.dispose() // released, GC scheduled — fetch still in flight
      expect(capturedSignal!.aborted).toBe(false)

      vi.advanceTimersByTime(1000)
      expect(capturedSignal!.aborted).toBe(true)
    } finally {
      vi.useRealTimers()
    }
  })

  it('each retry attempt gets its own fresh AbortSignal — an earlier attempt aborting does not abort a later one', async () => {
    vi.useFakeTimers()
    try {
      const scope = createScope()
      const signals: AbortSignal[] = []
      let call = 0
      const fetcher = vi.fn((signal: AbortSignal) => {
        signals.push(signal)
        call++
        return call < 2 ? Promise.reject(new Error('down')) : Promise.resolve('recovered')
      })
      withScope(scope, () => nativeQueried(['abort-per-retry'], fetcher, { retry: 3 }))

      await vi.advanceTimersByTimeAsync(0) // attempt 1 fails
      await vi.advanceTimersByTimeAsync(1000) // attempt 2 succeeds

      expect(signals).toHaveLength(2)
      expect(signals[0]).not.toBe(signals[1])
      expect(signals[1]!.aborted).toBe(false) // the winning attempt's signal is untouched
    } finally {
      vi.useRealTimers()
    }
  })

  it('a fetch that resolves after being aborted does not overwrite a later successful value (real cancellation, not just token-discard)', async () => {
    const scope = createScope()
    const cache = withScope(scope, () => useNativeCache())
    const key = ['abort-late-resolve']
    const d1 = deferred<string>()
    const d2 = deferred<string>()
    let call = 0
    const fetcher = vi.fn(() => (++call === 1 ? d1.promise : d2.promise))

    const s = withScope(scope, () => nativeQueried(key, fetcher))
    cache.invalidate(key) // aborts attempt #1, starts attempt #2

    d2.resolve('second') // second settles first
    await tick()
    await tick()
    expect(s()).toEqual({ kind: 'ok', value: 'second' })

    d1.resolve('first-late') // aborted attempt resolves anyway (fetcher ignored the signal)
    await tick()
    await tick()
    expect(s()).toEqual({ kind: 'ok', value: 'second' }) // untouched — token-guarded regardless
  })
})
