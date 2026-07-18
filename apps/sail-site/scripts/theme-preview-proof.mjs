// Real-Chromium proof of the theme hatch's decisive assertion (theme-api-spec
// gate #1): `theme.preview({'--brand-hue': …})` sets the token on `:root`, and a
// DESCENDANT that reads it through the real tokens.css var() chain
// (`--primary: oklch(… var(--brand-hue))`) reflects the change under a real
// browser's CSS cascade — which jsdom cannot resolve.
//
// This runs the ACTUAL `@ship/sail-runtime` theme module source (bundled here,
// unmodified). It deliberately does NOT go through the deck bundler / dev
// server: that path is #102-blocked in a fresh worktree (`build.rs: SAIL bundle
// assets are missing or empty`). Once #102 is unblocked, the same three
// assertions belong on a real /theming-live demo route driven like push-proof.mjs.
//
// Run:  bun run --cwd apps/sail-site  node scripts/theme-preview-proof.mjs
//   (needs `bun` on PATH to bundle the module + `playwright` installed)
import { chromium } from 'playwright'
import { execFileSync } from 'node:child_process'
import { mkdtempSync, readFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { fileURLToPath } from 'node:url'
import { dirname, resolve, join } from 'node:path'

const here = dirname(fileURLToPath(import.meta.url))
const themeSrc = resolve(here, '../../../packages/sail/runtime/src/theme.ts')

// Bundle the real module source to an ESM the page can import.
const out = join(mkdtempSync(join(tmpdir(), 'theme-proof-')), 'theme.mjs')
execFileSync('bun', ['build', themeSrc, '--format=esm', '--outfile', out], { stdio: 'inherit' })
const bundle = readFileSync(out, 'utf8')

// A minimal but REAL token chain lifted from packages/sail/tokens/src/tokens.css:
// `--primary` reads the `--brand-hue` channel, so a hue write repaints anything
// painted with `--primary`. `--brand-hue` is declared here so it is a DECLARED
// token (the page owns its tokens) — exactly what the hatch validates against.
const html = `<!doctype html><html><head><style>
  :root { --brand-hue: 58; --primary: oklch(0.67 0.16 var(--brand-hue, 58)); }
</style></head><body>
  <div id="swatch" style="background: var(--primary); width:40px; height:40px"></div>
  <script type="module">
    ${bundle}
    window.__theme = theme
  </script>
</body></html>`

const browser = await chromium.launch()
const page = await browser.newPage()
const errors = []
page.on('console', (m) => { if (m.type() === 'error') errors.push(m.text()) })
page.on('pageerror', (e) => errors.push(String(e)))

let failed = false
const check = (cond, msg) => { if (cond) console.log('PASS:', msg); else { console.error('FAIL:', msg); failed = true } }

try {
  await page.setContent(html, { waitUntil: 'load' })
  const bg = () => page.$eval('#swatch', (el) => getComputedStyle(el).backgroundColor)
  const rootHue = () =>
    page.evaluate(() => getComputedStyle(document.documentElement).getPropertyValue('--brand-hue').trim())

  const before = await bg()
  console.log('swatch before:', before, '| :root --brand-hue:', await rootHue())

  await page.evaluate(() => window.__theme.preview({ '--brand-hue': '260' }))
  check((await rootHue()) === '260', 'theme.preview set --brand-hue on :root')
  const after = await bg()
  console.log('swatch after preview:', after)
  check(before !== after, 'a descendant painted via var(--brand-hue) reflected the write (real CSS cascade)')

  await page.evaluate(() => window.__theme.reset())
  check((await bg()) === before, 'theme.reset restored the base value')

  check(
    await page.evaluate(() => { try { window.__theme.preview({ 'not-a-token': 'x' }); return false } catch { return true } }),
    "theme.preview({'not-a-token'}) refused",
  )
  check(
    await page.evaluate(() => { try { window.__theme.preview({ '--never-declared': '1' }); return false } catch { return true } }),
    'an undeclared --custom-property refused',
  )
  check(errors.length === 0, `zero console errors (${errors.length})`)
} catch (e) {
  console.error('FAIL: exception', e)
  failed = true
} finally {
  await browser.close()
}
process.exit(failed ? 1 : 0)
