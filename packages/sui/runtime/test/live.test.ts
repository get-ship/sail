import { describe, expect, it, vi } from 'vitest'
import { signal } from '../src/signal'
import { createScope, withScope } from '../src/effect-scope'
import { live, type LiveSignal, type LiveSource } from '../src/live'

const tick = () => new Promise((r) => setTimeout(r, 0))

describe('live', () => {
  it('starts in loading', () => {
    const s = live(() => new Promise<number>(() => {}))
    expect(s().kind).toBe('loading')
  })

  it('transitions to ok on resolve', async () => {
    const s = live(() => Promise.resolve(42))
    expect(s().kind).toBe('loading')
    await tick()
    await tick()
    const v = s()
    expect(v.kind).toBe('ok')
    if (v.kind === 'ok') expect(v.value).toBe(42)
  })

  it('transitions to error on reject', async () => {
    const s = live(() => Promise.reject(new Error('boom')))
    await tick()
    await tick()
    const v = s()
    expect(v.kind).toBe('error')
    if (v.kind === 'error') expect(v.error.message).toBe('boom')
  })

  it('wraps non-Error rejections in Error', async () => {
    const s = live(() => Promise.reject('plain'))
    await tick()
    await tick()
    const v = s()
    expect(v.kind).toBe('error')
    if (v.kind === 'error') expect(v.error.message).toBe('plain')
  })

  it('catches synchronous throws from source', async () => {
    const s = live<number>(() => {
      throw new Error('sync')
    })
    await tick()
    const v = s()
    expect(v.kind).toBe('error')
    if (v.kind === 'error') expect(v.error.message).toBe('sync')
  })

  it('accepts sync return values', async () => {
    const s = live(() => 7 as number | Promise<number>)
    await tick()
    const v = s()
    expect(v.kind).toBe('ok')
    if (v.kind === 'ok') expect(v.value).toBe(7)
  })

  it('re-fires when a tracked dep changes — flips to loading then ok', async () => {
    const id = signal(1)
    const s = live(() => Promise.resolve(`v${id()}`))
    await tick()
    await tick()
    expect(s().kind).toBe('ok')
    id.set(2)
    expect(s().kind).toBe('loading')
    await tick()
    await tick()
    const v = s()
    expect(v.kind).toBe('ok')
    if (v.kind === 'ok') expect(v.value).toBe('v2')
  })

  it('invalidates old in-flight resolutions (token)', async () => {
    const id = signal(1)
    let resolveFirst: (v: string) => void = () => {}
    let resolveSecond: (v: string) => void = () => {}
    const s = live(() => {
      const cur = id()
      if (cur === 1) return new Promise<string>((r) => { resolveFirst = r })
      return new Promise<string>((r) => { resolveSecond = r })
    })
    expect(s().kind).toBe('loading')
    id.set(2)
    expect(s().kind).toBe('loading')
    // Stale resolution arrives — must be ignored.
    resolveFirst('STALE')
    await tick()
    expect(s().kind).toBe('loading')
    resolveSecond('FRESH')
    await tick()
    const v = s()
    expect(v.kind).toBe('ok')
    if (v.kind === 'ok') expect(v.value).toBe('FRESH')
  })

  it('LiveSource: subscribe owns lifecycle, transitions to ok', () => {
    const source: LiveSource<number> = {
      subscribe: (sig: LiveSignal<number>) => {
        sig.set({ kind: 'ok', value: 11 })
        return () => {}
      },
    }
    const s = live(() => source)
    const v = s()
    expect(v.kind).toBe('ok')
    if (v.kind === 'ok') expect(v.value).toBe(11)
  })

  it('LiveSource: dispose runs on tracked-dep re-fire', () => {
    // Deps are read in `source` (where the query is built), mirroring the
    // real `liveStream(() => ops.x.yLive(req))` adapter. A re-fire disposes
    // the prior subscription and opens a fresh one.
    const id = signal(1)
    const dispose = vi.fn()
    const s = live<number>(() => {
      const cur = id()
      return {
        subscribe: (sig: LiveSignal<number>) => {
          sig.set({ kind: 'ok', value: cur * 10 })
          return dispose
        },
      }
    })
    expect(s()).toEqual({ kind: 'ok', value: 10 })
    id.set(2)
    expect(dispose).toHaveBeenCalledOnce()
    expect(s()).toEqual({ kind: 'ok', value: 20 })
  })

  it('calls onError on rejection', async () => {
    const onError = vi.fn()
    live(() => Promise.reject(new Error('oops')), { onError })
    await tick()
    await tick()
    expect(onError).toHaveBeenCalledOnce()
    expect((onError.mock.calls[0]![0] as Error).message).toBe('oops')
  })

  // Regression — reactive re-entry. A streaming `LiveSource` whose
  // `subscribe` reads `sig()` (e.g. to accumulate) and then `sig.set`s —
  // both synchronously and again on a microtask — used to subscribe the
  // effect to `sig`, so the set re-fired the effect, which re-called
  // `source`, re-subscribed, set again … unbounded re-entry ending in
  // `RangeError: Maximum call stack size exceeded`. The signal must reach
  // `ok` while `source` runs exactly once.
  it('LiveSource: subscribe that reads+sets sig does not re-enter', async () => {
    const factory = vi.fn()
    const src: LiveSource<number> = {
      subscribe: (sig: LiveSignal<number>) => {
        // Reading sig() inside subscribe must not make the effect a sub of
        // sig. Set synchronously …
        void sig()
        sig.set({ kind: 'ok', value: 1 })
        // … and again deferred one microtask (the real first-frame path).
        queueMicrotask(() => {
          void sig()
          sig.set({ kind: 'ok', value: 2 })
        })
        return () => {}
      },
    }
    const s = live<number>(() => {
      factory()
      return src
    })

    expect(s()).toEqual({ kind: 'ok', value: 1 })
    await tick()
    expect(s()).toEqual({ kind: 'ok', value: 2 })
    // The one decisive assertion: source ran exactly once, no re-entry.
    expect(factory).toHaveBeenCalledOnce()
  })

  // Regression — disposal contract (the OOM-leak fix). Unmounting the
  // owning scope must run the effect's stop fn AND the subscription's
  // disposer so the underlying stream/EventSource closes.
  it('LiveSource: unmount runs the subscription cleanup', () => {
    const scope = createScope()
    const dispose = vi.fn()
    const s = withScope(scope, () =>
      live<number>(() => ({
        subscribe: (sig: LiveSignal<number>) => {
          sig.set({ kind: 'ok', value: 9 })
          return dispose
        },
      })),
    )
    expect(s()).toEqual({ kind: 'ok', value: 9 })
    expect(dispose).not.toHaveBeenCalled()
    scope.dispose()
    expect(dispose).toHaveBeenCalledOnce()
  })
})
