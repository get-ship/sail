// query.ts — the public query()/mutation() primitives.
//
// query()/queryKeyed() layer: mostly a thin wrapper over nativeQueried
// (already torture-tested in native-cache-torture.test.ts) — these
// tests focus on what's NEW here: the reactive re-keying (a changed
// key attaches to a DIFFERENT entry rather than refetching the old
// one, forwarded through one stable outer signal) and the
// no-compiler-key refusal.
//
// mutation() layer: the idle/loading/ok/error lifecycle and
// auto-invalidation wiring against the shared NativeCache.invalidate.

import { describe, expect, it, vi } from 'vitest'
import { createScope, createChildScope, withScope } from '../src/effect-scope'
import { signal } from '../src/signal'
import { queryKeyed, query, mutation } from '../src/query'
import { useNativeCache } from '../src/native-cache'

const tick = () => new Promise((r) => setTimeout(r, 0))

describe('queryKeyed', () => {
  it('a key change attaches to a NEW entry (fresh fetch) rather than refetching the old one, forwarded through one stable outer signal', async () => {
    const scope = createScope()
    const stage = signal('open')
    const calls: string[] = []
    const outer = withScope(scope, () =>
      queryKeyed(
        () => ['crm', 'deals', 'list', stage()],
        () => {
          calls.push(stage())
          return Promise.resolve(`deals-${stage()}`)
        },
      ),
    )
    const ref = outer
    await tick()
    await tick()
    expect(outer()).toEqual({ kind: 'ok', value: 'deals-open' })

    stage.set('closed')
    // Immediately after the key change (before the new fetch's promise
    // settles) the NEW entry starts loading — not a silent refetch of
    // the old value.
    expect(outer()).toEqual({ kind: 'loading' })
    expect(outer).toBe(ref) // same outer signal object throughout

    await tick()
    await tick()
    expect(outer()).toEqual({ kind: 'ok', value: 'deals-closed' })
    expect(calls).toEqual(['open', 'closed'])
  })

  it('reverting to a previously-seen key reattaches to that entry (cache hit, no new fetch if still fresh)', async () => {
    const scope = createScope()
    const stage = signal('a')
    let fetchCount = 0
    withScope(scope, () =>
      queryKeyed(
        () => ['revert', stage()],
        () => {
          fetchCount++
          return Promise.resolve(stage())
        },
        { staleTime: 60_000 },
      ),
    )
    await tick()
    await tick()
    expect(fetchCount).toBe(1)

    stage.set('b')
    await tick()
    await tick()
    expect(fetchCount).toBe(2)

    stage.set('a') // revert — entry for 'a' still fresh (staleTime 60s), no refetch
    await tick()
    await tick()
    expect(fetchCount).toBe(2)
  })

  it('disposing the calling scope releases the current attachment (observer refcount drops, entry survives for GC)', async () => {
    const root = createScope()
    const cache = withScope(root, () => useNativeCache())
    const child = createChildScope(root)
    const key = ['dispose-releases']
    withScope(child, () => queryKeyed(() => key, () => Promise.resolve('v')))
    await tick()
    await tick()
    expect(cache.debugObserverCount(key)).toBe(1)

    child.dispose()
    expect(cache.debugObserverCount(key)).toBe(0) // released, entry still resident pending GC
    expect(cache.debugHasEntry(key)).toBe(true)
  })
})

describe('query()', () => {
  it('refuses loudly when called without the compiler-supplied key-fn', () => {
    const scope = createScope()
    expect(() =>
      withScope(scope, () => query(() => Promise.resolve('v'))),
    ).toThrow(/compiler-derived cache key/)
  })

  it('with a key-fn supplied (simulating compiled output) behaves exactly like queryKeyed', async () => {
    const scope = createScope()
    const s = withScope(scope, () =>
      query(() => Promise.resolve('compiled'), undefined, () => ['query-compiled']),
    )
    await tick()
    await tick()
    expect(s()).toEqual({ kind: 'ok', value: 'compiled' })
  })
})

describe('mutation()', () => {
  it('starts idle, walks idle -> loading -> ok on success, mutate() resolves with the value', async () => {
    const scope = createScope()
    const m = withScope(scope, () => mutation(() => Promise.resolve({ id: 1 })))
    expect(m.state()).toEqual({ kind: 'idle' })

    const promise = m.mutate()
    expect(m.state()).toEqual({ kind: 'loading' })

    const value = await promise
    expect(value).toEqual({ id: 1 })
    expect(m.state()).toEqual({ kind: 'ok', value: { id: 1 } })
  })

  it('walks idle -> loading -> error on rejection, mutate() rejects', async () => {
    const scope = createScope()
    const m = withScope(scope, () => mutation(() => Promise.reject(new Error('nope'))))

    await expect(m.mutate()).rejects.toThrow('nope')
    expect(m.state()).toEqual({ kind: 'error', error: expect.any(Error) })
  })

  it('re-running mutate() reads fresh signal state each call (not memoized from definition time)', async () => {
    const scope = createScope()
    const draft = signal('v1')
    const m = withScope(scope, () => mutation(() => Promise.resolve(draft())))

    expect(await m.mutate()).toBe('v1')
    draft.set('v2')
    expect(await m.mutate()).toBe('v2')
  })

  it('auto-invalidates the compiler-derived namespace on success', async () => {
    const scope = createScope()
    const cache = withScope(scope, () => useNativeCache())
    let listCalls = 0
    withScope(scope, () =>
      queryKeyed(() => ['crm', 'deals', 'list'], () => Promise.resolve(`list-${++listCalls}`), {
        staleTime: 60_000,
      }),
    )
    await tick()
    await tick()
    expect(listCalls).toBe(1)

    const m = withScope(scope, () =>
      mutation(() => Promise.resolve('updated'), undefined, ['crm', 'deals']),
    )
    await m.mutate()
    await tick()
    await tick()
    expect(listCalls).toBe(2) // auto-invalidated the crm.deals.* namespace
  })

  it('invalidates extra compiler-resolved references from opts.invalidates on success, in addition to the auto namespace', async () => {
    const scope = createScope()
    let billingCalls = 0
    withScope(scope, () =>
      queryKeyed(() => ['billing', 'summary'], () => Promise.resolve(`bill-${++billingCalls}`), {
        staleTime: 60_000,
      }),
    )
    await tick()
    await tick()
    expect(billingCalls).toBe(1)

    const m = withScope(scope, () =>
      mutation(() => Promise.resolve('updated'), undefined, ['crm', 'deals'], [['billing', 'summary']]),
    )
    await m.mutate()
    await tick()
    await tick()
    expect(billingCalls).toBe(2)
  })

  it('a failed mutation does NOT invalidate anything', async () => {
    const scope = createScope()
    const cache = withScope(scope, () => useNativeCache())
    const invalidateSpy = vi.spyOn(cache, 'invalidate')
    const m = withScope(scope, () =>
      mutation(() => Promise.reject(new Error('down')), undefined, ['crm', 'deals']),
    )
    await m.mutate().catch(() => {})
    expect(invalidateSpy).not.toHaveBeenCalled()
  })
})
