// @ship/tokens — webfont manifest. Single source of truth for the three
// OFL variable woff2 the framework ships. Each entry's `file` lives
// alongside this module in `./fonts/<file>`. The generator
// `packages/sui/runtime/scripts/emit-contracts-css.ts` reads this
// manifest, prepends one `@font-face` block per row to the generated
// `contracts.css` with `src: url('./fonts/<file>')`.
//
// The deck bundler's `ShipAssetCssPlugin` (see
// `crates/deck/deck-bundler/src/ship_asset_css.rs`) reads each `.css`
// it loads, walks the `url(...)` references, emits each referenced
// file as a hashed asset via `ctx.emit_file_async`, and rewrites the
// CSS body's `url(...)` to the hashed relative path. The woff2 land
// at `assets/<file>-<hash>.woff2`; the rewritten CSS reads
// `url('./<file>-<hash>.woff2')` — same directory as the CSS asset.
//
// To add a font: drop the woff2 into `./fonts/`, append a row here,
// re-run `bun packages/sui/runtime/scripts/emit-contracts-css.ts`,
// and the drift test in `test/fonts.test.ts` will lock the result
// against this manifest.

export interface FontFaceSpec {
  /** CSS `font-family` value (quoted in the emitted rule). */
  family: string
  /** CSS `font-style`. */
  style: 'normal' | 'italic'
  /** CSS `font-weight` — variable fonts use a range, e.g. `"100 1000"`. */
  weight: string
  /** Filename under `./fonts/`; emitted as `url('./fonts/<file>')`. */
  file: string
  /** CSS `unicode-range` (the variable woff2 covers the Latin block). */
  unicodeRange: string
}

export const FONTS: ReadonlyArray<FontFaceSpec> = [
  {
    family: 'DM Sans Variable',
    style: 'normal',
    weight: '100 1000',
    file: 'dm-sans-latin-wght-normal.woff2',
    unicodeRange:
      'U+0000-00FF, U+0131, U+0152-0153, U+02BB-02BC, U+02C6, U+02DA, U+02DC, U+0304, U+0308, U+0329, U+2000-206F, U+20AC, U+2122, U+2191, U+2193, U+2212, U+2215, U+FEFF, U+FFFD',
  },
  {
    family: 'Syne Variable',
    style: 'normal',
    weight: '400 800',
    file: 'syne-latin-wght-normal.woff2',
    unicodeRange:
      'U+0000-00FF, U+0131, U+0152-0153, U+02BB-02BC, U+02C6, U+02DA, U+02DC, U+0304, U+0308, U+0329, U+2000-206F, U+20AC, U+2122, U+2191, U+2193, U+2212, U+2215, U+FEFF, U+FFFD',
  },
  {
    family: 'JetBrains Mono Variable',
    style: 'normal',
    weight: '100 800',
    file: 'jetbrains-mono-latin-wght-normal.woff2',
    unicodeRange:
      'U+0000-00FF, U+0131, U+0152-0153, U+02BB-02BC, U+02C6, U+02DA, U+02DC, U+0304, U+0308, U+0329, U+2000-206F, U+20AC, U+2122, U+2191, U+2193, U+2212, U+2215, U+FEFF, U+FFFD',
  },
]
