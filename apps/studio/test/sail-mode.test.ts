import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'
import { join, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'
import {
  tokenSnapshot,
  tokenizeSail,
  skipBrace,
} from '../src/cm/sail-mode'

const FIXTURE = join(
  dirname(fileURLToPath(import.meta.url)),
  'fixtures/sample.sail',
)

describe('CodeMirror sail mode tokenizer', () => {
  it('tokenizes tags, attrs, sx, and exprs on fixture', () => {
    const src = readFileSync(FIXTURE, 'utf8')
    const tokens = tokenizeSail(src)
    const types = new Set(tokens.map((t) => t.type))
    expect(types.has('tagName')).toBe(true)
    expect(types.has('attrName')).toBe(true)
    expect(types.has('sx') || types.has('expr')).toBe(true)
    expect(types.has('comment')).toBe(true)

    const names = tokens.filter((t) => t.type === 'tagName').map((t) => t.text)
    expect(names).toContain('column')
    expect(names).toContain('card')
    expect(names).toContain('button')

    const snap = tokenSnapshot(src)
    expect(snap.length).toBeGreaterThan(10)
    // Stable snapshot of first meaningful tokens
    const interesting = snap.filter(
      (l) =>
        l.startsWith('tagName:') ||
        l.startsWith('attrName:') ||
        l.startsWith('sx:') ||
        l.startsWith('expr:') ||
        l.startsWith('directive:') ||
        l.startsWith('comment:'),
    )
    expect(interesting).toMatchInlineSnapshot(`
      [
        "comment:<!-- Fixture for CM tokenizer + explo…",
        "directive:controller",
        "expr:{ ControllerScope }",
        "tagName:any",
        "expr:{ const ticks = signal(0) return { ti…",
        "directive:controller",
        "directive:scope",
        "expr:{ pad(6).radius(lg) }",
        "directive:scope",
        "tagName:column",
        "attrName:gap",
        "tagName:card",
        "attrName:tone",
        "attrName:style",
        "attrName:sampleCard",
        "tagName:h3",
        "tagName:h3",
        "tagName:p",
        "tagName:p",
        "tagName:row",
        "attrName:gap",
        "tagName:button",
        "attrName:type",
        "attrName:tone",
        "tagName:button",
        "tagName:button",
        "attrName:type",
        "attrName:tone",
        "tagName:button",
        "tagName:row",
        "expr:{ticks().toFixed(0)}",
        "tagName:card",
        "tagName:column",
      ]
    `)
  })

  it('skipBrace respects nested braces and quotes', () => {
    const s = '{style({ pad(6), when: { focus: bg("a}") } })}'
    expect(skipBrace(s, 0)).toBe(s.length)
  })
})
