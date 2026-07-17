// queried() — @tanstack/query-core-backed LiveState adapter.
//
// Verifies the bet: two readers of the same key share one fetch
// (dedup), a remount within staleTime paints instantly from cache,
// stale data serves-then-revalidates, the error path maps correctly,
// two owner-tree roots get two independent caches (mirrors
// store.test.ts's two-roots test), unmount unsubscribes (no leak),
// and query-core's own dehydrate/hydrate seeds one root from another.

import { describe, expect, it, vi } from 'vitest'
import { dehydrate, hydrate } from '@tanstack/query-core'
import { createScope, withScope } from '../src/effect-scope'
import { queried } from '../src/queried'
import { useQueryClient } from '../src/query-client'

const tick = () => new Promise((r) => setTimeout(r, 0))

describe('queried', () => {
  it('starts in loading, resolves to ok', async () => {
    const scope = createScope()
    const s = withScope(scope, () => queried(['q1'], () => Promise.resolve(42)))
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
      queried(['q-err'], () => Promise.reject(new Error('boom')), { retry: false }),
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
      queried(['dedup'], fetcher),
      queried(['dedup'], fetcher),
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
    const s1 = withScope(first, () => queried(['remount'], fetcher, { staleTime: 60_000 }))
    await tick()
    await tick()
    expect(s1()).toEqual({ kind: 'ok', value: 'cached' })
    first.dispose()

    const second = createScope()
    Object.defineProperty(second, 'parent', { value: root, writable: false })
    const s2 = withScope(second, () => queried(['remount'], fetcher, { staleTime: 60_000 }))
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
    const s1 = withScope(first, () => queried(['stale'], fetcher, { staleTime: 0 }))
    await tick()
    await tick()
    expect(s1()).toEqual({ kind: 'ok', value: 'v1' })
    first.dispose()

    const second = createScope()
    Object.defineProperty(second, 'parent', { value: root, writable: false })
    const s2 = withScope(second, () => queried(['stale'], fetcher, { staleTime: 0 }))
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

    const clientA = withScope(rootA, () => useQueryClient())
    const clientB = withScope(rootB, () => useQueryClient())
    expect(clientA).not.toBe(clientB)

    const sA = withScope(rootA, () => queried(['iso'], fetcherA))
    const sB = withScope(rootB, () => queried(['iso'], fetcherB))
    await tick()
    await tick()
    expect(sA()).toEqual({ kind: 'ok', value: 'A' })
    expect(sB()).toEqual({ kind: 'ok', value: 'B' })
    expect(fetcherA).toHaveBeenCalledOnce()
    expect(fetcherB).toHaveBeenCalledOnce()
  })

  it('unmount unsubscribes the QueryObserver (no leak)', async () => {
    const scope = createScope()
    const client = withScope(scope, () => useQueryClient())
    const s = withScope(scope, () => queried(['unmount'], () => Promise.resolve('x')))
    await tick()
    await tick()
    expect(s()).toEqual({ kind: 'ok', value: 'x' })

    const query = client.getQueryCache().find({ queryKey: ['unmount'] })
    expect(query?.observers.length).toBeGreaterThan(0)

    scope.dispose()
    expect(query?.observers.length).toBe(0)
  })

  it('SSR dehydrate on one root seeds another via hydrate', async () => {
    const server = createScope()
    const serverClient = withScope(server, () => useQueryClient())
    withScope(server, () => queried(['ssr-key'], () => Promise.resolve('server-value')))
    await tick()
    await tick()

    const dehydrated = dehydrate(serverClient)
    expect(JSON.stringify(dehydrated)).toContain('server-value')
    server.dispose()

    const client_ = createScope()
    const clientClient = withScope(client_, () => useQueryClient())
    hydrate(clientClient, dehydrated)

    const fetcherThatShouldNotRun = vi.fn(() => Promise.resolve('SHOULD NOT RUN'))
    const s = withScope(client_, () =>
      queried(['ssr-key'], fetcherThatShouldNotRun, { staleTime: 60_000 }),
    )
    // Hydrated value is available synchronously — no client fetch needed.
    expect(s()).toEqual({ kind: 'ok', value: 'server-value' })
    expect(fetcherThatShouldNotRun).not.toHaveBeenCalled()
  })
})
