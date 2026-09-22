import { describe, expect, test } from 'vitest'
import { ensureHighlight, sailTokHtml, toTokHtml, type HighlightSpan } from '../src/index'

/** Strip tok-* spans and undo escaping — must round-trip to the source. */
function plainText(html: string): string {
  return html
    .replace(/<span class="tok-[\w-]+">/g, '')
    .replace(/<\/span>/g, '')
    .replace(/&lt;/g, '<')
    .replace(/&gt;/g, '>')
    .replace(/&amp;/g, '&')
}

function tokens(html: string, cls: string): string[] {
  const re = new RegExp(`<span class="tok-${cls}">([\\s\\S]*?)</span>`, 'g')
  return [...html.matchAll(re)].map((m) => plainText(m[1]!))
}

describe('toTokHtml', () => {
  test('round-trips and escapes when there are no spans', () => {
    const src = '<p>a < b && c > d</p>'
    const html = toTokHtml(src, [])
    expect(html).not.toMatch(/<(?!\/?span\b)/)
    expect(plainText(html)).toBe(src)
  })

  test('wraps mapped scopes and leaves punctuation bare', () => {
    const src = '<div>{n}</div>'
    const spans: HighlightSpan[] = [
      { start: 1, end: 4, scope: 'tag' },
      { start: 5, end: 6, scope: 'punctuation.bracket' },
      { start: 6, end: 7, scope: 'variable' },
      { start: 7, end: 8, scope: 'punctuation.bracket' },
      { start: 10, end: 13, scope: 'tag' },
    ]
    const html = toTokHtml(src, spans)
    expect(tokens(html, 'tag')).toEqual(['div', 'div'])
    expect(tokens(html, 'bind')).toEqual(['n'])
    expect(html).toContain('{<span class="tok-bind">n</span>}')
    expect(plainText(html)).toBe(src)
  })

  test('innermost span wins overlaps; adjacent same class merges', () => {
    const src = 'abcdef'
    const spans: HighlightSpan[] = [
      { start: 0, end: 6, scope: 'string' },
      { start: 2, end: 4, scope: 'keyword' },
    ]
    const html = toTokHtml(src, spans)
    expect(tokens(html, 'string')).toEqual(['ab', 'ef'])
    expect(tokens(html, 'keyword')).toEqual(['cd'])
    expect(plainText(html)).toBe(src)
  })
})

describe('sailTokHtml', () => {
  test('paints a .sail snippet through the grammar', async () => {
    await ensureHighlight()
    const src = `<scope>
  style card = { pad(4) }
</scope>
<div style(card)>{n()}</div>
`
    const html = sailTokHtml(src)
    expect(plainText(html)).toBe(src)
    expect(tokens(html, 'keyword').some((t) => t.includes('scope') || t.includes('style'))).toBe(
      true,
    )
    expect(tokens(html, 'tag').some((t) => t.includes('div'))).toBe(true)
    expect(html).not.toMatch(/<(?!\/?span\b)/)
  })
})
