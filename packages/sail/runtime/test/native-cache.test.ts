// native-cache — SPIKE test file. Ports the 8 `queried.test.ts` cases
// against the native core (same assertions, swapped import — the
// original file is untouched) plus new coverage for the one thing
// query-core cannot express: a push source, and pull+push interleaved
// on one key.

import { describe, expect, it, vi } from 'vitest'
import { createScope, withScope } from '../src/effect-scope'
import {
  nativeQueried,
  nativePush,
  useNativeCache,
  dehydrateNative,
  hydrateNative,
  type PushSource,
} from '../src/native-cache'

const tick = () => new Promise((r) => setTimeout(r, 0))

describe('nativeQueried — ported queried.test.ts cases', () => {
  it('starts in loading, resolves to ok', async () => {
    const scope = createScope()
    const s = withScope(scope, () => nativeQueried(['q1'], () => Promise.resolve(42)))
    expect(s().kind).toBe('loading')
    await tick()
    await tick()
    const v = s()
    expect(v.kind).toBe('ok')
    if (v.kind === 'ok') expect(v.value).toBe(42)
  })

  it('maps rejection to the error state', async () => {
    const scope = createScope()
    const s = withScope(scope, () =>
      nativeQueried(['q-err'], () => Promise.reject(new Error('boom')), { retry: false }),
    )
    await tick()
    await tick()
    const v = s()
    expect(v.kind).toBe('error')
    if (v.kind === 'error') expect(v.error.message).toBe('boom')
  })

  it('dedup: two readers of the same key under one root share one fetch', async () => {
    const scope = createScope()
    const fetcher = vi.fn(() => Promise.resolve('shared'))
    const [a, b] = withScope(scope, () => [
      nativeQueried(['dedup'], fetcher),
      nativeQueried(['dedup'], fetcher),
    ])
    await tick()
    await tick()
    expect(fetcher).toHaveBeenCalledOnce()
    const av = a()
    const bv = b()
    expect(av.kind).toBe('ok')
    expect(bv.kind).toBe('ok')
    if (av.kind === 'ok' && bv.kind === 'ok') {
      expect(av.value).toBe('shared')
      expect(bv.value).toBe('shared')
    }
  })

  it('a remount within staleTime paints instantly (cache hit, no refetch)', async () => {
    const root = createScope()
    const fetcher = vi.fn(() => Promise.resolve('cached'))

    const first = createScope()
    Object.defineProperty(first, 'parent', { value: root, writable: false })
    const s1 = withScope(first, () => nativeQueried(['remount'], fetcher, { staleTime: 60_000 }))
    await tick()
    await tick()
    expect(s1()).toEqual({ kind: 'ok', value: 'cached' })
    first.dispose()

    const second = createScope()
    Object.defineProperty(second, 'parent', { value: root, writable: false })
    const s2 = withScope(second, () => nativeQueried(['remount'], fetcher, { staleTime: 60_000 }))
    // Cache hit: value is available synchronously, no second fetch.
    expect(s2()).toEqual({ kind: 'ok', value: 'cached' })
    expect(fetcher).toHaveBeenCalledOnce()
  })

  it('stale data serves immediately, then revalidates in the background', async () => {
    const root = createScope()
    let call = 0
    const fetcher = vi.fn(() => Promise.resolve(`v${++call}`))

    const first = createScope()
    Object.defineProperty(first, 'parent', { value: root, writable: false })
    const s1 = withScope(first, () => nativeQueried(['stale'], fetcher, { staleTime: 0 }))
    await tick()
    await tick()
    expect(s1()).toEqual({ kind: 'ok', value: 'v1' })
    first.dispose()

    const second = createScope()
    Object.defineProperty(second, 'parent', { value: root, writable: false })
    const s2 = withScope(second, () => nativeQueried(['stale'], fetcher, { staleTime: 0 }))
    // Stale-while-revalidate: serves the cached v1 immediately...
    expect(s2()).toEqual({ kind: 'ok', value: 'v1' })
    await tick()
    await tick()
    // ...then the background refetch lands.
    expect(s2()).toEqual({ kind: 'ok', value: 'v2' })
    expect(fetcher).toHaveBeenCalledTimes(2)
  })

  it('owner-tree isolation: two roots produce two independent caches', async () => {
    const rootA = createScope()
    const rootB = createScope()
    const fetcherA = vi.fn(() => Promise.resolve('A'))
    const fetcherB = vi.fn(() => Promise.resolve('B'))

    const clientA = withScope(rootA, () => useNativeCache())
    const clientB = withScope(rootB, () => useNativeCache())
    expect(clientA).not.toBe(clientB)

    const sA = withScope(rootA, () => nativeQueried(['iso'], fetcherA))
    const sB = withScope(rootB, () => nativeQueried(['iso'], fetcherB))
    await tick()
    await tick()
    expect(sA()).toEqual({ kind: 'ok', value: 'A' })
    expect(sB()).toEqual({ kind: 'ok', value: 'B' })
    expect(fetcherA).toHaveBeenCalledOnce()
    expect(fetcherB).toHaveBeenCalledOnce()
  })

  it('unmount unsubscribes (no leak)', async () => {
    const scope = createScope()
    const cache = withScope(scope, () => useNativeCache())
    const s = withScope(scope, () => nativeQueried(['unmount'], () => Promise.resolve('x')))
    await tick()
    await tick()
    expect(s()).toEqual({ kind: 'ok', value: 'x' })

    // Captured by reference: `scope` doubles as the cache's own root here,
    // so disposing it also disposes the cache — a map lookup post-dispose
    // would find nothing. The entry OBJECT is mutated in place by the
    // observer's release() before the cache evicts it, so a reference taken
    // now still reflects that mutation afterward.
    const entryRef = cache.debugEntryRef(['unmount'])
    expect(entryRef?.observers).toBeGreaterThan(0)

    scope.dispose()
    expect(entryRef?.observers).toBe(0)
  })

  it('SSR dehydrate on one root seeds another via hydrate', async () => {
    const server = createScope()
    const serverCache = withScope(server, () => useNativeCache())
    withScope(server, () => nativeQueried(['ssr-key'], () => Promise.resolve('server-value')))
    await tick()
    await tick()

    const dehydrated = dehydrateNative(serverCache)
    expect(JSON.stringify(dehydrated)).toContain('server-value')
    server.dispose()

    const client_ = createScope()
    const clientCache = withScope(client_, () => useNativeCache())
    hydrateNative(clientCache, dehydrated)

    const fetcherThatShouldNotRun = vi.fn(() => Promise.resolve('SHOULD NOT RUN'))
    const s = withScope(client_, () =>
      nativeQueried(['ssr-key'], fetcherThatShouldNotRun, { staleTime: 60_000 }),
    )
    // Hydrated value is available synchronously — no client fetch needed.
    expect(s()).toEqual({ kind: 'ok', value: 'server-value' })
    expect(fetcherThatShouldNotRun).not.toHaveBeenCalled()
  })
})

describe('nativePush — the thing query-core cannot express', () => {
  it('loading until first push, ok(latest) per push thereafter', () => {
    const scope = createScope()
    let pushFn: ((v: number) => void) | null = null
    const source: PushSource<number> = (push) => {
      pushFn = push
      return () => {}
    }
    const s = withScope(scope, () => nativePush(['live-count'], source))
    expect(s()).toEqual({ kind: 'loading' })

    pushFn!(1)
    expect(s()).toEqual({ kind: 'ok', value: 1 })
    pushFn!(2)
    expect(s()).toEqual({ kind: 'ok', value: 2 })
  })

  it('error on failed close', () => {
    const scope = createScope()
    let failFn: ((err: unknown) => void) | null = null
    const source: PushSource<number> = (_push, fail) => {
      failFn = fail
      return () => {}
    }
    const s = withScope(scope, () => nativePush(['live-fail'], source))
    failFn!(new Error('closed'))
    const v = s()
    expect(v.kind).toBe('error')
    if (v.kind === 'error') expect(v.error.message).toBe('closed')
  })

  it('unsubscribe on scope disposal (after GC, once the entry is actually collected)', () => {
    vi.useFakeTimers()
    try {
      // Root/child split: the calling scope (child) disposes independently
      // of the cache's owning root, so GC on the ENTRY (gated by gcTime)
      // is observable separately from the OBSERVER release (immediate).
      const root = createScope()
      const child = createScope()
      Object.defineProperty(child, 'parent', { value: root, writable: false })

      const unsubscribe = vi.fn()
      const source: PushSource<number> = (push) => {
        push(1)
        return unsubscribe
      }
      const cache = withScope(root, () => useNativeCache())
      withScope(child, () => nativePush(['live-unsub'], source, { gcTime: 1000 }))
      expect(unsubscribe).not.toHaveBeenCalled()
      expect(cache.debugObserverCount(['live-unsub'])).toBe(1)

      child.dispose()
      // Observer count drops immediately; the push subscription itself
      // tears down only once the entry is actually GC'd.
      expect(cache.debugObserverCount(['live-unsub'])).toBe(0)
      expect(unsubscribe).not.toHaveBeenCalled()

      vi.advanceTimersByTime(1000)
      expect(unsubscribe).toHaveBeenCalledOnce()
      expect(cache.debugHasEntry(['live-unsub'])).toBe(false)
    } finally {
      vi.useRealTimers()
    }
  })

  it('two readers of the same key share one subscription', () => {
    const scope = createScope()
    let pushCalls = 0
    const source: PushSource<number> = (push) => {
      pushCalls++
      push(7)
      return () => {}
    }
    const [a, b] = withScope(scope, () => [
      nativePush(['shared-push'], source),
      nativePush(['shared-push'], source),
    ])
    expect(pushCalls).toBe(1)
    expect(a()).toEqual({ kind: 'ok', value: 7 })
    expect(b()).toEqual({ kind: 'ok', value: 7 })
  })
})

describe('interleaved pull+push on one key', () => {
  it('a fetch fills the entry, then a push source keeps writing into the same entry', async () => {
    const scope = createScope()
    const key = ['unified']

    const s1 = withScope(scope, () => nativeQueried(key, () => Promise.resolve('pulled')))
    await tick()
    await tick()
    expect(s1()).toEqual({ kind: 'ok', value: 'pulled' })

    let pushFn: ((v: string) => void) | null = null
    const source: PushSource<string> = (push) => {
      pushFn = push
      return () => {}
    }
    // Same key, same entry — nativePush reads the pulled value first...
    const s2 = withScope(scope, () => nativePush(key, source))
    expect(s2()).toEqual({ kind: 'ok', value: 'pulled' })
    expect(s1).toBe(s2) // literally the same signal — one entry, two readers

    // ...then a push overwrites it, visible to BOTH readers (same signal).
    pushFn!('pushed')
    expect(s1()).toEqual({ kind: 'ok', value: 'pushed' })
    expect(s2()).toEqual({ kind: 'ok', value: 'pushed' })
  })
})
