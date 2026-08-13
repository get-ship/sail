// @ship/tokens — webfont manifest. Single source of truth for the
// framework's self-hosted woff2 fonts. Variable-weight faces cover the
// Latin block; static display faces (e.g. Press Start 2P) are vendored
// the same way. Each entry's `file` lives alongside this module in
// `./fonts/<file>`. Hand-authored `./sail-base.css` must include one
// `@font-face` block per row with `src: url('./fonts/<file>')`.
//
// Provenance (SPEC-FONTS-1): every face pins a Fontsource (or compatible)
// package id + version + license. The woff2 bytes are the pinned files —
// production paint is offline; no runtime font CDN.
//
// The deck bundler's `ShipAssetCssPlugin` (see
// `crates/deck/deck-bundler/src/ship_asset_css.rs`) reads each `.css`
// it loads, walks the `url(...)` references, emits each referenced
// file as a hashed asset via `ctx.emit_file_async`, and rewrites the
// CSS body's `url(...)` to the hashed relative path. The woff2 land
// at `assets/<file>-<hash>.woff2`; the rewritten CSS reads
// `url('./<file>-<hash>.woff2')` — same directory as the CSS asset.
// Unused families (roles not reachable from the token/style graph)
// are stripped before emit.
//
// To add a font: drop the woff2 into `./fonts/`, append a row here
// AND in `./fonts-manifest.json` (Rust include), add the matching
// `@font-face` in `./sail-base.css`, and the drift test in
// `test/fonts.test.ts` will lock CSS + JSON against this manifest.

/** Font role the face binds to (body / heading / mono / app-custom). */
export type FontRole = 'body' | 'heading' | 'mono' | 'display' | 'press-start' | (string & {})

export interface FontFaceSpec {
  /** CSS `font-family` value (quoted in the emitted rule). */
  family: string
  /** Role this face serves in the default binding. */
  role: FontRole
  /** CSS `font-style`. */
  style: 'normal' | 'italic'
  /**
   * CSS `font-weight` — variable fonts use a range, e.g. `"100 1000"`.
   * Static display faces (e.g. Press Start 2P) use a single weight.
   */
  weight: string
  /** Filename under `./fonts/`; emitted as `url('./fonts/<file>')`. */
  file: string
  /** CSS `format` — variable fonts use `woff2-variations`; static fonts `woff2`. */
  format: 'woff2-variations' | 'woff2'
  /** Fontsource (or compatible) package id. */
  packageId: string
  /** Pinned package version. */
  version: string
  /** SPDX license id (OFL-1.1 for the default catalog). */
  license: string
  /** CSS `unicode-range` (the variable woff2 covers the Latin block). */
  unicodeRange: string
}

export const FONTS: ReadonlyArray<FontFaceSpec> = [
  {
    family: 'DM Sans Variable',
    role: 'body',
    style: 'normal',
    weight: '100 1000',
    file: 'dm-sans-latin-wght-normal.woff2',
    format: 'woff2-variations',
    packageId: '@fontsource-variable/dm-sans',
    version: '5.3.0',
    license: 'OFL-1.1',
    unicodeRange:
      'U+0000-00FF, U+0131, U+0152-0153, U+02BB-02BC, U+02C6, U+02DA, U+02DC, U+0304, U+0308, U+0329, U+2000-206F, U+20AC, U+2122, U+2191, U+2193, U+2212, U+2215, U+FEFF, U+FFFD',
  },
  {
    family: 'Syne Variable',
    role: 'heading',
    style: 'normal',
    weight: '400 800',
    file: 'syne-latin-wght-normal.woff2',
    format: 'woff2-variations',
    packageId: '@fontsource-variable/syne',
    version: '5.3.0',
    license: 'OFL-1.1',
    unicodeRange:
      'U+0000-00FF, U+0131, U+0152-0153, U+02BB-02BC, U+02C6, U+02DA, U+02DC, U+0304, U+0308, U+0329, U+2000-206F, U+20AC, U+2122, U+2191, U+2193, U+2212, U+2215, U+FEFF, U+FFFD',
  },
  {
    family: 'JetBrains Mono Variable',
    role: 'mono',
    style: 'normal',
    weight: '100 800',
    file: 'jetbrains-mono-latin-wght-normal.woff2',
    format: 'woff2-variations',
    packageId: '@fontsource-variable/jetbrains-mono',
    version: '5.3.0',
    license: 'OFL-1.1',
    unicodeRange:
      'U+0000-00FF, U+0131, U+0152-0153, U+02BB-02BC, U+02C6, U+02DA, U+02DC, U+0304, U+0308, U+0329, U+2000-206F, U+20AC, U+2122, U+2191, U+2193, U+2212, U+2215, U+FEFF, U+FFFD',
  },
  {
    // Press Start 2P is a static OFL pixel font used by the 8bit theme.
    // It is not a variable font, so format is `woff2` and weight is a
    // single value. Self-hosted under `./fonts/` so no CDN link is needed.
    // Role `press-start` is reachable only when the 8bit theme (or an
    // explicit role binding) is in the emit set — unused apps must not ship it.
    family: 'Press Start 2P',
    role: 'press-start',
    style: 'normal',
    weight: '400',
    file: 'press-start-2p-latin-normal.woff2',
    format: 'woff2',
    packageId: '@fontsource/press-start-2p',
    version: '5.3.0',
    license: 'OFL-1.1',
    unicodeRange:
      'U+0000-00FF, U+0131, U+0152-0153, U+02BB-02BC, U+02C6, U+02DA, U+02DC, U+0304, U+0308, U+0329, U+2000-206F, U+20AC, U+2122, U+2191, U+2193, U+2212, U+2215, U+FEFF, U+FFFD',
  },
]
