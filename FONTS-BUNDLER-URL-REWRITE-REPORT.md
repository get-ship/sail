# fonts-bundler-url-rewrite — real, self-hosted webfonts via the bundler

## Mission

Replace the base64-inline approach (`job/fonts-p2-webfonts`, gated, NOT merged) with
hashed, cacheable woff2 assets emitted by the deck bundler. The framework's
`contracts.css` ships with `url('./fonts/X.woff2')` references; the bundler must
resolve those to the bundled hashed woff2 path at build time. No CDN, no base64,
no re-read/re-write pass.

## Approach

Three things cooperate:

1. **Woff2 vendored under `@ship/tokens`** — three OFL variable woff2 (DM Sans,
   Syne, JetBrains Mono) live in `packages/sui/tokens/src/fonts/`. The
   `packages/sui/tokens/src/fonts.ts` manifest is the SSOT; a drift test in
   `packages/sui/tokens/test/fonts.test.ts` locks the vendored bytes against
   the manifest.

2. **`emit-contracts-css.ts` emits `url('./fonts/X.woff2')` references** —
   `packages/sui/runtime/scripts/emit-contracts-css.ts` reads the FONTS
   manifest and emits a `@font-face` block per row into `contracts.css`,
   with `src: url('./fonts/X.woff2') format('woff2-variations')`. The drift
   test in `packages/sui/runtime/test/contracts-css.test.ts` strips the
   `@font-face` block + banner before comparing against the runtime constant
   (the runtime has no font data). The `@ship/tokens` drift test asserts the
   `@font-face` block exists with the expected `url(...)` references and
   `format('woff2-variations')` for every face.

3. **`ShipAssetCssPlugin` rewrites at the bundler** —
   `crates/deck/deck-bundler/src/ship_asset_css.rs` is a new Rolldown plugin
   that runs in two phases:

   - **`load` (default order, runs BEFORE the `AssetModulePlugin`)**:
     when a `.css` is loaded, scan the body for `url(...)` references. For
     each unique local file, read its bytes, register it as a hashed asset
     via `ctx.emit_file_async`, and rewrite the CSS body so
     `url('./fonts/X.woff2')` becomes
     `url("__SHIP_CSS_ASSET__#<woff2_ref_id>")`. The CSS file itself is
     then emitted as an asset too, with the rewritten body. The `load`
     returns JS code that exports `__ROLLDOWN_ASSET__#<css_ref_id>` —
     the existing `AssetModulePlugin::render_chunk` hook rewrites that
     placeholder in the JS chunk to the CSS's hashed URL, so the entry's
     CSS link resolves to the real path.

   - **`generate_bundle` (`Post` order)**:
     walk every `Output::Asset` whose body looks like CSS, find each
     `__SHIP_CSS_ASSET__#<ref>` placeholder, resolve it to the woff2's
     final hashed filename via `ctx.get_file_name(ref_id)`, compute
     the relative URL from the CSS asset's filename to the woff2
     asset's filename (both live under `assets/`), and rewrite the
     body in place. The result: the CSS file on disk reads
     `url("./dm-sans-<hash>.woff2")` against its own `assets/`
     directory, and the browser fetches the hashed woff2.

   The mutable-hook path (`generate_bundle` mutates the asset list
   in-place via `Arc::make_mut`) is preferred over a re-read /
   re-write pass — `WriteBundle` runs after files are written to
   disk, when only metadata is mutable, not body bytes. `generate_bundle`
   is the latest stage where the body is still mutable AND the asset
   hashes are known (via `ctx.get_file_name(ref_id)`).

The plugin is **GENERAL**, not a font-only special case. Any `.css` with
`url('./X.<ext>')` references to local files with a known asset
extension (woff2, woff, png, jpg, jpeg, svg, gif, webp) gets the same
treatment. The next non-font asset to use the framework will ride the
same pipeline for free.

## The chosen hook (vs. alternatives)

- **`WriteBundle` rewrite** (the p2 work's suggestion): runs AFTER
  files are written to disk, can mutate metadata but not body bytes.
  Would have to re-read + re-write the file ourselves. Rejected.

- **`generate_bundle` body mutation** (this work): runs while
  `Output::Asset.source` is still mutable via `Arc::make_mut`, the
  asset list is `&mut Vec<Output>`, and the asset hashes are known
  via `ctx.get_file_name(ref_id)`. Single-pass, no re-read.

- **A pre-emit pass that re-reads and rewrites the file**: rejected
  for the same reason as `WriteBundle` — the body is already on
  disk by the time we'd rewrite it, and we lose the bundler's
  single-pass invariant.

## Files touched

```
A  packages/sui/tokens/src/fonts/dm-sans-latin-wght-normal.woff2       (36932 B)
A  packages/sui/tokens/src/fonts/syne-latin-wght-normal.woff2          (34608 B)
A  packages/sui/tokens/src/fonts/jetbrains-mono-latin-wght-normal.woff2 (40404 B)
A  packages/sui/tokens/src/fonts.ts                                   (FONTS manifest, SSOT)
A  packages/sui/tokens/test/fonts.test.ts                             (drift lock)
M  packages/sui/runtime/scripts/emit-contracts-css.ts                 (prepend @font-face from FONTS)
M  packages/sui/runtime/test/contracts-css.test.ts                     (strip @font-face + banner before compare)
A  crates/deck/deck-bundler/src/ship_asset_css.rs                     (new Rolldown plugin)
M  crates/deck/deck-bundler/src/lib.rs                                (pub mod ship_asset_css)
M  crates/deck/deck-bundler/src/rolldown_bundle.rs                     (wire ShipAssetCssPlugin)
M  crates/deck/deck-bundler/src/app.rs                                (wire ShipAssetCssPlugin)
A  crates/deck/deck-bundler/tests/framework_contracts_rewrite.rs       (integration test vs. real @ship/tokens)
M  crates/deck/deck-bundler/src/sui_plugin.rs                         (pub for test access)
```

## The `@font-face` home

`packages/sui/tokens/src/contracts.css` (framework-emitted, drift-locked
by both `runtime/test/contracts-css.test.ts` and `tokens/test/fonts.test.ts`).
The `@font-face` block is generated by
`packages/sui/runtime/scripts/emit-contracts-css.ts` from the FONTS
manifest:

```css
@font-face {
  font-family: "DM Sans Variable";
  font-style: normal;
  font-weight: 100 1000;
  font-display: swap;
  src: url('./fonts/dm-sans-latin-wght-normal.woff2') format('woff2-variations');
  unicode-range: U+0000-00FF, U+0131, U+0152-0153, U+02BB-02BC, U+02C6, U+02DA, U+02DC, U+0304, U+0308, U+0329, U+2000-206F, U+20AC, U+2122, U+2191, U+2193, U+2215, U+2212, U+2215, U+FEFF, U+FFFD;
}
```

The bundler's `ShipAssetCssPlugin` rewrites the `src:` URL to the bundled
hashed path at `generate_bundle` time:

```css
@font-face {
  font-family: "DM Sans Variable";
  …
  src: url("./dm-sans-latin-wght-normal-Xz1IZZA0.woff2") format('woff2-variations');
  …
}
```

The tokens chain is unchanged: `--font-body-app: "DM Sans Variable", …` in
`tokens.css` already resolves; now the `@font-face` block makes that name
load (and load from a real, cacheable, hashed asset).

## Verification — by RUNNING

### Baseline (mission: e5771ed, before this job)

| Probe | Value |
| --- | --- |
| `document.fonts` count | **0** |
| Loaded `FontFace` entries | `[]` |
| `@font-face` rules in any served CSS | **0** |
| `loadedFontFaces` | `[]` |

The p2 work gated the base64 inline path but didn't ship — the framework
emitted `url('./fonts/X.woff2')` references with no assets at those paths,
so the browser fetched `/sui-site/fonts/X.woff2` (404) for every font.

### After this job

| Probe | Value |
| --- | --- |
| `document.fonts` count | **3** |
| Loaded `FontFace` family names | `"DM Sans Variable"`, `"Syne Variable"`, `"JetBrains Mono Variable"` |
| `@font-face` rules in `contracts.css` | **3** |
| `data:font/woff2` URLs in `contracts.css` | **0** |
| Vendored woff2 magic | `wOF2` (`774f4632`) on all three — variable-font format |
| Vendored woff2 sizes | 36932 B (DM Sans), 34608 B (Syne), 40404 B (JetBrains Mono) — matches `@fontsource-variable/*` CDNs |

### Served URLs (live `127.0.0.1:7890` from the build's `dist/assets/`)

| URL | Status | Size | Content-Type |
| --- | --- | --- | --- |
| `assets/contracts-CJsg5l0m.css` | **200** | 37,308 B | `text/css` |
| `assets/dm-sans-latin-wght-normal-Xz1IZZA0.woff2` | **200** | 36,932 B | `font/woff2` |
| `assets/syne-latin-wght-normal-BVsvWWA4.woff2` | **200** | 34,608 B | `font/woff2` |
| `assets/jetbrains-mono-latin-wght-normal-B9CIFXIH.woff2` | **200** | 40,404 B | `font/woff2` |

`contracts.css` is 37KB (was 35KB without `@font-face`; p2's base64-inline
made it 186KB). The `+1.7KB` is the three `@font-face` blocks (now
referencing real assets, not inline data). The browser does 3 separate
woff2 fetches instead of decoding 150KB of base64. Real, cacheable, hashed.

### Floor check — no authored `<style>` / `class=` for fonts

```
$ grep -rE "<style|font-face|font-family" apps/sui-site/src/sui-site-shell/sui-site-shell.sui
(no matches)
```

The shell has zero font-related markup. The only `<style>` text in
`tokens/src/` is a comment in `element-skins.css`. The `@font-face` rules
live in the framework-emitted `contracts.css`.

### Test results

```
$ bun test packages/sui/tokens/ packages/sui/runtime/test/contracts-css.test.ts
72 pass, 0 fail, 230 expect() calls

$ cargo test -p deck-bundler
test result: ok. 124 passed; 0 failed; 0 ignored; 0 measured
test result: ok.   1 passed; 0 failed; 0 ignored; 0 measured  (framework_contracts_rewrite)
test result: ok.   2 passed; 0 failed; 0 ignored; 0 measured  (per_view_resolver)
test result: ok.   0 passed; 0 failed; 0 ignored; 0 measured  (doctests)
```

The new `framework_contracts_rewrite` integration test (in
`crates/deck/deck-bundler/tests/framework_contracts_rewrite.rs`) is the
smoke that proves end-to-end correctness: it builds the real framework
`contracts.css` via the `SuiPlugin + ShipAssetCssPlugin` pipeline,
asserts the three woff2 are emitted as hashed assets, the contracts.css
references them, no `data:font/woff2` URLs remain, and the
`format('woff2-variations')` count is 3.

### Browser probe (Playwright)

A real chromium probe against the served assets:
- Navigates to `127.0.0.1:7890/`, links the contracts.css, renders three
  `<div>` elements using each family.
- After `document.fonts.ready`:
  - `document.fonts.size = 3`
  - `document.fonts` status: all three families `"loaded"`
  - 3 woff2 responses intercepted, all 200 + `font/woff2`

The browser successfully fetched the hashed woff2 from the same
`assets/` directory the CSS asset landed in. The fonts load. The
mission's "Browser: document.fonts count = 3; the three families load"
gate passes.

## How to regenerate

After any woff2 change (or any change to the `FONTS` manifest in
`packages/sui/tokens/src/fonts.ts`):

```sh
bun packages/sui/runtime/scripts/emit-contracts-css.ts
```

The script reads `./src/fonts/<file>` (from `@ship/tokens`),
materializes the `@font-face` block, and rewrites `./contracts.css`
with the new content. Idempotent — re-running on a file that's already
been processed produces the same output. The drift test in
`tokens/test/fonts.test.ts` fails if the inlined `@font-face` block
falls out of sync with the vendored bytes; re-run the script to clear
it. The `runtime/test/contracts-css.test.ts` drift test fails if the
contracts.css body (sans the `@font-face` block) falls out of sync
with the runtime constant; re-run the script to clear it.

After any change to the bundler pipeline (new asset extension
supported, new plugin, etc.):

```sh
cargo test -p deck-bundler
```

The 124 lib tests + 3 integration tests cover the parser, the
placeholder/URL scanner, the comment-skip behavior, the relative URL
math, the load → generate_bundle end-to-end flow, and the real
framework contracts.css smoke.

## Tauri (desktop) — same pattern, no code change

The mission's "Note: the same `@font-face url()` resolves against local
dist files under Tauri" — this work is platform-agnostic. The
`ShipAssetCssPlugin` rewrites `url('./fonts/X.woff2')` to
`url('./<X>-<hash>.woff2')` — both relative paths inside the same
`assets/` directory. Tauri serves the `dist/` directly; the CSS
resolves `./<X>-<hash>.woff2` against `assets/contracts-<hash>.css`
the same way the browser does. No CDN, no base64, no separate
desktop path. The same `cargo run -p deck build` produces a
self-contained dist that ships identically to web and desktop.

## What this work does NOT do

- It does not touch `Output::Chunk` (JS) bodies. The
  `AssetModulePlugin` owns that rewrite; the new placeholder prefix
  is distinct (`__SHIP_CSS_ASSET__#` vs `__ROLLDOWN_ASSET__#`) so
  the two never collide.
- It does not touch non-CSS asset bodies (woff2, png, …). The woff2
  themselves are emitted as bytes and written as-is; only the CSS
  that references them is rewritten.
- It does not resolve `data:` URLs. Those are inlined content; no
  rewrite needed.
- It does not rewrite references inside `/* … */` comments. CSS
  comments can carry illustrative `url(...)` text (the framework
  banner uses `url('./<file>-<hash>.woff2')` as a doc example);
  the parser skips them or it would try to resolve placeholder
  filenames like `<file>-<hash>.woff2` as real assets.

## Future work (NOT this job)

The mission's "Future" paragraph notes a `fonts([...])` root-scope
helper (scope-spec.md §4) as the eventual authoring surface. This work
builds the bundler pipeline (vendored woff2 → emitted `@font-face` →
hashed assets via the bundler) that such a helper would emit into. The
helper itself stays future work.

A future improvement: the `css_placeholder_occurrences` and
`url_references` scanners currently use byte-by-byte parsing (no regex
dependency). A `regex` dep would make the parsers ~30 lines shorter
and easier to read, at the cost of a heavier compile and a small
runtime hit. The byte-by-byte parsers handle all the cases the
framework emits today.
