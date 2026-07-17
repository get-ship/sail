// SSR handshake — the per-request store snapshot the server emits
// into the HTML, the client ingests, and `useStore` consumes to
// pre-seed signal values before any effect runs.

import { afterEach, describe, expect, it } from 'vitest'
import {
  createScope,
  createChildScope,
  withScope,
  signal,
  defineStore,
  useStore,
  setStoreSSRPayload,
  clearStoreSSRPayload,
  readStoreSSRSeed,
  snapshotStore,
  snapshotScopeStores,
  hydrateStoreSSRPayload,
  SSR_PAYLOAD_GLOBAL,
} from '../src/index'

afterEach(() => {
  clearStoreSSRPayload()
  delete (globalThis as Record<string, unknown>)[SSR_PAYLOAD_GLOBAL]
})

describe('SSR — server snapshot', () => {
  it('snapshotStore reads every signal in a shape', () => {
    const inst = { count: signal(0), label: signal('cart') }
    inst.count.set(5)
    const snap = snapshotStore(inst)
    expect(snap).toEqual({ count: 5, label: 'cart' })
  })

  it('snapshotScopeStores ignores non-store context entries', () => {
    const root = createScope()
    const def = defineStore('ssr.snap.iso', () => ({ v: signal(0) }))
    withScope(root, () => {
      useStore(def)
      const sym = Symbol('not.a.store') as unknown as { __ctx?: unknown } & symbol
      ;(root.contexts as Map<symbol, unknown>).set(sym, { x: 1 })
    })
    const snap = snapshotScopeStores(root)
    expect(snap['ssr.snap.iso']).toEqual({ v: 0 })
    expect(Object.keys(snap)).toEqual(['ssr.snap.iso'])
  })
})

describe('SSR — client seed (no flash)', () => {
  it('seeds signal values from the payload before the first effect reads', () => {
    setStoreSSRPayload({ 'ssr.seed.basic': { count: 42, label: 'ssr' } })
    const def = defineStore('ssr.seed.basic', () => ({
      count: signal(0),
      label: signal('initial'),
    }))
    const inst = useStore(def)
    expect(inst.count()).toBe(42)
    expect(inst.label()).toBe('ssr')
  })

  it('reads a partial payload (only some signals seeded)', () => {
    setStoreSSRPayload({ 'ssr.seed.partial': { count: 7 } })
    const def = defineStore('ssr.seed.partial', () => ({
      count: signal(0),
      untouched: signal('initial'),
    }))
    const inst = useStore(def)
    expect(inst.count()).toBe(7)
    expect(inst.untouched()).toBe('initial')
  })

  it('a payload entry with undefined values does NOT overwrite', () => {
    setStoreSSRPayload({ 'ssr.seed.undef': { count: undefined as unknown as number } })
    const def = defineStore('ssr.seed.undef', () => ({ count: signal(99) }))
    const inst = useStore(def)
    expect(inst.count()).toBe(99)
  })

  it('payload survives the chain walk — descendant useStore sees it', () => {
    setStoreSSRPayload({ 'ssr.seed.walk': { v: 'A' } })
    const root = createScope()
    const leaf = createChildScope(root)
    const def = defineStore('ssr.seed.walk', () => ({ v: signal('initial') }))
    let inst: { v: () => string } | null = null
    withScope(leaf, () => { inst = useStore(def) })
    expect(inst!.v()).toBe('A')
  })

  it('an existing instance is not re-seeded (idempotent over re-renders)', () => {
    // Both `a` and `b` are looked up on the SAME chain — that
    // is the re-render / HMR case (the view re-runs, calls
    // useStore again). The second call returns the cached
    // instance, NOT a fresh one seeded from the latest payload.
    const def = defineStore('ssr.seed.hmr', () => ({ n: signal(0) }))
    const root = createScope()
    let a: { n: () => number } = { n: () => 0 }
    withScope(root, () => {
      setStoreSSRPayload({ 'ssr.seed.hmr': { n: 1 } })
      a = useStore(def)
      expect(a.n()).toBe(1)
      a.n.set(99)
      setStoreSSRPayload({ 'ssr.seed.hmr': { n: 2 } })
      const b = useStore(def)
      expect(b).toBe(a)
      expect(b.n()).toBe(99)
    })
  })
})

describe('SSR — per-request isolation', () => {

  it('two SSR requests on the same process do not share state', () => {
    // Each SSR request gets its own root scope. The store cache
    // is per-scope, so request #1's instance lives on request
    // #1's root and is unreachable from request #2.
    const cart = defineStore('ssr.iso.cart', () => ({ total: signal(0) }))

    // ── request #1 ─────────────────────────────────────────────
    const rootA = createScope()
    setStoreSSRPayload({ 'ssr.iso.cart': { total: 0 } })
    let req1Total = 0
    withScope(rootA, () => {
      const r1 = useStore(cart)
      r1.total.set(150)
      req1Total = r1.total()
    })
    const req1Payload = snapshotScopeStores(rootA)
    clearStoreSSRPayload()
    rootA.dispose()

    // ── request #2 ─────────────────────────────────────────────
    // Fresh root, fresh payload (empty — the new request's
    // setup() initial is 0, not 150). Even if #1's root were
    // somehow still around, it's not on #2's chain.
    const rootB = createScope()
    setStoreSSRPayload({})
    let req2Total: number | null = null
    withScope(rootB, () => {
      const r2 = useStore(cart)
      req2Total = r2.total()
    })
    expect(req2Total).toBe(0)
    expect(req1Total).toBe(150)
    expect(req1Payload).toEqual({ 'ssr.iso.cart': { total: 150 } })

    // ── client for #1 hydrates from #1's payload ──────────────
    setStoreSSRPayload(req1Payload)
    const client = useStore(cart)
    expect(client.total()).toBe(150)
  })

  it('clearStoreSSRPayload drops the payload', () => {
    setStoreSSRPayload({ 'ssr.clear': { v: 1 } })
    expect(readStoreSSRSeed('ssr.clear')).toEqual({ v: 1 })
    clearStoreSSRPayload()
    expect(readStoreSSRSeed('ssr.clear')).toBeNull()
  })
})

describe('SSR — hydrate from window global', () => {
  it('reads window[SSR_PAYLOAD_GLOBAL] into the module state', () => {
    ;(globalThis as Record<string, unknown>)[SSR_PAYLOAD_GLOBAL] = {
      'ssr.hydrate.fromWin': { n: 11 },
    }
    hydrateStoreSSRPayload()
    const def = defineStore('ssr.hydrate.fromWin', () => ({ n: signal(0) }))
    const inst = useStore(def)
    expect(inst.n()).toBe(11)
  })

  it('is a no-op when the global is absent', () => {
    hydrateStoreSSRPayload()
    expect(readStoreSSRSeed('ssr.hydrate.absent')).toBeNull()
  })

  it('survives a non-object global (e.g. a number, null)', () => {
    ;(globalThis as Record<string, unknown>)[SSR_PAYLOAD_GLOBAL] = 42
    hydrateStoreSSRPayload()
    expect(readStoreSSRSeed('ssr.hydrate.num')).toBeNull()
  })
})
