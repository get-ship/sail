#!/usr/bin/env node
// SUI framework E2E runner.
//
// Spins up a chromium page per view in `VIEWS`, asserts the body
// contains the expected sentinel string, and fails on any console
// error or page error.
//
// Run against a daemon serving the fixture workspace:
//   shipd --port 9385 --dev \
//     --workspace packages/sui/runtime/fixtures
//
//   node packages/sui/runtime/fixtures/e2e/run.mjs

import { chromium, firefox, webkit } from 'playwright'

const BASE = process.env.SUI_E2E_BASE_URL || 'http://127.0.0.1:9385'
const BROWSER = (process.env.SUI_E2E_BROWSER || 'chromium').toLowerCase()
const ENGINES = { chromium, firefox, webkit }
const engine = ENGINES[BROWSER]
if (!engine) {
  console.error(`unknown browser "${BROWSER}". Use chromium | firefox | webkit.`)
  process.exit(2)
}

const VIEWS = [
  { path: '/_deck/view/counter', sentinel: 'Counter' },
  { path: '/_deck/view/sui-demo', sentinel: 'SUI' },
  { path: '/_deck/view/demos', sentinel: 'demo' },
  { path: '/_deck/view/demos/conditional-render', sentinel: 'conditional' },
  { path: '/_deck/view/demos/iteration', sentinel: 'iteration' },
  { path: '/_deck/view/demos/signals-and-derived', sentinel: 'signals' },
  { path: '/_deck/view/demos/lifecycle', sentinel: 'lifecycle' },
  { path: '/_deck/view/demos/match', sentinel: 'match' },
]

const browser = await engine.launch()
console.log(`browser: ${BROWSER}`)
const fails = []
const t0 = Date.now()

for (const v of VIEWS) {
  const ctx = await browser.newContext()
  const page = await ctx.newPage()
  const errors = []
  page.on('console', (m) => { if (m.type() === 'error') errors.push(m.text()) })
  page.on('pageerror', (e) => errors.push('pageerror: ' + e.message))

  try {
    await page.goto(BASE + v.path, { waitUntil: 'networkidle', timeout: 10000 })
    const body = (await page.locator('body').innerText()).toLowerCase()
    const localFails = []
    if (!body.includes(v.sentinel.toLowerCase())) {
      localFails.push(`body missing sentinel "${v.sentinel}"`)
    }
    if (errors.length) {
      localFails.push(`console errors:\n  ${errors.join('\n  ')}`)
    }
    if (localFails.length) {
      fails.push({ path: v.path, reasons: localFails })
      console.error(`× ${v.path}`)
      for (const r of localFails) console.error(`    ${r}`)
    } else {
      console.log(`✓ ${v.path}`)
    }
  } catch (e) {
    fails.push({ path: v.path, reasons: [`navigation failed: ${e.message}`] })
    console.error(`× ${v.path}: ${e.message}`)
  } finally {
    await ctx.close()
  }
}

await browser.close()
const dt = ((Date.now() - t0) / 1000).toFixed(1)
console.log(`\n${VIEWS.length - fails.length}/${VIEWS.length} views OK in ${dt}s`)
process.exit(fails.length ? 1 : 0)
