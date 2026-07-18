// query() PUSH path — the type-directed routing that folded `live()`'s
// PUSH half into `query()` (data-arch-seed.md, "Owner RULINGS 2026-07-18").
//
// A query thunk whose client method returns a `Promise<T>` routes to the
// PULL path (one-shot fill, staleness, retry). A thunk whose method
// returns a `LiveSource<T>` (a `subscribe` shape) routes to the PUSH path
// (the subscription keeps writing the SAME cache entry). The decision is
// made at the runtime type-resolved seam — `isLiveSource` on the value
// the fetcher returns — so no author marker and no second primitive.
//
// These tests drive `queryKeyed` (the direct/uncompiled entry point) and
// `nativeQuery` (the shared cache seam) since the routing is a runtime
// property of the fetcher, independent of the compiler's key derivation.

import { describe, expect, it, vi } from 'vitest'
import { signal } from '../src/signal'
import { createScope, withScope } from '../src/effect-scope'
import { queryKeyed } from '../src/query'
import { nativeQuery, useNativeCache } from '../src/native-cache'
import type { LiveSignal, LiveSource } from '../src/live-state'

const tick = () => new Promise((r) => setTimeout(r, 0))

describe('query() PULL vs PUSH routing', () => {
  it('a Promise client method → PULL: one fetch, settles ok', async () => {
    const scope = createScope()
    let calls = 0
    const s = withScope(scope, () =>
      queryKeyed(
        () => ['crm', 'deals', 'list'],
        () => {
          calls++
          return Promise.resolve({ items: ['d1', 'd2'] })
        },
      ),
    )
    expect(s().kind).toBe('loading')
    await tick()
    await tick()
    expect(s()).toEqual({ kind: 'ok', value: { items: ['d1', 'd2'] } })
    expect(calls).toBe(1)
    scope.dispose()
  })

  it('a LiveSource client method → PUSH: subscribes, ok on first push', () => {
    const scope = createScope()
    const source: LiveSource<number> = {
      subscribe: (sig: LiveSignal<number>) => {
        sig.set({ kind: 'ok', value: 11 })
        return () => {}
      },
    }
    const s = withScope(scope, () => queryKeyed(() => ['feed', 'clock'], () => source))
    // Push is synchronous here — settles ok immediately, no microtask.
    expect(s()).toEqual({ kind: 'ok', value: 11 })
    scope.dispose()
  })

  it('PUSH: subsequent pushes update the SAME cache entry (ok per push)', async () => {
    const scope = createScope()
    let emit: (v: number) => void = () => {}
    const source: LiveSource<number> = {
      subscribe: (sig: LiveSignal<number>) => {
        emit = (v) => sig.set({ kind: 'ok', value: v })
        return () => {}
      },
    }
    const cache = withScope(scope, () => useNativeCache())
    const s = withScope(scope, () => queryKeyed(() => ['feed', 'tick'], () => source))
    expect(s().kind).toBe('loading')
    emit(1)
    expect(s()).toEqual({ kind: 'ok', value: 1 })
    emit(2)
    emit(3)
    expect(s()).toEqual({ kind: 'ok', value: 3 })
    // One entry, one observer — every push wrote the same keyed signal.
    expect(cache.debugObserverCount(['feed', 'tick'])).toBe(1)
    scope.dispose()
  })

  it('PUSH: a source failure transitions the entry to error', () => {
    const scope = createScope()
    let fail: (e: Error) => void = () => {}
    const source: LiveSource<number> = {
      subscribe: (sig: LiveSignal<number>) => {
        fail = (e) => sig.set({ kind: 'error', error: e })
        return () => {}
      },
    }
    const s = withScope(scope, () => queryKeyed(() => ['feed', 'boom'], () => source))
    fail(new Error('transport down'))
    const v = s()
    expect(v.kind).toBe('error')
    if (v.kind === 'error') expect(v.error.message).toBe('transport down')
    scope.dispose()
  })

  it('PUSH: a subscribe that reads+sets sig does not re-enter', async () => {
    const scope = createScope()
    const factory = vi.fn()
    const src: LiveSource<number> = {
      subscribe: (sig: LiveSignal<number>) => {
        void sig() // reading must not subscribe the cache to its own writes
        sig.set({ kind: 'ok', value: 1 })
        queueMicrotask(() => {
          void sig()
          sig.set({ kind: 'ok', value: 2 })
        })
        return () => {}
      },
    }
    const s = withScope(scope, () =>
      queryKeyed(() => ['feed', 'accumulate'], () => {
        factory()
        return src
      }),
    )
    expect(s()).toEqual({ kind: 'ok', value: 1 })
    await tick()
    expect(s()).toEqual({ kind: 'ok', value: 2 })
    expect(factory).toHaveBeenCalledOnce()
    scope.dispose()
  })

  it('PUSH: root dispose runs the subscription cleanup (no leaked stream)', () => {
    const scope = createScope()
    const dispose = vi.fn()
    const s = withScope(scope, () =>
      queryKeyed<number>(
        () => ['feed', 'leak'],
        () => ({
          subscribe: (sig: LiveSignal<number>) => {
            sig.set({ kind: 'ok', value: 9 })
            return dispose
          },
        }),
      ),
    )
    expect(s()).toEqual({ kind: 'ok', value: 9 })
    expect(dispose).not.toHaveBeenCalled()
    scope.dispose()
    expect(dispose).toHaveBeenCalledOnce()
  })

  it('PUSH: a second observer on the same key reuses the one subscription', () => {
    const scope = createScope()
    const subscribeSpy = vi.fn((sig: LiveSignal<number>) => {
      sig.set({ kind: 'ok', value: 5 })
      return () => {}
    })
    const source: LiveSource<number> = { subscribe: subscribeSpy }
    const mk = () => nativeQuery(['feed', 'shared'], () => source)
    const a = withScope(scope, mk)
    const b = withScope(scope, mk)
    expect(a()).toEqual({ kind: 'ok', value: 5 })
    expect(b()).toEqual({ kind: 'ok', value: 5 })
    // Exactly one subscription opened, shared by both observers.
    expect(subscribeSpy).toHaveBeenCalledOnce()
    scope.dispose()
  })

  it('interleave: a pull then a push on the SAME key write one entry', async () => {
    const scope = createScope()
    const cache = withScope(scope, () => useNativeCache())
    // Pull fills the entry once.
    const pull = withScope(scope, () =>
      nativeQuery(['mix', 'k'], () => Promise.resolve('pulled')),
    )
    await tick()
    await tick()
    expect(pull()).toEqual({ kind: 'ok', value: 'pulled' })
    // A push source on the SAME key takes over the SAME entry — no adapter
    // layer between a "query" and a "subscription" vocabulary, so pull and
    // push interleave on one keyed signal. (staleTime 0 → the entry is not
    // fresh, so this observer drives a real subscribe that overwrites.)
    const push = withScope(scope, () =>
      nativeQuery(['mix', 'k'], () => ({
        subscribe: (sig: LiveSignal<string>) => {
          sig.set({ kind: 'ok', value: 'pushed' })
          return () => {}
        },
      })),
    )
    expect(push).toBe(pull) // one entry, one signal object
    expect(push()).toEqual({ kind: 'ok', value: 'pushed' })
    expect(cache.debugObserverCount(['mix', 'k'])).toBe(2)
    scope.dispose()
  })
})
