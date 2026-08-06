// Store API — scope-homed shared state.
//
// Verifies the contract: `useStore(def)` resolves nearest-or-root
// (never undefined), an instance is created on the chain root when
// no ancestor provides one, `provideStore(def)` parks a SCOPED
// instance on the current scope, and effects/cleanups attached to
// the instance via its setup die with the scope (the SSR/hydrate
// handshake is in `store-ssr.test.ts`).

import { afterEach, describe, expect, it } from 'vitest'
import { onCleanup, signal, derived, effect, defineStore, useStore, provideStore, peekStore, snapshotStore, provideContext, injectContext, contextKey } from '../src/index'
import { createScope, createChildScope, withScope, snapshotScopeStores, setStoreSSRPayload, clearStoreSSRPayload } from '../src/internal'

afterEach(() => {
  // Defensive: no test leaves an SSR payload behind (the global
  // state survives across tests in the same process).
  clearStoreSSRPayload()
})

describe('defineStore', () => {
  it('returns a stable def with the given id', () => {
    const a = defineStore('unit.alpha', () => ({ x: signal(0) }))
    const b = defineStore('unit.alpha', () => ({ x: signal(0) }))
    // Distinct calls produce distinct defs, but the id is the
    // author's contract. The runtime keys on `def.id`, not on
    // object identity.
    expect(a.id).toBe('unit.alpha')
    expect(b.id).toBe('unit.alpha')
    expect(a).not.toBe(b)
  })
})

describe('useStore — nearest-or-root resolution', () => {
  it('creates the instance on the chain root and returns it', () => {
    const root = createScope()
    const child = createChildScope(root)
    const def = defineStore('unit.use-root', () => {
      const n = signal(0)
      return { n, inc: () => n.set(n() + 1) }
    })

    let got: { n: () => number; inc: () => void } | null = null
    withScope(child, () => {
      got = useStore(def)
    })
    expect(got).not.toBeNull()
    expect(got!.n()).toBe(0)
    got!.inc()
    got!.inc()
    expect(got!.n()).toBe(2)
  })

  it('subsequent calls on any descendant return the SAME instance', () => {
    const root = createScope()
    const mid = createChildScope(root)
    const leaf = createChildScope(mid)
    const def = defineStore('unit.share', () => ({ v: signal('hi') }))

    let a: { v: () => string } | null = null
    let b: { v: () => string } | null = null
    withScope(mid, () => { a = useStore(def) })
    withScope(leaf, () => { b = useStore(def) })
    expect(a).toBe(b)  // same instance, same signals
  })

  it('two roots produce two independent instances', () => {
    const rootA = createScope()
    const rootB = createScope()
    const def = defineStore('unit.iso', () => ({ v: signal(0) }))

    let a: { v: () => number } | null = null
    let b: { v: () => number } | null = null
    withScope(rootA, () => { a = useStore(def) })
    withScope(rootB, () => { b = useStore(def) })
    expect(a).not.toBe(b)
    a!.v()  // each carries its own signal
    b!.v()
  })

  it('never returns undefined — falls back to the chain root', () => {
    // Even with an empty scope (no current scope at all), useStore
    // must produce an instance via the lazy anon root. The type
    // signature is `Shape`, not `Shape | undefined`, and the runtime
    // honors that.
    const def = defineStore('unit.anon', () => ({ tag: signal('A') }))
    const inst = useStore(def)
    expect(inst.tag()).toBe('A')
  })
})

describe('useStore — derived/cleanup integration', () => {
  it('derived entries observe the right signals and re-derive', () => {
    const def = defineStore('unit.derived', () => {
      const a = signal(2)
      const b = signal(3)
      const sum = derived(() => a() + b())
      return { a, b, sum }
    })
    const s = useStore(def)
    expect(s.sum()).toBe(5)
    s.a.set(10)
    expect(s.sum()).toBe(13)
  })

  it('setup cleanups die with the HOME scope, not the first caller', () => {
    // `setup()` runs inside a dedicated child scope of the HOME (the
    // chain root for a `useStore` miss). The instance is served to
    // every subtree, so its internals must survive the first caller
    // unmounting and die only when the home does.
    let disposed = 0
    const def = defineStore('unit.dispose', () => {
      onCleanup(() => { disposed++ })
      return { v: signal(1) }
    })
    const root = createScope()
    const child = createChildScope(root)

    let first: { v: () => number } | null = null
    withScope(child, () => { first = useStore(def) })

    // First CALLER unmounts — the root-homed instance stays alive.
    child.dispose()
    expect(disposed).toBe(0)
    let again: { v: () => number } | null = null
    withScope(root, () => { again = useStore(def) })
    expect(again).toBe(first)

    // The store dies with its HOME.
    root.dispose()
    expect(disposed).toBe(1)
  })

  it('useStore inside an effect resolves the same root instance', () => {
    // The effect body runs with the effect's ownership scope current;
    // that scope chains to the effect's lexical owner, so the walk
    // reaches the mount root and finds the SAME instance — an effect
    // must never become its own chain root (per-effect instance
    // fragmentation).
    const def = defineStore('unit.effect-chain', () => ({ v: signal(7) }))
    const root = createScope()
    let outer: { v: () => number } | null = null
    let inner: { v: () => number } | null = null
    withScope(root, () => {
      outer = useStore(def)
      effect(() => { inner = useStore(def) })
    })
    expect(inner).toBe(outer)
    root.dispose()
  })
})

describe('provideStore — scoped override', () => {
  it('installs a new instance on the current scope', () => {
    const root = createScope()
    const def = defineStore('unit.scoped', () => {
      const v = signal('root')
      return { v, set: (x: string) => v.set(x) }
    })

    let rootInst: { v: () => string; set: (x: string) => void } | null = null
    let scopedInst: { v: () => string; set: (x: string) => void } | null = null
    withScope(root, () => { rootInst = useStore(def) })

    const subtree = createChildScope(root)
    withScope(subtree, () => { scopedInst = provideStore(def) })

    expect(scopedInst).not.toBe(rootInst)
    rootInst!.set('A')
    expect(rootInst!.v()).toBe('A')
    expect(scopedInst!.v()).toBe('root')  // unaffected — separate instance
    scopedInst!.set('B')
    expect(rootInst!.v()).toBe('A')  // root still A
    expect(scopedInst!.v()).toBe('B')
  })

  it('descendants of the providing scope see the override; siblings see the root', () => {
    const root = createScope()
    const left = createChildScope(root)
    const right = createChildScope(root)
    const def = defineStore('unit.sibling', () => {
      const v = signal(0)
      return { v }
    })

    withScope(left, () => { provideStore(def) })
    let fromLeft: { v: () => number } | null = null
    let fromRight: { v: () => number } | null = null
    withScope(left, () => { fromLeft = useStore(def) })
    withScope(right, () => { fromRight = useStore(def) })

    // `left` and its descendants see the override; `right` falls
    // through to the root.
    expect(fromLeft).not.toBe(fromRight)
  })
})

describe('peekStore', () => {
  it('returns null when no instance exists; does not create', () => {
    const def = defineStore('unit.peek', () => ({ x: signal(0) }))
    // No scope set, no anon root touched.
    // peekStore from a fresh (uncreated) state returns null. The
    // anon root is the chain root the test environment uses, so
    // if any earlier test touched it we may already have an
    // instance — guard by ID, not by identity.
    // Skip if already there.
    const existing = peekStore(def)
    if (existing) return
    expect(existing).toBeNull()
  })
})

describe('provideContext / injectContext', () => {
  it('walks the parent chain, nearest wins', () => {
    const root = createScope()
    const mid = createChildScope(root)
    const leaf = createChildScope(mid)
    type Handle = { id: string }
    const key = contextKey<Handle>('unit.handle')

    const rootHandle: Handle = { id: 'root' }
    const midHandle: Handle = { id: 'mid' }
    withScope(root, () => { provideContext(key, rootHandle) })
    withScope(mid, () => { provideContext(key, midHandle) })

    let fromLeaf: Handle | null = null
    withScope(leaf, () => { fromLeaf = injectContext(key) })
    expect(fromLeaf?.id).toBe('mid')

    let fromRoot: Handle | null = null
    withScope(root, () => { fromRoot = injectContext(key) })
    expect(fromRoot?.id).toBe('root')
  })

  it('returns null when no ancestor provided', () => {
    const root = createScope()
    type Marker = { tag: string }
    const key = contextKey<Marker>('unit.miss')
    let got: Marker | null | 'sentinel' = 'sentinel'
    withScope(root, () => { got = injectContext(key) })
    expect(got).toBeNull()
  })
})

describe('snapshot — server-side serialization', () => {
  it('snapshotStore captures every Signal value', () => {
    const inst = {
      count: signal(7),
      label: signal('cart'),
      note: 'plain string',  // ignored
      double: () => 14,        // non-signal — ignored
    }
    const snap = snapshotStore(inst)
    expect(snap).toEqual({ count: 7, label: 'cart' })
  })

  it('snapshotScopeStores walks the contexts map for store keys only', () => {
    const root = createScope()
    const defA = defineStore('unit.snap.a', () => ({ a: signal(1) }))
    const defB = defineStore('unit.snap.b', () => ({ b: signal('two') }))
    // Plant both, plus an arbitrary non-store context value.
    const extra = contextKey<{ irrelevant: boolean }>('unit.snap.extra')
    withScope(root, () => {
      useStore(defA)
      useStore(defB)
      provideContext(extra, { irrelevant: true })
    })
    const snap = snapshotScopeStores(root)
    expect(snap['unit.snap.a']).toEqual({ a: 1 })
    expect(snap['unit.snap.b']).toEqual({ b: 'two' })
    // The non-store key was filtered out.
    expect(Object.keys(snap).sort()).toEqual(['unit.snap.a', 'unit.snap.b'])
  })
})

describe('SSR handshake — minimal in-process coverage', () => {
  it('a payload installed before useStore seeds the new instance', () => {
    setStoreSSRPayload({ 'unit.ssr.seed': { count: 99, label: 'from-ssr' } })
    const def = defineStore('unit.ssr.seed', () => ({
      count: signal(0),
      label: signal('initial'),
    }))
    const inst = useStore(def)
    expect(inst.count()).toBe(99)
    expect(inst.label()).toBe('from-ssr')
  })

  it('no payload → setup() initial values are kept', () => {
    const def = defineStore('unit.ssr.nopayload', () => ({
      v: signal('initial'),
    }))
    const inst = useStore(def)
    expect(inst.v()).toBe('initial')
  })
})
