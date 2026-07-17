// Resolver behavior tests — exercises `resolveController` end-to-end
// without going through `hydrate()`. Each test drives the resolver
// directly with a synthetic `ResolverContext` and asserts on the
// returned bindings.
//
// Per the "state is declarative-only" ADR, controllers no longer have
// a `state:` section. State signals enter the resolver via
// `ctx.scope` (the merged template scope) and are identified as
// writable by `ctx.stateKeys`. The harness below seeds them directly.

import { describe, expect, it, vi } from 'vitest'
import {
  controller,
  resolveController,
  signal,
  derived,
  type ResolvedBindings,
  type Signal,
} from '../src/index'

const noShip = { ship: undefined }

/** Build a `ResolverContext` with template-state signals seeded. */
function withState(
  state: Record<string, unknown>,
  extra: Record<string, unknown> = {},
): {
  ship: undefined
  scope: Record<string, unknown>
  stateKeys: Set<string>
  stateSignals: Record<string, Signal<unknown>>
} {
  const stateSignals: Record<string, Signal<unknown>> = {}
  for (const [k, v] of Object.entries(state)) {
    stateSignals[k] = signal(v) as Signal<unknown>
  }
  return {
    ship: undefined,
    scope: { ...stateSignals, ...extra },
    stateKeys: new Set(Object.keys(state)),
    stateSignals,
  }
}

describe('state — signals seeded by the template flow into scope', () => {
  it('exposes template state via scope (resolver does not own signal construction)', () => {
    const ctx = withState({ count: 0, label: 'hi' })
    resolveController({}, ctx)
    expect((ctx.stateSignals.count as () => unknown)()).toBe(0)
    expect((ctx.stateSignals.label as () => unknown)()).toBe('hi')
  })

  it('handlers can read seeded state via the writable proxy', () => {
    const ctx = withState({ count: 5 })
    const seen = vi.fn()
    const b = resolveController(
      { on: { probe: (s: { count: number }) => seen(s.count) } },
      ctx,
    ) as Record<string, () => void>
    b.probe!()
    expect(seen).toHaveBeenCalledWith(5)
  })
})

describe('on — handlers receive a writable proxy', () => {
  it('handler mutations propagate to the underlying signal', () => {
    const ctx = withState({ count: 0 })
    const b = resolveController(
      { on: { incr: (s: { count: number }) => { s.count++ } } },
      ctx,
    ) as Record<string, ((...a: unknown[]) => void)>
    expect((ctx.stateSignals.count as () => number)()).toBe(0)
    b.incr!()
    expect((ctx.stateSignals.count as () => number)()).toBe(1)
    b.incr!()
    b.incr!()
    expect((ctx.stateSignals.count as () => number)()).toBe(3)
  })

  it('handler accepts extra args alongside the proxy', () => {
    const ctx = withState({ selectedId: null as string | null })
    const b = resolveController(
      {
        on: {
          select: (s: { selectedId: string | null }, id: string) => {
            s.selectedId = id
          },
        },
      },
      ctx,
    ) as Record<string, (id: string) => void>
    b.select!('a-42')
    expect((ctx.stateSignals.selectedId as () => string | null)()).toBe('a-42')
  })

  it('handler can read scope (ship) via the proxy fallthrough', () => {
    const refresh = vi.fn()
    const ship = { agents: { refresh } }
    const b = resolveController(
      {
        on: {
          refresh: ({ ship: s }: { ship: typeof ship }) => s.agents.refresh(),
        },
      },
      { ship },
    ) as Record<string, () => void>
    b.refresh!()
    expect(refresh).toHaveBeenCalledOnce()
  })

  it('$signals exposes the raw signal map for explicit .set()', () => {
    // Pre-fix the only way to mutate state from `mount` was via the
    // assignment proxy (`state.viewportH = el.clientHeight`).
    // `state.viewportH.set(...)` looked correct but silently threw
    // because the value-side read returned a number. $signals gives
    // mount handlers an explicit signal-access path.
    const ctx = withState({ count: 0, label: 'hi' })
    type ScopeS = { $signals: { count: Signal<number>; label: Signal<string> } }
    const b = resolveController(
      {
        on: {
          bumpViaSignal: (s: ScopeS) => { s.$signals.count.set(7) },
          setLabel: (s: ScopeS) => { s.$signals.label.set('there') },
        },
      },
      ctx,
    ) as Record<string, () => void>
    b.bumpViaSignal!()
    b.setLabel!()
    expect((ctx.stateSignals.count as () => number)()).toBe(7)
    expect((ctx.stateSignals.label as () => string)()).toBe('there')
  })

  it('writes to $signals warn and are ignored (read-only accessor)', () => {
    const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {})
    const ctx = withState({ count: 0 })
    const b = resolveController(
      {
        on: {
          stomp: (s: Record<string, unknown>) => { s.$signals = {} },
        },
      },
      ctx,
    ) as Record<string, () => void>
    b.stomp!()
    expect(warnSpy).toHaveBeenCalledWith(expect.stringContaining('$signals'))
    warnSpy.mockRestore()
  })

  it('writes to a non-state key warn and are ignored', () => {
    const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {})
    const ctx = withState({ count: 0 })
    const b = resolveController(
      {
        on: {
          // typo — `total` is not a declared writable signal
          bogus: (s: Record<string, unknown>) => {
            s.total = 99
          },
        },
      },
      ctx,
    ) as Record<string, () => void>
    b.bogus!()
    expect(warnSpy).toHaveBeenCalledWith(
      expect.stringContaining(`total is not a writable signal`),
    )
    warnSpy.mockRestore()
  })
})


describe('mount / unmount lifecycle — attached as non-enumerable LIFECYCLE_KEY', () => {
  const lifecycleOf = (b: ResolvedBindings) =>
    (b as Record<string, unknown>)[
      '__shipSetLifecycle'
    ] as
      | { state: unknown; mount?: (s: unknown, el: HTMLElement) => void; unmount?: (s: unknown) => void }
      | undefined

  it('attaches lifecycle when mount or unmount is declared', () => {
    const ctx = withState({ count: 0 })
    const b = resolveController(
      {
        mount: (s) => {
          ;(s as { count: number }).count = 5
        },
        unmount: () => {},
      },
      ctx,
    )
    const lc = lifecycleOf(b)
    expect(lc).toBeDefined()
    expect(typeof lc!.mount).toBe('function')
    expect(typeof lc!.unmount).toBe('function')
  })

  it('omits lifecycle when neither hook is declared', () => {
    const ctx = withState({ count: 0 })
    const b = resolveController({}, ctx)
    expect(lifecycleOf(b)).toBeUndefined()
  })

  it('lifecycle.state is the writable proxy — mount can mutate template state', () => {
    let captured: unknown
    const ctx = withState({ count: 0 })
    const b = resolveController(
      {
        mount: (s) => {
          ;(s as { count: number }).count = 7
          captured = s
        },
      },
      ctx,
    )
    const lc = lifecycleOf(b)!
    lc.mount!(lc.state, document.createElement('div'))
    expect(captured).toBe(lc.state)
    expect((ctx.stateSignals.count as () => number)()).toBe(7)
  })

  it('LIFECYCLE_KEY is enumerable so the compiled view carries it through Object.assign', () => {
    // The compiled view does `Object.assign($scope, __r)` to fold the
    // resolver result into the template scope. A non-enumerable key was
    // silently dropped by that assign, so async mounts (e.g. ship-ui-studio)
    // never wired up. Keeping it enumerable is the contract.
    const ctx = withState({ x: 1 })
    const b = resolveController({ mount: () => {} }, ctx)
    expect(Object.keys(b)).toContain('__shipSetLifecycle')
    const spread = { ...b } as Record<string, unknown>
    expect(spread.__shipSetLifecycle).toBeDefined()
  })
})

describe('controller() — drop-in compatibility with view() shape', () => {
  it('returns a {setup, __def} that hydrate can call', () => {
    const def = {
      on: { incr: (s: { count: number }) => { s.count++ } },
    }
    const out = controller(def)
    expect(typeof out.setup).toBe('function')
    expect(out.__def).toBe(def)

    const ctx = withState({ count: 0 })
    const b = out.setup(ctx)
    expect(typeof (b as Record<string, unknown>).incr).toBe('function')
    ;((b as Record<string, () => void>).incr)()
    expect((ctx.stateSignals.count as () => number)()).toBe(1)
  })

  it('function-form def works through the wrap', () => {
    const ship = { hello: () => 'world' }
    const def = ({ ship: s }: { ship: typeof ship }) => ({
      setup: () => ({ greeting: derived(() => s.hello()) }),
    })
    const out = controller(def)
    const b = out.setup({ ship }) as Record<string, () => string>
    expect(b.greeting!()).toBe('world')
  })
})
