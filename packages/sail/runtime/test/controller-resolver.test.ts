// Resolver behavior tests — exercises `resolveController` end-to-end
// without going through `hydrate()`. Each test drives the resolver
// directly with a synthetic `ResolverContext` and asserts on the
// returned bindings.
//
// `resolveController` no longer walks sections or builds a writable
// proxy: it builds the ambient route scope (flat providers + route/navigate
// + template state) and calls `def` with it. Whatever `def` returns —
// signals, `derived()`, handlers — IS the bindings record, verbatim.
// Handlers close over the scope's signals directly and mutate them via
// `.set()`, the single idiom `controller_state_set_call` enforces.

import { describe, expect, it, vi } from 'vitest'
import {
  controller,
  resolveController,
  signal,
  derived,
  type Signal,
} from '../src/index'

/** Build a `ResolverContext` with template-state signals seeded. */
function withState(
  state: Record<string, unknown>,
  extra: Record<string, unknown> = {},
): {
  scope: Record<string, unknown>
  stateKeys: Set<string>
  stateSignals: Record<string, Signal<unknown>>
} {
  const stateSignals: Record<string, Signal<unknown>> = {}
  for (const [k, v] of Object.entries(state)) {
    stateSignals[k] = signal(v) as Signal<unknown>
  }
  return {
    scope: { ...stateSignals, ...extra },
    stateKeys: new Set(Object.keys(state)),
    stateSignals,
  }
}

describe('resolveController — the ambient scope reaches the def', () => {
  it('exposes template state via scope (resolver does not own signal construction)', () => {
    const ctx = withState({ count: 0, label: 'hi' })
    resolveController((s: Record<string, unknown>) => {
      expect((s.count as () => unknown)()).toBe(0)
      expect((s.label as () => unknown)()).toBe('hi')
      return {}
    }, ctx)
    expect((ctx.stateSignals.count as () => unknown)()).toBe(0)
  })

  it('def receives the merged scope and its return is the bindings, verbatim', () => {
    const ctx = withState({ count: 5 })
    const seen = vi.fn()
    const b = resolveController(
      (s: Record<string, unknown>) => ({
        probe: () => seen((s.count as () => number)()),
      }),
      ctx,
    ) as Record<string, () => void>
    b.probe!()
    expect(seen).toHaveBeenCalledWith(5)
  })
})

describe('the single mutation idiom — handlers close over signals and call .set() directly', () => {
  it('handler mutations propagate to the underlying signal', () => {
    const ctx = withState({ count: 0 })
    const b = resolveController(
      (s: Record<string, unknown>) => ({
        incr: () => {
          const count = s.count as Signal<number>
          count.set(count() + 1)
        },
      }),
      ctx,
    ) as Record<string, () => void>
    expect((ctx.stateSignals.count as () => number)()).toBe(0)
    b.incr!()
    expect((ctx.stateSignals.count as () => number)()).toBe(1)
    b.incr!()
    b.incr!()
    expect((ctx.stateSignals.count as () => number)()).toBe(3)
  })

  it('handler accepts args beyond the closed-over scope', () => {
    const ctx = withState({ selectedId: null as string | null })
    const b = resolveController(
      (s: Record<string, unknown>) => ({
        select: (id: string) => (s.selectedId as Signal<string | null>).set(id),
      }),
      ctx,
    ) as Record<string, (id: string) => void>
    b.select!('a-42')
    expect((ctx.stateSignals.selectedId as () => string | null)()).toBe('a-42')
  })

  it('handler can read ship directly off scope', () => {
    const refresh = vi.fn()
    const ship = { agents: { refresh } }
    const b = resolveController(
      (s: Record<string, unknown>) => ({
        refresh: () => (s.ship as typeof ship).agents.refresh(),
      }),
      { ship },
    ) as Record<string, () => void>
    b.refresh!()
    expect(refresh).toHaveBeenCalledOnce()
  })

  it('derived() values returned alongside handlers stay memoized/reactive', () => {
    const ctx = withState({ count: 3 })
    const b = resolveController(
      (s: Record<string, unknown>) => ({
        doubled: derived(() => (s.count as () => number)() * 2),
      }),
      ctx,
    ) as Record<string, () => number>
    expect(b.doubled!()).toBe(6)
  })
})

describe('controller() — the one form', () => {
  it('returns a {setup, __def} that hydrate can call', () => {
    const def = (s: Record<string, unknown>) => ({
      incr: () => {
        const count = s.count as Signal<number>
        count.set(count() + 1)
      },
    })
    const out = controller(def)
    expect(typeof out.setup).toBe('function')
    expect(out.__def).toBe(def)

    const ctx = withState({ count: 0 })
    const b = out.setup(ctx)
    expect(typeof (b as Record<string, unknown>).incr).toBe('function')
    ;(b as Record<string, () => void>).incr!()
    expect((ctx.stateSignals.count as () => number)()).toBe(1)
  })

  it('setup closes over ship at instantiation time', () => {
    const ship = { hello: () => 'world' }
    const out = controller((s: Record<string, unknown>) => ({
      greeting: derived(() => (s.ship as typeof ship).hello()),
    }))
    const b = out.setup({ ship }) as Record<string, () => string>
    expect(b.greeting!()).toBe('world')
  })
})
