// Type-level + runtime tests for the controller contract.
//
// Per the "state is declarative-only" ADR, the controller has five
// sections — `derived`, `on`, `use`, `mount`, `unmount` — and no
// `state:` section. State signals come from the template's `<state>`
// directives and reach the controller via the merged scope. The tests
// below model that flow by seeding scope signals directly.

import { describe, expect, it } from 'vitest'
import {
  controller,
  signal,
  derived,
  type ControllerDef,
  type ControllerHook,
  type MutableState,
  type ReadonlyState,
  type Signal,
  type TemplateHook,
  type ViewScope,
} from '../src/index'

/** Build a `ResolverContext`-shaped harness with template state seeded. */
function withState(state: Record<string, unknown>): {
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
    scope: { ...stateSignals },
    stateKeys: new Set(Object.keys(state)),
    stateSignals,
  }
}

describe('controller() helper', () => {
  // P1.x wave 2: deleted two structural-shape tests (was lines 43-52
  // and 54-59 before this commit): 'wraps a def in {setup, __def} export'
  // and 'passes function-form defs through (still wrapped)'. They
  // asserted the wrapper's shape (typeof out.setup, out.__def identity)
  // and nothing the next 12 behavior tests in this file don't already
  // prove implicitly. The behavior tests (controller contract, mount +
  // unmount lifecycle, use-section composition, etc.) are kept.


  it('setup() runs the resolver and returns bindings', () => {
    const ctx = withState({ count: 5 })
    const out = controller({
      setup: (init) => ({ doubled: derived(() => ((init as { count: () => number }).count()) * 2) }),
      on: { incr: (s) => { s.count++ } },
    })
    const bindings = out.setup(ctx)
    expect(typeof (bindings as Record<string, () => unknown>).doubled).toBe('function')
    expect(typeof (bindings as Record<string, (...a: unknown[]) => unknown>).incr).toBe('function')
    expect((bindings as Record<string, () => number>).doubled!()).toBe(10)
    ;((bindings as Record<string, () => void>).incr)()
    expect((ctx.stateSignals.count as () => number)()).toBe(6)
  })
})

describe('tier-2 — object form compiles for every section', () => {
  it('on (no scope needed)', () => {
    type Scope = { count: Signal<number>; open: Signal<boolean> }
    controller<Scope>({
      on: {
        incr:   (s) => { s.count++ },
        toggle: (s) => { s.open = !s.open },
      },
    })
  })

  it('scope deriveds + on', () => {
    type Scope = { count: Signal<number> }
    controller({
      setup: (s: ViewScope<Scope>) => ({
        doubled: derived(() => s.count() * 2),
        over9k: derived(() => s.count() > 9000),
      }),
      on: {
        set:  (s, n: number) => { s.count = n },
        zero: (s)            => { s.count = 0 },
      },
    })
  })

  it('scope factory pulls template scope', () => {
    type Scope = {
      query: Signal<string>
      agents: Signal<{ items: string[] }>
    }
    controller({
      setup: (s: ViewScope<Scope>) => ({
        filtered: derived(() => s.agents().items.filter((n) => n.includes(s.query()))),
      }),
      on: {
        setQuery: (s, q: string) => { s.query = q },
      },
    })
  })

  it('mount + unmount', () => {
    type Scope = { tickerId: Signal<number> }
    controller<Scope>({
      mount: (s) => {
        s.tickerId = window.setInterval(() => {}, 1000)
      },
      unmount: (s) => {
        window.clearInterval(s.tickerId)
      },
    })
  })
})

describe('tier-3 — function form for closure-scoped variables', () => {
  it('closes over instantiation-time refs', () => {
    type Scope = { open: Signal<boolean> }
    controller<Scope>(({ ship: _ship }) => {
      const stableId = 'view-' + Math.random().toString(36).slice(2)
      return {
        on:    { toggle: (s) => { s.open = !s.open } },
        mount: (_s, el) => { el.dataset.viewId = stableId },
      }
    })
  })

  it('uses ship at section level', () => {
    type Scope = { selectedId: Signal<string | null> }
    controller<Scope>(({ ship: _ship }) => ({
      on: {
        select:  (s, id: string) => { s.selectedId = id },
      },
    }))
  })
})

describe('controller hooks — keep their state-bearing shape', () => {
  // A canonical controller hook: useFilteredList
  type MockSignal<T> = { (): T; set: (v: T) => void }

  const useFilteredList = <T>(
    source: MockSignal<T[]>,
    pred: (item: T, q: string) => boolean,
  ): ControllerHook<{ query: string }> => ({
    state: { query: '' },
    derived: {
      filtered: ({ query }) =>
        source().filter((i) => pred(i, (query as () => string)())),
    },
    on: {
      setQuery: (s, q: string) => { s.query = q },
    },
  })

  it('compiles as a ControllerHook with internal state', () => {
    const mockSig: MockSignal<string[]> = (() => ['a', 'b']) as MockSignal<string[]>
    mockSig.set = () => {}
    const hook = useFilteredList(mockSig, (s, q) => s.includes(q))
    expect(hook.state).toEqual({ query: '' })
    expect(hook.derived).toHaveProperty('filtered')
    expect(hook.on).toHaveProperty('setQuery')
  })

  it('composes via the use: section, namespaced by key', () => {
    const mockSig: MockSignal<string[]> = (() => ['a', 'b']) as MockSignal<string[]>
    mockSig.set = () => {}

    controller({
      use: {
        list: useFilteredList(mockSig, (s, q) => s.includes(q)),
        nested: useFilteredList(mockSig, (s, q) => s.startsWith(q)),
      },
    })
  })
})

describe('discipline — types enforce mutation rules', () => {
  it('handlers get a mutable state proxy', () => {
    type Scope = { count: Signal<number> }

    const ok: ControllerDef<Scope> = {
      on: {
        incr: (s) => { s.count++ }, // writable
      },
    }
    void ok
  })

  it('handler signature accepts both mutator and scope forms', () => {
    type Scope = { selectedId: Signal<string | null> }

    controller<Scope>({
      on: {
        select: (s, id: string) => { s.selectedId = id },
      },
    })

    controller<Scope>({
      on: {
        refresh: ({ ship: _ship }) => { /* call ship */ },
      },
    })
  })
})

describe('TemplateHook — authoring contract', () => {
  it('compiles for an attrs-only hook (e.g. <use-context as="ctx" />)', () => {
    const useContext: TemplateHook<{ as: string }, { theme: string }> = {
      name: 'context',
      attrs: { as: 'identifier' },
      factory: (ctx) => {
        ctx.cleanup(() => {})
        return { theme: 'dark' }
      },
    }
    expect(useContext.name).toBe('context')
  })

  it('compiles for a children-accepting hook (e.g. <use-title>{x}</use-title>)', () => {
    const useTitle: TemplateHook<object, void> = {
      name: 'title',
      children: true,
      factory: (ctx) => {
        ctx.cleanup(() => {})
      },
    }
    expect(useTitle.children).toBe(true)
  })

  it('compiles for a binding hook (e.g. <use-focus-trap bind:target={el} />)', () => {
    const useFocusTrap: TemplateHook<{ target: HTMLElement }, void> = {
      name: 'focus-trap',
      attrs: { target: 'binding' },
      factory: (ctx) => {
        const target = ctx.attrs.target
        target.focus()
        ctx.cleanup(() => {})
      },
    }
    expect(useFocusTrap.attrs?.target).toBe('binding')
  })
})

describe('ReadonlyState / MutableState shape distinction', () => {
  it('readonly view is callable, not assignable', () => {
    type Scope = { count: Signal<number> }
    type R = ReadonlyState<Scope>
    type M = MutableState<Scope>

    const _readonlyCheck: R = { count: () => 0 }
    const _mutableCheck:  M = { count: 0 }
    void _readonlyCheck
    void _mutableCheck
  })
})
