#!/usr/bin/env node
// Counter view: full reactive round-trip.
//   node packages/sui/runtime/fixtures/e2e/counter.mjs

import { chromium, firefox, webkit } from 'playwright'

const BASE = process.env.SUI_E2E_BASE_URL || 'http://127.0.0.1:9385'
const BROWSER = (process.env.SUI_E2E_BROWSER || 'chromium').toLowerCase()
const ENGINES = { chromium, firefox, webkit }
const engine = ENGINES[BROWSER]
if (!engine) {
  console.error(`unknown browser "${BROWSER}". Use chromium | firefox | webkit.`)
  process.exit(2)
}

const browser = await engine.launch()
const ctx = await browser.newContext()
const page = await ctx.newPage()
const errors = []
page.on('console', (m) => { if (m.type() === 'error') errors.push(m.text()) })
page.on('pageerror', (e) => errors.push('pageerror: ' + e.message))

const fails = []
function check(cond, msg) {
  if (!cond) fails.push(msg)
}

await page.goto(BASE + '/_deck/view/counter', { waitUntil: 'networkidle' })
const value = page.locator('[data-test="value"]')

await value.waitFor()
check((await value.textContent())?.trim() === '0', 'initial != 0')

for (let i = 0; i < 3; i++) await page.click('[data-test="inc"]')
check((await value.textContent())?.trim() === '3', `after 3 incs != 3 (got ${(await value.textContent())?.trim()})`)

await page.click('[data-test="dec"]')
check((await value.textContent())?.trim() === '2', 'after dec != 2')

await page.click('[data-test="reset"]')
check((await value.textContent())?.trim() === '0', 'after reset != 0')

// Rapid-click stress: 25 increments back to back, no debounce.
for (let i = 0; i < 25; i++) await page.click('[data-test="inc"]', { delay: 0 })
check((await value.textContent())?.trim() === '25', `rapid 25 incs != 25 (got ${(await value.textContent())?.trim()})`)

if (errors.length) fails.push(`console errors: ${errors.join('; ')}`)

await browser.close()

if (fails.length) {
  console.error('counter E2E FAILED:')
  for (const f of fails) console.error('  ', f)
  process.exit(1)
}
console.log('counter E2E OK')
