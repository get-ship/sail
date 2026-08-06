// SPEC-HARDENING-1 WI1 — Signal conformance suite (S7).
//
// Named acceptance cases for the reactive core. Each case fails when the
// corresponding scheduler invariant is broken. Teeth proof (glitch-bug):
// if mark-phase were replaced with eager recompute of each observer as it is
// marked (classic diamond glitch), `diamond glitch-free propagation` would
// observe a half-updated join (e.g. seen includes an intermediate value that
// is neither the pre-write nor the post-write settle) and/or
// `effect reading both arms never sees a half-updated graph` would see [1,0].

import { describe, expect, it, vi } from 'vitest'
import { signal, derived, effect, batch } from '../src/signal'
import { createScope, withScope, onCleanup } from '../src/effect-scope'

describe('signal-conformance / glitch-free diamond', () => {
  //        a
  //       / \
  //      b   c
  //       \ /
  //        d
  it('diamond glitch-free propagation — effect sees only pre and post settle', () => {
    const a = signal(1)
    const b = derived(() => a() + 1)
    const c = derived(() => a() * 10)
    const d = derived(() => b() + c())
    const seen: number[] = []
    effect(() => seen.push(d()))
    a.set(2)
    // Pre: 1+1 + 1*10 = 12. Post: 2+1 + 2*10 = 23.
    // A glitchy scheduler would push an intermediate (e.g. 13 = b' + c_old).
    expect(seen).toEqual([12, 23])
  })

  it('effect reading both arms of a diamond never sees a half-updated graph', () => {
    const a = signal(0)
    const left = derived(() => a())
    const right = derived(() => a())
    const pairs: Array<[number, number]> = []
    effect(() => pairs.push([left(), right()]))
    a.set(1)
    expect(pairs).toEqual([
      [0, 0],
      [1, 1],
    ])
  })
})

describe('signal-conformance / batch atomicity', () => {
  it('batch coalesces multi-source writes into one effect run with final values', () => {
    const x = signal(1)
    const y = signal(1)
    const sum = derived(() => x() + y())
    const seen: number[] = []
    effect(() => seen.push(sum()))
    batch(() => {
      x.set(10)
      y.set(20)
    })
    // Atomic: never observes 11 or 21 mid-batch.
    expect(seen).toEqual([2, 30])
  })
})

describe('signal-conformance / derived memoization', () => {
  it('derived body runs once per actual dep change, not per reader', () => {
    const a = signal(2)
    const compute = vi.fn(() => a() * 3)
    const d = derived(compute)
    expect(d()).toBe(6)
    expect(d()).toBe(6)
    expect(compute).toHaveBeenCalledTimes(1)
    a.set(5)
    expect(d()).toBe(15)
    expect(compute).toHaveBeenCalledTimes(2)
  })

  it('Object.is-equal recompute does not notify downstream', () => {
    const n = signal(0)
    const positive = derived(() => n() > 0)
    const fn = vi.fn()
    effect(() => {
      positive()
      fn()
    })
    expect(fn).toHaveBeenCalledTimes(1)
    n.set(5) // false → true
    expect(fn).toHaveBeenCalledTimes(2)
    n.set(10) // true → true
    expect(fn).toHaveBeenCalledTimes(2)
  })
})

describe('signal-conformance / effect ordering', () => {
  it('effects created first run before effects created later on the same write', () => {
    const s = signal(0)
    const order: string[] = []
    effect(() => {
      s()
      order.push('a')
    })
    effect(() => {
      s()
      order.push('b')
    })
    order.length = 0
    s.set(1)
    expect(order).toEqual(['a', 'b'])
  })

  it('cleanup runs before the next effect body, then on dispose', () => {
    const s = signal(0)
    const log: string[] = []
    const dispose = effect(() => {
      const v = s()
      log.push(`run:${v}`)
      return () => log.push(`clean:${v}`)
    })
    s.set(1)
    dispose()
    expect(log).toEqual(['run:0', 'clean:0', 'run:1', 'clean:1'])
  })
})

describe('signal-conformance / cleanup-on-dispose', () => {
  it('effect disposer detaches sources and runs final cleanup', () => {
    const s = signal(0)
    const cleanup = vi.fn()
    const body = vi.fn(() => {
      s()
      return cleanup
    })
    const dispose = effect(body)
    expect(body).toHaveBeenCalledTimes(1)
    dispose()
    expect(cleanup).toHaveBeenCalledTimes(1)
    s.set(1)
    expect(body).toHaveBeenCalledTimes(1)
  })

  it('onCleanup registered in a scope fires on scope.dispose', () => {
    const scope = createScope()
    const cleaned = vi.fn()
    withScope(scope, () => {
      onCleanup(cleaned)
    })
    expect(cleaned).not.toHaveBeenCalled()
    scope.dispose()
    expect(cleaned).toHaveBeenCalledTimes(1)
  })
})

describe('signal-conformance / no-leak remount', () => {
  it('repeated mount/unmount of nested effects does not leave orphan subscribers', () => {
    const outer = signal(0)
    const inner = signal(0)
    const innerRun = vi.fn()

    // Simulate mount: parent effect creates child effect.
    const disposeParent = effect(() => {
      outer()
      effect(() => {
        inner()
        innerRun()
      })
    })

    expect(innerRun).toHaveBeenCalledTimes(1)
    inner.set(1)
    expect(innerRun).toHaveBeenCalledTimes(2)

    // Re-run parent (old child disposed, new child created).
    outer.set(1)
    expect(innerRun).toHaveBeenCalledTimes(3)
    inner.set(2)
    expect(innerRun).toHaveBeenCalledTimes(4)

    // Unmount parent — no further inner fires.
    disposeParent()
    inner.set(3)
    outer.set(2)
    expect(innerRun).toHaveBeenCalledTimes(4)
  })

  it('scope remount: cleanups from prior mount do not accumulate', () => {
    const cleaned: number[] = []
    for (let i = 0; i < 5; i++) {
      const scope = createScope()
      withScope(scope, () => {
        onCleanup(() => cleaned.push(i))
      })
      scope.dispose()
    }
    expect(cleaned).toEqual([0, 1, 2, 3, 4])
    // A sixth dispose of a fresh empty scope must not re-fire old cleanups.
    createScope().dispose()
    expect(cleaned).toEqual([0, 1, 2, 3, 4])
  })
})
