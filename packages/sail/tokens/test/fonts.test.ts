// @ship/tokens — webfont drift + structural tests.
//
// The woff2 sources for DM Sans Variable, Syne Variable, and JetBrains Mono
// Variable are vendored under `./src/fonts/`. The generated `sail-base.css`
// (materialized by `packages/sail/runtime/scripts/emit-contracts-css.ts` from
// the runtime's `LIGHT_CONTRACTS_CSS` + the `FONTS` manifest in
// `./src/fonts.ts`) emits one `@font-face` per font with a relative
// `url('./fonts/<file>')` reference.
//
// At deck build time the bundler's `ShipAssetCssPlugin`
// (`crates/deck/deck-bundler/src/ship_asset_css.rs`) walks the generated
// CSS, calls `ctx.emit_file_async` for each woff2 reference, and rewrites
// the URL to the bundled hashed asset path. The woff2 ship as real
// `assets/<file>-<hash>.woff2` files; the browser fetches them from the
// same `assets/` directory the CSS lives in. NO base64 inlining.
//
// This file (1) asserts the structural invariants the design system relies
// on (family names resolve to tokens.css tokens, every woff2 file is
// vendored, every file has the woff2 magic, etc.), and (2) drift-locks the
// emitted `sail-base.css` against the FONTS manifest — a woff2 change or a
// manifest row drift would surface as a failing test rather than a silent
// mismatch.

import { readFileSync, existsSync, readFileSync as readFile } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'
import { FONTS } from '../src/fonts'

const loadSrc = (rel: string): string =>
  readFileSync(resolve(dirname(fileURLToPath(import.meta.url)), `../src/${rel}`), 'utf8')

const readWoff2 = (file: string): Buffer =>
  readFile(
    resolve(dirname(fileURLToPath(import.meta.url)), `../src/fonts/${file}`),
  )

function buildExpected(face: (typeof FONTS)[number]): string {
  return (
    `@font-face {\n` +
    `  font-family: ${JSON.stringify(face.family)};\n` +
    `  font-style: ${face.style};\n` +
    `  font-weight: ${face.weight};\n` +
    `  font-display: swap;\n` +
    `  src: url('./fonts/${face.file}') format('${face.format}');\n` +
    `  unicode-range: ${face.unicodeRange};\n` +
    `}\n`
  )
}

describe('@ship/tokens — webfont manifest (FONTS export)', () => {
  it('every family resolves to a token in tokens.css', () => {
    const tokens = loadSrc('tokens.css')
    // Extract every `--font-*-app` value and pull out the quoted family
    // strings. The `*-app` row is where the literal family lives; the rest
    // of the chain is `var()` aliases.
    const re = /--font-[a-z0-9-]+-app\s*:\s*([^;]+);/gi
    const families = new Set<string>()
    let m: RegExpExecArray | null
    while ((m = re.exec(tokens)) !== null) {
      for (const q of m[1]!.matchAll(/"([^"]+)"/g)) families.add(q[1]!)
    }
    for (const face of FONTS) {
      expect(families.has(face.family)).toBe(true)
    }
  })

  it('every `file` references a woff2 that exists in src/fonts/', () => {
    for (const face of FONTS) {
      const path = resolve(
        dirname(fileURLToPath(import.meta.url)),
        '../src/fonts',
        face.file,
      )
      expect(existsSync(path)).toBe(true)
    }
  })

  it('every family declares a valid `font-weight` (variable range or static face)', () => {
    for (const face of FONTS) {
      if (face.format === 'woff2-variations') {
        expect(face.weight).toMatch(/^\d+\s+\d+$/)
        const [lo, hi] = face.weight.split(/\s+/).map(Number)
        expect(hi).toBeGreaterThan(lo!)
      } else {
        expect(face.weight).toMatch(/^\d+$/)
      }
    }
  })

  it('vendored woff2 files have the woff2 magic (no corruption)', () => {
    for (const face of FONTS) {
      const bytes = readWoff2(face.file)
      // woff2 magic: `wOF2` (0x77 0x4F 0x46 0x32)
      expect(bytes[0]).toBe(0x77)
      expect(bytes[1]).toBe(0x4f)
      expect(bytes[2]).toBe(0x46)
      expect(bytes[3]).toBe(0x32)
    }
  })

  it('FONTS is non-empty (the design system has body, display, mono)', () => {
    expect(FONTS.length).toBeGreaterThanOrEqual(3)
    const families = FONTS.map((f) => f.family)
    expect(families).toContain('DM Sans Variable')
    expect(families).toContain('Syne Variable')
    expect(families).toContain('JetBrains Mono Variable')
  })
})

describe('@ship/tokens — sail-base.css webfont drift lock', () => {
  it('every face in FONTS produces the expected @font-face block in sail-base.css', () => {
    const css = loadSrc('sail-base.css')
    for (const face of FONTS) {
      const expected = buildExpected(face)
      expect(css).toContain(expected)
    }
  })

  it('declares a woff2 `format` for every face (variable or static)', () => {
    const css = loadSrc('sail-base.css')
    const formatCount = (css.match(/format\('woff2(?:-variations)?'\)/g) ?? []).length
    expect(formatCount).toBe(FONTS.length)
  })

  it('references the vendored woff2 by relative path (not data: URLs)', () => {
    // The old base64-inline approach is gone. Each woff2 is a separate
    // file the bundler will emit as a hashed asset. Zero data:font URLs.
    const css = loadSrc('sail-base.css')
    expect(css).not.toMatch(/data:font\/woff2/)
    for (const face of FONTS) {
      expect(css).toContain(`url('./fonts/${face.file}')`)
    }
  })
})
