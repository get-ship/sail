// Drift lock: the materialized `packages/sail/tokens/src/contracts.css`
// (the asset build-time consumers read — the deck bundler's sail.css,
// non-JS hosts) must byte-match the runtime's LIGHT_CONTRACTS_CSS after
// its generated banner. Regenerate on mismatch:
//   bun packages/sail/runtime/scripts/emit-contracts-css.ts
//
// The asset also carries the framework `@font-face` block(s) (prepended
// from `@ship/tokens`'s `FONTS` manifest); the drift check strips them
// AND the matching font banner before comparing against the runtime
// constant — the runtime has no font data and the contracts.css script
// doesn't see fonts. The `FONTS` manifest is drift-locked separately
// in `packages/sail/tokens/test/fonts.test.ts`.

import { readFileSync } from 'node:fs'
import { resolve, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'
import { LIGHT_CONTRACTS_CSS } from '../src/styles/index'

describe('contracts.css asset', () => {
  it('matches the runtime constants (regenerate via emit-contracts-css.ts)', () => {
    const here = dirname(fileURLToPath(import.meta.url))
    const asset = readFileSync(
      resolve(here, '../../tokens/src/contracts.css'),
      'utf8',
    )
    const body = asset.slice(asset.indexOf('*/') + 3) // strip generated banner
    // Strip the framework `@font-face` block(s) the contracts generator
    // prepends. Each block opens with `@font-face {` and runs to its
    // matching `}`. The woff2 are vendored under `@ship/tokens` and
    // drift-locked against the `FONTS` manifest in
    // `packages/sail/tokens/test/fonts.test.ts`; here we only care that
    // the LIGHT_CONTRACTS_CSS body itself is byte-stable.
    let withoutFonts = body.replace(/@font-face\s*\{[^}]*\}\s*/g, '')
    // Strip the `Framework webfonts — ...` banner the generator prepends.
    withoutFonts = withoutFonts.replace(
      /\/\* Framework webfonts —[\s\S]*?\*\/\s*/,
      '',
    )
    // The asset is wrapped in the `sail` cascade layer; unwrap to compare the
    // rule body against the constant.
    const inner = withoutFonts
      .replace(/^@layer sail \{\n/, '')
      .replace(/\}\n$/, '')
    expect(inner).toBe(LIGHT_CONTRACTS_CSS)
  })
})
