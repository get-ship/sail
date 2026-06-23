// Whole-file confinement: a compiled view (glue + bundled controller) IS the
// sandbox unit. The glue is the DOM renderer (raw document.createElement &c.),
// so the endowment surface includes a NARROW node-factory `document` — not the
// full one. Node creation works; querySelector / cookie / fetch stay denied.

import { beforeAll, describe, expect, it } from 'vitest'

import { confineController, lowerModuleToExpression, lockdownRealm } from '../src/index.ts'

describe('lowerModuleToExpression', () => {
  it('lowers export default to a returnable trailing expression', () => {
    const out = lowerModuleToExpression(
      `import { signal } from '@ship/sui-runtime';\nexport default { setup: () => signal(0) };\n`,
    )
    expect(out).not.toContain('import')
    expect(out).not.toContain('export default')
    expect(out).toContain('const __sui_default__ =')
    expect(out.trimEnd().endsWith('__sui_default__;')).toBe(true)
  })

  it('strips export keyword from declarations and removes export lists', () => {
    const out = lowerModuleToExpression(
      `export const A = 1;\nexport function f() {}\nexport { A, f };\nexport default A;\n`,
    )
    expect(out).toContain('const A = 1;')
    expect(out).toContain('function f() {}')
    expect(out).not.toMatch(/export\s*\{/)
    expect(out).not.toContain('export const')
  })
})

describe('whole compiled view runs confined with a narrow document', () => {
  beforeAll(() => lockdownRealm())

  it('endows node factories + runtime; denies query/cookie/fetch', () => {
    // A view-shaped module: framework import, raw document node creation (what
    // the compiler emits), a default export — and ambient-authority probes.
    const viewModule = `
      import { signal } from "@ship/sui-runtime";
      const el = document.createElement('div');
      const count = signal(5);
      const probes = {
        createElement: typeof document.createElement,
        querySelector: typeof document.querySelector,
        cookieReachable: 'cookie' in document,
        fetch: typeof fetch,
      };
      export default { tag: el.tagName, count: count(), probes };
    `
    const source = lowerModuleToExpression(viewModule)

    // The capability-narrowed endowment surface: a node-factory-only document,
    // plus the runtime symbol the module uses. NOT full document, NOT fetch.
    const narrowDocument = {
      createElement: (tag: string) => ({ tagName: String(tag).toUpperCase() }),
      createTextNode: (t: string) => ({ nodeValue: t }),
      createComment: (t: string) => ({ nodeValue: t }),
      createDocumentFragment: () => ({ kind: 'fragment' }),
    }
    const result = confineController<{
      tag: string
      count: number
      probes: { createElement: string; querySelector: string; cookieReachable: boolean; fetch: string }
    }>(source, {
      signal: (v: number) => () => v,
      document: narrowDocument,
    })

    // Rendering (node creation) works through the narrow factory.
    expect(result.tag).toBe('DIV')
    expect(result.count).toBe(5)
    expect(result.probes.createElement).toBe('function')
    // The dangerous document capabilities are NOT reachable.
    expect(result.probes.querySelector).toBe('undefined')
    expect(result.probes.cookieReachable).toBe(false)
    // Ambient network is denied.
    expect(result.probes.fetch).toBe('undefined')
  })
})
