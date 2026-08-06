// SPEC-HARDENING-1 WI2 — Cache conformance suite (S8).
//
// Four named tanstack-independence behaviors. Each is fail-when-broken:
//  1. dedupe — concurrent identical keys share one fetch
//  2. invalidation — prefix clears exactly the tagged namespace
//  3. revalidate-never-flickers — stale ok stays rendered (DOM stability)
//  4. SSR hydration — server-fetched state does not refetch on mount
//
// Flicker is asserted via DOM textContent stability (not setTimeout races).

import { describe, expect, it, vi } from 'vitest'
import { effect } from '../src/signal'
import { createScope, withScope, createChildScope } from '../src/effect-scope'
import {
  nativeQueried,
  useNativeCache,
  dehydrateNative,
  hydrateNative,
} from '../src/native-cache'

const tick = () => new Promise((r) => setTimeout(r, 0))

describe('cache-conformance / dedupe', () => {
  it('concurrent identical queries share one fetch', async () => {
    const scope = createScope()
    const fetcher = vi.fn(() => Promise.resolve('shared'))
    const [a, b] = withScope(scope, () => [
      nativeQueried(['dedupe-conf'], fetcher),
      nativeQueried(['dedupe-conf'], fetcher),
    ])
    await tick()
    await tick()
    expect(fetcher).toHaveBeenCalledOnce()
    expect(a()).toEqual({ kind: 'ok', value: 'shared' })
    expect(b()).toEqual({ kind: 'ok', value: 'shared' })
    // Broken: two fetches → toHaveBeenCalledOnce fails.
  })
})

describe('cache-conformance / invalidation', () => {
  it('invalidate(prefix) clears exactly the tagged namespace', async () => {
    const scope = createScope()
    const cache = withScope(scope, () => useNativeCache())
    let deals = 0
    let billing = 0
    withScope(scope, () =>
      nativeQueried(['crm', 'deals', 'list'], () => Promise.resolve(`d${++deals}`), {
        staleTime: 60_000,
      }),
    )
    withScope(scope, () =>
      nativeQueried(['crm', 'deals', 'detail', 1], () => Promise.resolve(`x${deals}`), {
        staleTime: 60_000,
      }),
    )
    withScope(scope, () =>
      nativeQueried(['billing', 'summary'], () => Promise.resolve(`b${++billing}`), {
        staleTime: 60_000,
      }),
    )
    await tick()
    await tick()
    expect(deals).toBe(1)
    expect(billing).toBe(1)

    cache.invalidate(['crm', 'deals'])
    await tick()
    await tick()
    // Both crm.deals.* refetched; billing untouched.
    expect(deals).toBe(2)
    expect(billing).toBe(1)
    // Broken: over-invalidation bumps billing; under-invalidation leaves deals=1.
  })
})

describe('cache-conformance / revalidate-never-flickers', () => {
  it('stale data stays rendered during background refetch — DOM text never flickers to loading', async () => {
    const root = createScope()
    let call = 0
    let resolveNext: ((v: string) => void) | null = null
    const fetcher = vi.fn(
      () =>
        new Promise<string>((resolve) => {
          call++
          if (call === 1) {
            resolve('stale-label')
          } else {
            resolveNext = resolve
          }
        }),
    )

    const first = createChildScope(root)
    const s1 = withScope(first, () =>
      nativeQueried(['flicker-dom'], fetcher, { staleTime: 0 }),
    )
    await tick()
    await tick()
    expect(s1()).toEqual({ kind: 'ok', value: 'stale-label' })
    first.dispose()

    // Remount: stale ok served immediately, background revalidate starts.
    const second = createChildScope(root)
    const s2 = withScope(second, () =>
      nativeQueried(['flicker-dom'], fetcher, { staleTime: 0 }),
    )
    expect(s2()).toEqual({ kind: 'ok', value: 'stale-label' })

    // Bind LiveState → DOM the way a template would (text is the rendered value).
    const el = document.createElement('span')
    document.body.appendChild(el)
    const stop = withScope(second, () =>
      effect(() => {
        const st = s2()
        // Template-shaped: while-ok paints value; loading would clear/replace.
        el.textContent = st.kind === 'ok' ? st.value : st.kind === 'loading' ? '' : 'ERR'
      }),
    )

    // Snapshot DOM before revalidate settles — must still show stale label.
    const snapshots: string[] = [el.textContent ?? '']
    // Force a microtask drain while still in-flight; capture after each tick
    // WITHOUT relying on wall-clock. We only resolve the fetch after asserting.
    await Promise.resolve()
    snapshots.push(el.textContent ?? '')
    await tick()
    snapshots.push(el.textContent ?? '')

    // Broken cache that flips ok→loading would produce '' in the snapshots.
    expect(snapshots.every((t) => t === 'stale-label')).toBe(true)
    expect(s2().kind).toBe('ok')

    // Complete revalidate — DOM advances to fresh value, still never empty.
    expect(resolveNext).not.toBeNull()
    resolveNext!('fresh-label')
    await tick()
    await tick()
    expect(el.textContent).toBe('fresh-label')
    expect(s2()).toEqual({ kind: 'ok', value: 'fresh-label' })

    stop()
    second.dispose()
    el.remove()
  })
})

describe('cache-conformance / SSR hydration', () => {
  it('server-fetched state does not refetch on client mount', async () => {
    const server = createScope()
    const serverCache = withScope(server, () => useNativeCache())
    withScope(server, () =>
      nativeQueried(['ssr-conf'], () => Promise.resolve('from-server')),
    )
    await tick()
    await tick()
    const dehydrated = dehydrateNative(serverCache)
    server.dispose()

    const client = createScope()
    const clientCache = withScope(client, () => useNativeCache())
    hydrateNative(clientCache, dehydrated)

    const fetcher = vi.fn(() => Promise.resolve('SHOULD-NOT-RUN'))
    const s = withScope(client, () =>
      nativeQueried(['ssr-conf'], fetcher, { staleTime: 60_000 }),
    )
    expect(s()).toEqual({ kind: 'ok', value: 'from-server' })
    expect(fetcher).not.toHaveBeenCalled()
    // Broken: missing hydrate would fetch; wrong key would miss the entry.
  })
})
