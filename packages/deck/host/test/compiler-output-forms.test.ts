// Regression: the lowering must handle the forms the SUI COMPILER actually
// emits — not the hand-written `export default {…}` fixtures the earlier tests
// used. A real compiled page module (e.g. views/git) emits:
//   import { bindText, component, … } from "@ship/sail-runtime";
//   import { collectElementMarkers, … } from "@ship/sail-dom";
//   import"@ship/sail-runtime/template-hooks";   ← no space, side-effect
//   …
//   export { page_default as default };          ← named-rename, multi-line
// Both tripped the original lowering (which required `import\s+` and only knew
// `export default EXPR`).

import { describe, expect, it } from 'vitest'
import { lowerFrameworkImports, lowerModuleToExpression } from '../src/lower-imports.ts'

describe('lowerFrameworkImports — real compiler import forms', () => {
  it('strips a no-space side-effect import (import"…")', () => {
    const out = lowerFrameworkImports('import"@ship/sail-runtime/template-hooks";\nconst x = 1\n')
    expect(out).not.toContain('template-hooks')
    expect(out).toContain('const x = 1')
  })

  it('strips named framework imports while leaving non-framework imports', () => {
    const src =
      'import { bindText, component } from "@ship/sail-runtime";\n' +
      'import { collectElementMarkers } from "@ship/sail-dom";\n' +
      'import { foo } from "./local-dep";\n'
    const out = lowerFrameworkImports(src)
    expect(out).not.toContain('@ship/sail-runtime')
    expect(out).not.toContain('@ship/sail-dom')
    expect(out).toContain('./local-dep') // a controller's own dep stays
  })
})

describe('lowerModuleToExpression — `export { X as default }`', () => {
  it('trails the local bound to default (multi-line export block)', () => {
    const src =
      'import { component } from "@ship/sail-runtime";\n' +
      'const page_default = component(() => ({}));\n' +
      'export {\n  page_default as default\n};\n'
    const out = lowerModuleToExpression(src)
    expect(out).not.toMatch(/export\s*\{/)
    expect(out.trimEnd().endsWith('page_default;')).toBe(true)
    expect(out).toContain('const page_default =')
  })

  it('still handles `export default EXPR`', () => {
    const out = lowerModuleToExpression('export default { setup: () => ({}) };\n')
    expect(out).toContain('const __sui_default__ =')
    expect(out.trimEnd().endsWith('__sui_default__;')).toBe(true)
  })
})
