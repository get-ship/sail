// @vitest-environment node
// Unit-test tree-sitter spans (no CodeMirror / jsdom). Same wasm + scm as
// packages/sail/highlight — the editor maps each span through SCOPE_TO_CM_CLASS.

import { readFileSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { describe, expect, test } from 'vitest'
import {
  highlight,
  initHighlight,
  SCOPE_TO_CM_CLASS,
} from '@ship/sail-highlight'

const here = dirname(fileURLToPath(import.meta.url))
const wasmPath = resolve(here, '../../../../sail/highlight/tree-sitter-sail.wasm')
const scm = readFileSync(
  resolve(here, '../../../../sail/grammar/queries/highlights.scm'),
  'utf8',
)

const FIXTURE = `<scope>
  style card = { pad(4) }
</scope>
<div style(card)>{n()}</div>
`

describe('sail-code-editor tree-sitter highlight', () => {
  test('paints style() / style decl and directive tags', async () => {
    await initHighlight({
      wasm: wasmPath,
      highlightsScm: scm,
    })
    const spans = highlight(FIXTURE)
    const scopes = new Set(spans.map((s) => s.scope))

    expect(scopes.has('keyword') || scopes.has('function')).toBe(true)
    expect(scopes.has('keyword.directive')).toBe(true)

    const bits = spans
      .filter((s) => s.scope === 'keyword' || s.scope === 'function')
      .map((s) => FIXTURE.slice(s.start, s.end))
    expect(bits.some((b) => b.includes('style') || b === 'card')).toBe(true)

    const painted = spans
      .filter((s) => s.scope === 'keyword.directive')
      .map((s) => FIXTURE.slice(s.start, s.end))
    expect(painted.some((b) => b.includes('scope'))).toBe(true)
  })

  test('every span.scope has a SCOPE_TO_CM_CLASS row', async () => {
    await initHighlight({
      wasm: wasmPath,
      highlightsScm: scm,
    })
    const spans = highlight(FIXTURE)
    expect(spans.length).toBeGreaterThan(0)
    for (const span of spans) {
      expect(
        Object.hasOwn(SCOPE_TO_CM_CLASS, span.scope),
        `missing SCOPE_TO_CM_CLASS[${span.scope}]`,
      ).toBe(true)
    }
  })
})
