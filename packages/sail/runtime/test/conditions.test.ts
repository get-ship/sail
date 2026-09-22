// Behavioral conditions for the reactive runtime (glitch-free diamond, disposal, host-free mount).

import { describe, expect, it } from 'vitest'
import { signal, derived, effect, batch } from '../src/signal'
import { mount, hostContext } from '../src/internal'
import type { RoutePayload } from '../src/internal'

// JSDOM has no EventSource; stub so mount() construction never fails on it.
if (typeof (globalThis as { EventSource?: unknown }).EventSource === 'undefined') {
  ;(globalThis as { EventSource?: unknown }).EventSource = class {
    readyState = 1
    addEventListener(): void {}
    removeEventListener(): void {}
    close(): void {}
  }
}

describe('runtime — glitch-free, exactly-once propagation', () => {
  //        a
  //       / \
  //      b   c        b = a+1 , c = a*10
  //       \ /
  //        d          d = b + c
  function diamond() {
    const a = signal(1)
    const b = derived(() => a() + 1)
    const c = derived(() => a() * 10)
    const d = derived(() => b() + c())
    const seen: number[] = []
    effect(() => seen.push(d()))
    return { a, seen } // initial: a=1 → b=2, c=10, d=12
  }

  it('a bare diamond write updates the dependent exactly once with no stale read', () => {
    const { a, seen } = diamond()
    a.set(2)
    expect(seen).toEqual([12, 23])
  })

  it('a batched diamond write is glitch-free and runs the effect once', () => {
    const { a, seen } = diamond()
    batch(() => a.set(2))
    expect(seen).toEqual([12, 23])
  })

  it('an equal-value write notifies nobody', () => {
    const s = signal(7)
    const seen: number[] = []
    effect(() => seen.push(s()))
    s.set(7)
    expect(seen).toEqual([7]) // initial run only
  })
})

describe('runtime — disposal stops reactivity and runs cleanups', () => {
  it('disposing an effect halts re-runs and fires its registered cleanup', () => {
    const s = signal(0)
    const runs: number[] = []
    const cleanups: number[] = []

    const dispose = effect(() => {
      const v = s()
      runs.push(v)
      return () => cleanups.push(v)
    })

    s.set(1)
    dispose()
    s.set(2)

    expect(runs).toEqual([0, 1])
    expect(cleanups).toEqual([0, 1])
  })
})

describe('runtime — host-agnostic mount', () => {
  it('reads the host seam safely when no host is registered', () => {
    expect(() => hostContext()).not.toThrow()
  })

  it('mounts and renders a route with no ship/route/navigate supplied', async () => {
    const target = document.createElement('div')
    document.body.appendChild(target)
    const payload: RoutePayload = {
      name: 'host-free',
      html: '<p>hello</p>',
      css: '',
      scope: {},
      moduleUrl: '/x',
    }
    const handle = await mount(target, payload, {
      module: { default: { setup: () => ({}) } },
    })
    expect(target.querySelector('p')?.textContent).toBe('hello')
    handle.dispose()
    target.remove()
  })
})
