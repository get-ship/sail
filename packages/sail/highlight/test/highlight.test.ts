import { readFileSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { describe, expect, test } from 'vitest'
import { highlight, initHighlight } from '../src/index'

const pkg = JSON.parse(
  readFileSync(resolve(dirname(fileURLToPath(import.meta.url)), '../package.json'), 'utf8'),
) as { deckCoAssets?: Record<string, string> }

const here = dirname(fileURLToPath(import.meta.url))
const wasmPath = resolve(here, '../tree-sitter-sail.wasm')
const scm = readFileSync(resolve(here, '../../grammar/queries/highlights.scm'), 'utf8')

describe('tree-sitter-sail highlight', () => {
  test('paints style() and controller/scope tags', async () => {
    await initHighlight({
      wasm: wasmPath,
      highlightsScm: scm,
    })
    const src = `<scope>
  style card = { pad(4) }
</scope>
<div style(card)>{n()}</div>
`
    const spans = highlight(src)
    const scopes = new Set(spans.map((s) => s.scope))
    expect(scopes.has('keyword.directive') || scopes.has('keyword')).toBe(true)
    const bits = spans
      .filter((s) => s.scope === 'keyword' || s.scope === 'function')
      .map((s) => src.slice(s.start, s.end))
    expect(bits.some((b) => b.includes('style') || b === 'card')).toBe(true)
  })

  test('deckCoAssets dests are the site-root files ensureHighlight fetches', () => {
    expect(Object.keys(pkg.deckCoAssets ?? {}).sort()).toEqual([
      'highlights.scm',
      'tree-sitter-sail.wasm',
      'web-tree-sitter.wasm',
    ])
  })
})
