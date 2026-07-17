import { beforeAll, describe, expect, it } from 'vitest'

import {
  buildViewEndowments,
  confineViewModule,
  lockdownRealm,
} from '../src/index.ts'

// A node env has no `document`; supply node factories only (what the real
// narrowDocument exposes from a browser `document`).
const fakeDoc = {
  createElement: (t: string) => ({ tagName: t.toUpperCase() }),
  createElementNS: (_ns: string, t: string) => ({ tagName: t }),
  createTextNode: (t: string) => ({ nodeValue: t }),
  createComment: (t: string) => ({ nodeValue: t }),
  createDocumentFragment: () => ({ kind: 'fragment' }),
} as unknown as Document

describe('buildViewEndowments', () => {
  beforeAll(() => lockdownRealm())

  it('assembles runtime + narrow document + ops, hardened', () => {
    const signal = (v: number) => () => v
    const ship = { git: { overview: () => ({}) } }
    const surface = buildViewEndowments({
      runtime: { signal, controller: () => ({}) },
      dom: { bindText: () => {} },
      ship,
      doc: fakeDoc,
    }) as Record<string, unknown>

    expect(surface.signal).toBe(signal)
    expect(typeof surface.controller).toBe('function')
    expect(typeof surface.bindText).toBe('function')
    expect(surface.ship).toBe(ship)
    // document is narrowed: node factories present, query/cookie absent.
    const doc = surface.document as Record<string, unknown>
    expect(typeof doc.createElement).toBe('function')
    expect(doc.querySelector).toBeUndefined()
    expect('cookie' in doc).toBe(false)
    // Hardened — the view can't graft new capabilities onto the surface.
    expect(Object.isFrozen(surface)).toBe(true)
  })

  it('does not endow fetch by default (no "network" grant) — unchanged from before the capability existed', () => {
    const surface = buildViewEndowments({
      runtime: {},
      ship: {},
      doc: fakeDoc,
    }) as Record<string, unknown>
    expect(surface.fetch).toBeUndefined()
  })

  it('endows a confined fetch WRAPPER (never the bare global) when "network" is granted', async () => {
    const surface = buildViewEndowments({
      runtime: {},
      ship: {},
      doc: fakeDoc,
      network: ['https://api.example.com'],
    }) as Record<string, unknown>
    expect(typeof surface.fetch).toBe('function')
    expect(surface.fetch).not.toBe(globalThis.fetch)
    // Same allowlist semantics as the confine-level wrapper: granted origin
    // is reachable (network error, not a capability refusal, since this test
    // has no real network); a refused origin throws before ever attempting
    // the request.
    const confinedFetch = surface.fetch as typeof fetch
    await expect(confinedFetch('https://evil.com/x')).rejects.toThrow(
      /network" allowlist/,
    )
  })

  it('an empty "network" array behaves like no grant at all — fetch stays un-endowed', () => {
    const surface = buildViewEndowments({
      runtime: {},
      ship: {},
      doc: fakeDoc,
      network: [],
    }) as Record<string, unknown>
    expect(surface.fetch).toBeUndefined()
  })
})

describe('confineViewModule', () => {
  beforeAll(() => lockdownRealm())

  it('confines a whole view into the mount opts.module shape; setup runs confined', () => {
    // A view-shaped module: framework import, raw document node creation in
    // setup (what the compiler emits), default export, ambient probe.
    const source = `
      import { signal } from "@ship/sail-runtime";
      export default {
        setup: (ctx) => {
          const el = document.createElement('section');
          const n = signal(3);
          return { tag: el.tagName, n: n(), fetch: typeof fetch, query: typeof document.querySelector };
        },
      };
    `
    const endowments = buildViewEndowments({
      runtime: { signal: (v: number) => () => v },
      ship: {},
      doc: fakeDoc,
    })

    const mod = confineViewModule(source, endowments)
    expect(typeof mod.default.setup).toBe('function')

    const out = mod.default.setup({}) as Record<string, unknown>
    // Endowed rendering + runtime work inside the confined view.
    expect(out.tag).toBe('SECTION')
    expect(out.n).toBe(3)
    // Ambient authority + dangerous document caps stay denied even when the
    // Compartment-defined setup is invoked from the host realm.
    expect(out.fetch).toBe('undefined')
    expect(out.query).toBe('undefined')
  })
})
