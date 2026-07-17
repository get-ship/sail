// Type-level + runtime tests for the controller(fn) contract.
//
// One idiom: `controller((ctx) => { ...; return {...} })`. State is a
// plain `signal()` closed over by the setup function; handlers
// returned from it mutate that signal directly. Lifecycle is ambient
// (`mount(fn)` / `onCleanup(fn)`, tested in `effect-scope.test.ts` and
// `mount-leak.test.ts`), not a section.

import { describe, expect, it } from 'vitest'
import {
  controller,
  signal,
  derived,
  type ControllerDef,
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
  it('setup() runs the def and returns its bindings verbatim', () => {
    const ctx = withState({ count: 5 })
    const out = controller((init: ViewScope<{ count: Signal<number> }>) => {
      const doubled = derived(() => init.count() * 2)
      const incr = () => init.count.set(init.count() + 1)
      return { doubled, incr }
    })
    const bindings = out.setup(ctx)
    expect(typeof (bindings as Record<string, () => unknown>).doubled).toBe('function')
    expect(typeof (bindings as Record<string, () => unknown>).incr).toBe('function')
    expect((bindings as Record<string, () => number>).doubled!()).toBe(10)
    ;(bindings as Record<string, () => void>).incr!()
    expect((ctx.stateSignals.count as () => number)()).toBe(6)
  })

  it('__def carries the original setup function', () => {
    const fn = (_init: ViewScope<object>) => ({})
    const out = controller(fn)
    expect(out.__def).toBe(fn)
  })
})

describe('the one form — single setup function returns the whole surface', () => {
  it('signals, derived, and handlers are all plain returned properties', () => {
    type Scope = { count: Signal<number>; open: Signal<boolean> }
    controller((s: ViewScope<Scope>) => ({
      incr: () => s.count.set(s.count() + 1),
      toggle: () => s.open.set(!s.open()),
    }))
  })

  it('setup can derive from template scope and return handlers that read/write it', () => {
    type Scope = { count: Signal<number> }
    controller((s: ViewScope<Scope>) => ({
      doubled: derived(() => s.count() * 2),
      over9k: derived(() => s.count() > 9000),
      set: (n: number) => s.count.set(n),
      zero: () => s.count.set(0),
    }))
  })

  it('setup factory pulls template scope directly (no separate scope: section)', () => {
    type Scope = {
      query: Signal<string>
      agents: Signal<{ items: string[] }>
    }
    controller((s: ViewScope<Scope>) => ({
      filtered: derived(() => s.agents().items.filter((n) => n.includes(s.query()))),
      setQuery: (q: string) => s.query.set(q),
    }))
  })

  it('closes over instantiation-time locals (the old tier-3 use case, now the only case)', () => {
    type Scope = { open: Signal<boolean> }
    controller((s: ViewScope<Scope>, host?: Element) => {
      const stableId = 'view-' + Math.random().toString(36).slice(2)
      if (host) (host as HTMLElement).dataset.viewId = stableId
      return { toggle: () => s.open.set(!s.open()) }
    })
  })

  it('setup can reach ship directly — no separate section-level access', () => {
    type Scope = { selectedId: Signal<string | null> }
    controller((s: ViewScope<Scope>) => ({
      select: (id: string) => {
        void s.ship
        s.selectedId.set(id)
      },
    }))
  })
})

describe('discipline — types enforce the single mutation idiom', () => {
  it('a handler returned from setup mutates a signal directly (no writable-proxy indirection)', () => {
    type Scope = { count: Signal<number> }

    const def: ControllerDef<Scope> = (s) => ({
      incr: () => (s as unknown as Scope).count.set((s as unknown as Scope).count() + 1),
    })
    void def
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
