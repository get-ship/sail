// Parity gate: sail.config.json `network` / `unconfined` resolution must drive
// BOTH the author-time lint (Policy::*_for → check_confined) AND the runtime
// SES endowments through the same algorithm. This file pins the JS host half
// and the prepareRouteConfinement live path that wires resolved policy into
// buildRouteEndowments / confineRouteModule (or skips the Compartment).
//
// Compile-time counterparts live in:
//   crates/sail/compiler/tests/confinement_by_default.rs
//   crates/deck/deck-core/tests/network_allowlist_coherence.rs

import { mkdtempSync, mkdirSync, rmSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { afterEach, beforeAll, describe, expect, it } from 'vitest'

import {
  buildRouteEndowments,
  classifyAmbient,
  classifyAmbientJson,
  confineRouteModule,
  createConfinedFetch,
  lockdownRealm,
  prepareRouteConfinement,
  resolveAmbientAuthority,
} from '../src/index.ts'

const fakeDoc = {
  createElement: (t: string) => ({ tagName: t.toUpperCase() }),
  createElementNS: (_ns: string, t: string) => ({ tagName: t }),
  createTextNode: (t: string) => ({ nodeValue: t }),
  createComment: (t: string) => ({ nodeValue: t }),
  createDocumentFragment: () => ({ kind: 'fragment' }),
} as unknown as Document

const temps: string[] = []
function fixture(config: string | null, fileName = 'widget.controller.ts'): string {
  const root = mkdtempSync(join(tmpdir(), 'ambient-parity-'))
  temps.push(root)
  const srcDir = join(root, 'src')
  mkdirSync(srcDir, { recursive: true })
  if (config !== null) {
    writeFileSync(join(root, 'sail.config.json'), config)
  }
  const file = join(srcDir, fileName)
  writeFileSync(file, 'export default {}\n')
  return file
}

afterEach(() => {
  while (temps.length) {
    const d = temps.pop()
    if (d) rmSync(d, { recursive: true, force: true })
  }
})

describe('classifyAmbient — fail-closed mirror of Policy::classify ambient arms', () => {
  it('absent keys → empty network, unconfined false', () => {
    const a = classifyAmbient({ preset: 'strict', sx: true })
    expect(a.network).toEqual([])
    expect(a.unconfined).toBe(false)
  })

  it('declared network origins resolve verbatim', () => {
    const a = classifyAmbient({ network: ['https://api.example.com', 'self'] })
    expect(a.network).toEqual(['https://api.example.com', 'self'])
    expect(a.unconfined).toBe(false)
  })

  it('unconfined true is orthogonal to network', () => {
    const a = classifyAmbient({ unconfined: true, network: ['self'] })
    expect(a.unconfined).toBe(true)
    expect(a.network).toEqual(['self'])
  })

  it('wrong-typed network fails closed (Rejected)', () => {
    expect(classifyAmbient({ network: 'self' }).network).toEqual([])
    expect(classifyAmbient({ network: [1] }).network).toEqual([])
    expect(classifyAmbient({ network: ['ok', false] }).unconfined).toBe(false)
  })

  it('wrong-typed unconfined fails closed (Rejected)', () => {
    const a = classifyAmbient({ unconfined: 'yes', network: ['self'] })
    expect(a.unconfined).toBe(false)
    expect(a.network).toEqual([]) // whole config fails closed
  })

  it('unknown top-level key fails closed', () => {
    const a = classifyAmbient({ polcy: 'strict', network: ['self'] })
    expect(a.network).toEqual([])
    expect(a.unconfined).toBe(false)
  })

  it('unparseable JSON fails closed', () => {
    const a = classifyAmbientJson('{not json')
    expect(a.network).toEqual([])
    expect(a.unconfined).toBe(false)
  })
})

describe('resolveAmbientAuthority — walk-up mirrors Policy::network_for / unconfined_for', () => {
  it('no config on the walk → deny-all', () => {
    const file = fixture(null)
    const a = resolveAmbientAuthority(file)
    expect(a.network).toEqual([])
    expect(a.unconfined).toBe(false)
    expect(a.configPath).toBeNull()
  })

  it('network: ["self"] resolves from nearest package config', () => {
    const file = fixture(JSON.stringify({ network: ['self'] }))
    const a = resolveAmbientAuthority(file)
    expect(a.network).toEqual(['self'])
    expect(a.unconfined).toBe(false)
    expect(a.configPath).toMatch(/sail\.config\.json$/)
  })

  it('unconfined: true resolves from nearest package config', () => {
    const file = fixture(JSON.stringify({ unconfined: true }))
    const a = resolveAmbientAuthority(file)
    expect(a.unconfined).toBe(true)
    expect(a.network).toEqual([])
  })

  it('canvas-shaped network self + code-editor-shaped unconfined', () => {
    // Mirrors known in-repo packages (packages/components/sail/canvas|code-editor).
    const canvas = fixture(JSON.stringify({ sx: true, network: ['self'] }))
    expect(resolveAmbientAuthority(canvas).network).toEqual(['self'])
    const editor = fixture(JSON.stringify({ unconfined: true, sx: true }))
    expect(resolveAmbientAuthority(editor).unconfined).toBe(true)
  })
})

describe('runtime endowments honor resolved network / unconfined', () => {
  beforeAll(() => lockdownRealm())

  it('default deny: no config → confined fetch undefined (lint would Error on fetch)', () => {
    const file = fixture(null)
    const prepared = prepareRouteConfinement({
      source: `
        import { signal } from "@ship/sail-runtime";
        export default {
          setup: () => ({ fetchType: typeof fetch }),
        };
      `,
      ambient: resolveAmbientAuthority(file),
      endowments: { runtime: { signal: (v: number) => () => v }, ship: {}, doc: fakeDoc },
    })
    expect(prepared.kind).toBe('confined')
    if (prepared.kind !== 'confined') return
    expect(prepared.ambient.network).toEqual([])
    const out = prepared.module.default.setup({}) as { fetchType: string }
    expect(out.fetchType).toBe('undefined')
  })

  it('network: ["self"] → confined fetch allows relative, rejects evil absolute', async () => {
    const file = fixture(JSON.stringify({ network: ['self'] }))
    let reachedReal = false
    const realFetch = ((input: RequestInfo | URL) => {
      reachedReal = true
      return Promise.resolve(new Response(`ok:${String(input)}`))
    }) as typeof fetch
    // Stamp ambient then build endowments with a test double for real fetch
    // by going through createConfinedFetch via buildRouteEndowments after
    // resolve — and probe inside the compartment.
    const ambient = resolveAmbientAuthority(file)
    expect(ambient.network).toEqual(['self'])

    const surface = buildRouteEndowments({
      runtime: {},
      ship: {},
      doc: fakeDoc,
      network: ambient.network,
    }) as Record<string, unknown>
    expect(typeof surface.fetch).toBe('function')
    expect(surface.fetch).not.toBe(globalThis.fetch)

    // Swap in a controlled real fetch by re-wrapping — the production
    // wrapper closes over global fetch; for the allow/deny proof we call
    // createConfinedFetch directly with ambient.network (same list).
    const confined = createConfinedFetch(ambient.network, realFetch)
    await expect(confined('/api')).resolves.toBeInstanceOf(Response)
    expect(reachedReal).toBe(true)
    reachedReal = false
    await expect(confined('https://evil.test/x')).rejects.toThrow(/network" allowlist/)
    expect(reachedReal).toBe(false)

    // Inside a confined route module, relative fetch is endowed + callable.
    const prepared = prepareRouteConfinement({
      source: `
        export default {
          setup: () => ({
            fetchType: typeof fetch,
            // Dynamic target — lint cannot prove; runtime wrapper is the wall.
            dynamic: typeof fetch,
          }),
        };
      `,
      ambient,
      endowments: { runtime: {}, ship: {}, doc: fakeDoc },
    })
    expect(prepared.kind).toBe('confined')
    if (prepared.kind !== 'confined') return
    const out = prepared.module.default.setup({}) as { fetchType: string }
    expect(out.fetchType).toBe('function')
  })

  it('network grant does not imply bare global fetch — still a wrapper', () => {
    const surface = buildRouteEndowments({
      runtime: {},
      ship: {},
      doc: fakeDoc,
      network: ['https://api.example.com'],
    }) as Record<string, unknown>
    expect(surface.fetch).not.toBe(globalThis.fetch)
  })

  it('unconfined: true → prepareRouteConfinement skips the Compartment', () => {
    const file = fixture(JSON.stringify({ unconfined: true }))
    const source = `
      export default { setup: () => ({ fetchType: typeof fetch }) };
    `
    const prepared = prepareRouteConfinement({
      source,
      ambient: resolveAmbientAuthority(file),
      endowments: { runtime: {}, ship: {}, doc: fakeDoc },
    })
    expect(prepared.kind).toBe('unconfined')
    if (prepared.kind !== 'unconfined') return
    expect(prepared.ambient.unconfined).toBe(true)
    expect(prepared.source).toBe(source)
    // "Allows" means: host must load with real ambient — we do NOT run
    // confineRouteModule. A confined path with no network would have made
    // fetch undefined; unconfined opts out of that wall entirely.
  })

  it('unconfined false + empty network stays confined even if ambient is stamped', () => {
    const prepared = prepareRouteConfinement({
      source: `export default { setup: () => ({ f: typeof fetch }) };`,
      ambient: { network: [], unconfined: false, configPath: null },
      endowments: { runtime: {}, ship: {}, doc: fakeDoc },
    })
    expect(prepared.kind).toBe('confined')
    if (prepared.kind !== 'confined') return
    expect((prepared.module.default.setup({}) as { f: string }).f).toBe('undefined')
  })

  it('lint-shaped parity matrix: default / self / evil / unconfined', async () => {
    // Mirrors confinement_by_default.rs intent at runtime.
    type Case = {
      name: string
      config: string | null
      expectUnconfined: boolean
      expectFetchEndowed: boolean
    }
    const cases: Case[] = [
      { name: 'default deny', config: null, expectUnconfined: false, expectFetchEndowed: false },
      {
        name: 'network self',
        config: JSON.stringify({ network: ['self'] }),
        expectUnconfined: false,
        expectFetchEndowed: true,
      },
      {
        name: 'unconfined',
        config: JSON.stringify({ unconfined: true }),
        expectUnconfined: true,
        expectFetchEndowed: false, // not endowed via SES — skipped entirely
      },
    ]
    for (const c of cases) {
      const file = fixture(c.config)
      const prepared = prepareRouteConfinement({
        source: `export default { setup: () => ({ t: typeof fetch }) };`,
        ambient: resolveAmbientAuthority(file),
        endowments: { runtime: {}, ship: {}, doc: fakeDoc },
      })
      if (c.expectUnconfined) {
        expect(prepared.kind, c.name).toBe('unconfined')
      } else {
        expect(prepared.kind, c.name).toBe('confined')
        if (prepared.kind !== 'confined') continue
        const t = (prepared.module.default.setup({}) as { t: string }).t
        expect(t, c.name).toBe(c.expectFetchEndowed ? 'function' : 'undefined')
      }
    }
  })

  it('confineRouteModule + network still rejects evil origins at call time', async () => {
    const ambient = classifyAmbient({ network: ['self'] })
    const endowments = buildRouteEndowments({
      runtime: {},
      ship: {},
      doc: fakeDoc,
      network: ambient.network,
    })
    const mod = confineRouteModule(
      `export default { setup: () => ({ fetch }) };`,
      endowments,
    )
    const { fetch: f } = mod.default.setup({}) as { fetch: typeof fetch }
    await expect(f('https://evil.test/x')).rejects.toThrow(/network" allowlist/)
  })
})
