import { beforeAll, describe, expect, it } from 'vitest'

import { confineController, lowerFrameworkImports, lockdownRealm } from '../src/index.ts'

describe('lowerFrameworkImports', () => {
  it('strips named framework imports, keeps the bindings as free identifiers', () => {
    const out = lowerFrameworkImports(
      `import { controller, signal } from "@ship/sail-runtime";\nexport default controller({});\n`,
    )
    expect(out).not.toContain('import')
    expect(out).toContain('controller({})')
  })

  it('strips dom, sdk-client, subpath, and side-effect framework imports', () => {
    const out = lowerFrameworkImports(
      [
        `import { bindText } from '@ship/sail-dom';`,
        `import { ship } from '@ship/sdk-client';`,
        `import '@ship/sail-runtime/template-hooks';`,
        `import * as rt from "@ship/sail-runtime/patch";`,
        `const x = 1;`,
      ].join('\n'),
    )
    expect(out).not.toMatch(/@ship\/(sail-runtime|sail-dom|sdk-client)/)
    expect(out).toContain('const x = 1;')
  })

  it('leaves non-framework imports (packs, relative) intact', () => {
    const src = `import { dag } from '@ship/pipelines';\nimport { helper } from './util.js';\n`
    expect(lowerFrameworkImports(src)).toBe(src)
  })
})

describe('lowered controller runs confined', () => {
  beforeAll(() => lockdownRealm())

  it('a lowered controller resolves its runtime from endowments + stays confined', () => {
    // A realistic built controller: framework import at top, uses the lowered
    // `controller`/`signal` bindings, and probes ambient authority FROM INSIDE
    // the Compartment (typeof fetch in the controller body, not in an endowed
    // host function — endowments run in the host realm).
    const built = `
      import { controller, signal } from "@ship/sail-runtime";
      const out = controller(signal);
      ;({ ok: out.ok, fetch: typeof fetch });
    `
    const source = lowerFrameworkImports(built)
    expect(source).not.toContain('import')

    const audit: string[] = []
    const result = confineController<{ ok: boolean; fetch: string }>(source, {
      signal: (v: number) => v,
      // Proves the lowered `controller` binding resolved as an endowed global.
      controller: (s: (v: number) => number) => {
        audit.push('controller')
        return { ok: s(7) === 7 }
      },
    })
    expect(result.ok).toBe(true)
    expect(result.fetch).toBe('undefined')
    expect(audit).toEqual(['controller'])
  })
})
