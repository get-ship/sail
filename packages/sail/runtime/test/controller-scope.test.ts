// setup() + useStore/defineStore flow.
//
// The setup function contributes signals/derived/hooks to the merged
// scope AND returns handlers in the same object — one function, no
// sections. State shared across components goes through
// `defineStore(setup)` + `useStore(def)` (scope-homed, no DOM walk). A
// controller's setup function reaches a store via `useStore(def)`,
// identical to a regular component; the store instance lives on the
// chain root of the mount.

import { describe, expect, it } from 'vitest'
import { signal, derived, defineStore, useStore } from '../src/index'
import { resolveController } from '../src/controller-resolver'
import type { ResolverContext } from '../src/controller-resolver-types'

function ctx(host?: Element): ResolverContext {
  return { host }
}

describe('setup() — the whole controller surface', () => {
  it('return value merges into bindings', () => {
    const def = () => ({
      count: signal(7),
      doubled: derived(() => 14),
    })
    const bindings = resolveController(def, ctx())
    expect(typeof bindings.count).toBe('function')
    expect((bindings.count as () => number)()).toBe(7)
    expect((bindings.doubled as () => number)()).toBe(14)
  })

  it('a returned handler mutates a signal closed over by setup, directly', () => {
    const def = () => {
      const count = signal(0)
      return { count, inc: () => count.set(count() + 1) }
    }
    const bindings = resolveController(def, ctx())
    const count = bindings.count as () => number
    const inc = bindings.inc as () => void
    expect(count()).toBe(0)
    inc()
    inc()
    expect(count()).toBe(2)
  })

  it('setup can pull a scoped store via useStore', () => {
    // A store is defined once, at module top-level. A controller's
    // setup function calls useStore(def) the same way a regular
    // component does — the runtime walks the scope chain, finds
    // the existing instance (or creates one on the chain root), and
    // returns it. The returned shape is destructured into the
    // returned scope like anything else the controller declares.
    const ui = defineStore('test.ui', () => {
      const mode = signal<'find' | 'command'>('find')
      const setMode = (m: 'find' | 'command'): void => mode.set(m)
      return { mode, setMode }
    })

    const def = () => {
      const s = useStore(ui)
      return { ui: s.mode, setUiMode: s.setMode }
    }
    const bindings = resolveController(def, ctx())
    const mode = bindings.ui as () => 'find' | 'command'
    const setMode = bindings.setUiMode as (m: 'find' | 'command') => void
    expect(mode()).toBe('find')
    setMode('command')
    expect(mode()).toBe('command')
  })

  it('two resolves of the same store share the instance', () => {
    // setup runs once per `resolveController` call. Two controllers
    // that both call useStore(def) for the same store id get the same
    // signals — a write in one is visible in the other.
    const counter = defineStore('test.counter', () => {
      const n = signal(0)
      return { n, inc: () => n.set(n() + 1) }
    })

    const defA = () => useStore(counter)
    const defB = () => useStore(counter)

    const a = resolveController(defA, ctx()) as { n: () => number; inc: () => void }
    const b = resolveController(defB, ctx()) as { n: () => number; inc: () => void }

    a.inc()
    a.inc()
    expect(a.n()).toBe(2)
    expect(b.n()).toBe(2) // shared — same instance
  })

  it('setup-created signals are mutated directly, no lifecycle/proxy plumbing needed', () => {
    // Regression precursor: ship-ui-studio's old `mount()` section did
    // `state.sail = catalog.sail` against signals the setup: factory
    // created, working around an empty-stateKeys writable-proxy bug.
    // There is no proxy anymore — a controller with no <state>
    // declarations mutates its own setup-created signals the same way
    // any other signal write works, with no lifecycle indirection.
    const def = () => {
      const items = signal<string[]>([])
      const loading = signal(true)
      const load = () => {
        items.set(['a', 'b'])
        loading.set(false)
      }
      return { items, loading, load }
    }
    const bindings = resolveController(def, { stateKeys: new Set() })
    ;(bindings.load as () => void)()
    expect((bindings.items as () => string[])()).toEqual(['a', 'b'])
    expect((bindings.loading as () => boolean)()).toBe(false)
  })

  it('a controller returning only handlers (no state) still resolves', () => {
    const def = () => ({ fn: () => {} })
    const bindings = resolveController(def, ctx())
    expect(typeof bindings.fn).toBe('function')
  })

  it('flat providers reach the controller as bare names', () => {
    const api = { list: () => ['a'] }
    const def = (s: Record<string, unknown>) => {
      expect(s.api).toBe(api)
      return {}
    }
    resolveController(def, { api } as ResolverContext)
  })
})
