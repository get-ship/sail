// Emit the light-DOM framework contracts as a real CSS asset —
// `packages/sail/tokens/src/contracts.css` — so build-time consumers (the
// deck bundler's `sail.css` aggregation, any non-JS host) read the SAME
// text the runtime injects. The runtime TS constants remain the authoring
// source; this script materializes them, and the drift test in
// `test/contracts-css.test.ts` fails the suite whenever the two diverge
// (regenerate by running this script).
//
//   bun packages/sail/runtime/scripts/emit-contracts-css.ts
//
// (The runtime's client-injection path `ensureLightContracts` is itself a
// migration-window mechanism: once every host serves the asset, injection
// detects the served sheet and no-ops; the injection path dies when the
// last JS-only host converts — complete-cutover doctrine.)
//
// The emitted file ALSO carries the framework's `@font-face` block
// (prepended before the runtime's contracts body). Each face references
// a vendored woff2 via `url('./fonts/<file>')` — the deck bundler's
// `ShipAssetCssPlugin` rewrites those references to the bundled hashed
// asset path at `generate_bundle` time, so the browser sees
// `url('./<file>-<hash>.woff2')` against the same `assets/` directory
// the CSS asset lands in. NO base64 inlining — the woff2 are real,
// cacheable, hashed assets (and Tauri ships them as local files using
// the same rewrite path; the web ⟷ desktop distinction is just URL
// resolution). The `@font-face` block's drift lock lives in
// `packages/sail/tokens/test/fonts.test.ts`.

import { writeFileSync } from 'node:fs'
import { resolve, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'
import { LIGHT_CONTRACTS_CSS, layered } from '../src/styles/index'
import { FONTS } from '../../tokens/src/fonts'

const here = dirname(fileURLToPath(import.meta.url))
const out = resolve(here, '../../tokens/src/contracts.css')

const fontFace = (f: (typeof FONTS)[number]): string =>
  `@font-face {\n` +
  `  font-family: ${JSON.stringify(f.family)};\n` +
  `  font-style: ${f.style};\n` +
  `  font-weight: ${f.weight};\n` +
  `  font-display: swap;\n` +
  `  src: url('./fonts/${f.file}') format('${f.format}');\n` +
  `  unicode-range: ${f.unicodeRange};\n` +
  `}\n`

const fontBanner =
  '/* Framework webfonts — self-hosted, referenced as relative `url(\'./fonts/<file>\')`.\n' +
  ' * The deck bundler (`ShipAssetCssPlugin`, crates/deck/deck-bundler/src/ship_asset_css.rs)\n' +
  ' * emits each woff2 as a hashed asset and rewrites the URL to the bundled path at\n' +
  ' * `generate_bundle` time, so the browser sees `url(\'./<file>-<hash>.woff2\')` against the\n' +
  ' * same `assets/` directory the CSS asset lands in. No base64 inlining — the woff2 are\n' +
  ' * real, cacheable, hashed assets (and Tauri ships them as local files using the same\n' +
  ' * rewrite path). The FONTS manifest at `packages/sail/tokens/src/fonts.ts` is the SSOT;\n' +
  ' * `packages/sail/tokens/test/fonts.test.ts` drift-locks this block against it. */\n'

const fontFaces = FONTS.map(fontFace).join('\n')

const banner =
  '/* GENERATED from @ship/sail-runtime styles — do not hand-edit.\n' +
  ' * Regenerate: bun packages/sail/runtime/scripts/emit-contracts-css.ts\n' +
  ' * The light-DOM framework contracts: tone/size channels (attribute-\n' +
  ' * keyed), interactive-surface baseline, focus/disabled, L1 element\n' +
  ' * skins ride separately via the stdlib registry. Framework webfonts\n' +
  ' * are listed at the top of the file and resolved as hashed assets\n' +
  ' * by the deck bundler at build time. */\n'

writeFileSync(
  out,
  banner + fontBanner + fontFaces + '\n' + layered(LIGHT_CONTRACTS_CSS),
)
console.log(
  `wrote ${out} (${LIGHT_CONTRACTS_CSS.length} bytes contracts, ${FONTS.length} @font-face rule${FONTS.length === 1 ? '' : 's'} at top)`,
)
