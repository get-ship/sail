// scope: block + useStore/defineStore flow.
//
// The scope: factory contributes signals/derived/hooks to the merged
// scope. State that needs to be shared across components now goes
// through `defineStore(setup)` + `useStore(def)` (scope-homed, no DOM
// walk) — the old `shared(signal)` + `useShared(host, tag)` auto-publish
// on the element was deleted with context.ts. A controller's scope
// factory reaches a store via `useStore(def)`, identical to a regular
// component; the store instance lives on the chain root of the mount.

import { describe, expect, it } from 'vitest'
import { signal, derived, defineStore, useStore } from '../src/index'
import { resolveController } from '../src/controller-resolver'
import type { ResolverContext } from '../src/controller-resolver-types'
import type { ShipSDK } from '../src/ship'

const fakeShip = {} as ShipSDK

function ctx(host?: Element): ResolverContext {
  return { ship: fakeShip, host }
}

describe('scope: block', () => {
  it('factory output merges into bindings', () => {
    const def = {
      scope: () => ({
        count: signal(7),
        doubled: derived(() => 14),
      }),
    }
    const bindings = resolveController(def, ctx())
    expect(typeof bindings.count).toBe('function')
    expect((bindings.count as () => number)()).toBe(7)
    expect((bindings.doubled as () => number)()).toBe(14)
  })

  it('on: handlers can mutate scope: signals via writable proxy', () => {
    const def = {
      scope: () => ({ count: signal(0) }),
      on: {
        inc: (s: { count: number }) => { s.count = s.count + 1 },
      },
    }
    const bindings = resolveController(def, ctx())
    const count = bindings.count as () => number
    const inc = bindings.inc as () => void
    expect(count()).toBe(0)
    inc()
    inc()
    expect(count()).toBe(2)
  })

  it('scope: factory can pull a scoped store via useStore', () => {
    // A store is defined once, at module top-level. A controller's
    // scope: factory calls useStore(def) the same way a regular
    // component does — the runtime walks the scope chain, finds
    // the existing instance (or creates one on the chain root), and
    // returns it. The returned shape is destructured into the
    // scope like any other scope: contribution.
    const ui = defineStore('test.ui', () => {
      const mode = signal<'find' | 'command'>('find')
      const setMode = (m: 'find' | 'command'): void => mode.set(m)
      return { mode, setMode }
    })

    const def = {
      scope: () => {
        const s = useStore(ui)
        return { ui: s.mode, setUiMode: s.setMode }
      },
    }
    const bindings = resolveController(def, ctx())
    const mode = bindings.ui as () => 'find' | 'command'
    const setMode = bindings.setUiMode as (m: 'find' | 'command') => void
    expect(mode()).toBe('find')
    setMode('command')
    expect(mode()).toBe('command')
  })

  it('two resolves of the same store share the instance', () => {
    // The scope: factory runs once per `resolveController` call. Two
    // controllers (or two scope: blocks across renders) that both
    // call useStore(def) for the same store id get the same signals
    // — a write in one is visible in the other.
    const counter = defineStore('test.counter', () => {
      const n = signal(0)
      return { n, inc: () => n.set(n() + 1) }
    })

    const defA = { scope: () => useStore(counter) }
    const defB = { scope: () => useStore(counter) }

    const a = resolveController(defA, ctx()) as { n: () => number; inc: () => void }
    const b = resolveController(defB, ctx()) as { n: () => number; inc: () => void }

    a.inc()
    a.inc()
    expect(a.n()).toBe(2)
    expect(b.n()).toBe(2)  // shared — same instance
  })

  it('mount: can mutate scope: signals when template declares no <state>', () => {
    // Regression: ship-ui-studio's mount() does `state.sui = catalog.sui`
    // against signals created by the scope: factory. The template carries
    // no <state> declarations, so ctx.stateKeys is an empty Set. Before
    // the fix, an empty (but defined) stateKeys short-circuited the
    // signal-picker and scope: signals were treated as read-only — the
    // assignment dropped silently with a console warning.
    const def = {
      scope: () => ({ items: signal<string[]>([]), loading: signal(true) }),
      mount: (s: { items: string[]; loading: boolean }) => {
        s.items = ['a', 'b']
        s.loading = false
      },
    }
    const stateKeys: ReadonlySet<string> = new Set()
    const bindings = resolveController(def, { ship: fakeShip, stateKeys })
    const lifecycle = (
      bindings as Record<string, unknown>
    )['__shipSetLifecycle'] as { state: Record<string, unknown>; mount: (s: Record<string, unknown>, el: HTMLElement) => void }
    lifecycle.mount(lifecycle.state, document.createElement('div'))
    expect((bindings.items as () => string[])()).toEqual(['a', 'b'])
    expect((bindings.loading as () => boolean)()).toBe(false)
  })

  it('skipped when no scope: block declared', () => {
    const def = { on: { fn: () => {} } }
    const bindings = resolveController(def, ctx())
    expect(typeof bindings.fn).toBe('function')
  })
})
