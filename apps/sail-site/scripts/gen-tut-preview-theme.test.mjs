// Guard: gen-tut-preview-theme rewrites framework font URLs off the relative
// form that 404s when the sheet is served from /public.
import { describe, expect, test } from 'bun:test'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import {
  FRAMEWORK_FONT_ROUTE,
  buildTutPreviewThemeCss,
  rewriteFrameworkFontUrls,
} from './gen-tut-preview-theme.mjs'

const here = dirname(fileURLToPath(import.meta.url))
const tokensSrc = resolve(here, '../../../packages/sail/tokens/src')

describe('gen-tut-preview-theme font rewrite', () => {
  test('rewrites single- and double-quoted relative font urls', () => {
    const input = `
@font-face { src: url('./fonts/dm-sans-latin-wght-normal.woff2') format('woff2-variations'); }
@font-face { src: url("./fonts/syne-latin-wght-normal.woff2") format('woff2-variations'); }
`
    const out = rewriteFrameworkFontUrls(input)
    expect(out.includes("url('./fonts/")).toBe(false)
    expect(out.includes('url("./fonts/')).toBe(false)
    expect(out.includes(`url('${FRAMEWORK_FONT_ROUTE}/dm-sans-latin-wght-normal.woff2')`)).toBe(
      true,
    )
    expect(out.includes(`url('${FRAMEWORK_FONT_ROUTE}/syne-latin-wght-normal.woff2')`)).toBe(true)
  })

  test('rewrites unquoted relative font urls', () => {
    const out = rewriteFrameworkFontUrls(
      "src: url(./fonts/press-start-2p-latin-normal.woff2) format('woff2');",
    )
    expect(out.includes('url(./fonts/')).toBe(false)
    expect(out).toBe(
      `src: url('${FRAMEWORK_FONT_ROUTE}/press-start-2p-latin-normal.woff2') format('woff2');`,
    )
  })

  test('leaves non-font urls alone', () => {
    const input = `.x { background: url('./img.png'); } .y { background: url(data:image/png;base64,XX); }`
    expect(rewriteFrameworkFontUrls(input)).toBe(input)
  })

  test('built tut-preview theme has zero relative font urls', () => {
    const { body } = buildTutPreviewThemeCss(tokensSrc)
    expect(body.includes("url('./fonts/")).toBe(false)
    expect(body.includes('url("./fonts/')).toBe(false)
    expect(body.includes('url(./fonts/')).toBe(false)
    expect(body).toMatch(new RegExp(`url\\('${FRAMEWORK_FONT_ROUTE}/[\\w.-]+\\.woff2'\\)`))
    for (const file of [
      'dm-sans-latin-wght-normal.woff2',
      'syne-latin-wght-normal.woff2',
      'jetbrains-mono-latin-wght-normal.woff2',
      'press-start-2p-latin-normal.woff2',
    ]) {
      expect(body.includes(`url('${FRAMEWORK_FONT_ROUTE}/${file}')`)).toBe(true)
    }
  })
})
