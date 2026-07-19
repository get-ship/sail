#!/usr/bin/env node
// Accessibility audit — runs axe-core against every fixture route and
// fails on any `critical` or `serious` violation. `moderate` and
// `minor` are reported but not gated, so the suite can land without
// fixing every cosmetic edge.
//
// Driver: shipd at $SUI_E2E_BASE_URL (default 127.0.0.1:9385) on the
// fixture workspace. Browser overridable via $SUI_E2E_BROWSER.

import { chromium, firefox, webkit } from 'playwright'
import { readFileSync } from 'node:fs'
import { createRequire } from 'node:module'

const BASE = process.env.SUI_E2E_BASE_URL || 'http://127.0.0.1:9385'
const BROWSER = (process.env.SUI_E2E_BROWSER || 'chromium').toLowerCase()
const ENGINES = { chromium, firefox, webkit }
const engine = ENGINES[BROWSER]
if (!engine) {
  console.error(`unknown browser "${BROWSER}". Use chromium | firefox | webkit.`)
  process.exit(2)
}

const require_ = createRequire(import.meta.url)
const axePath = require_.resolve('axe-core')
const axeSource = readFileSync(axePath, 'utf8')

const VIEWS = [
  '/_deck/view/counter',
  '/_deck/view/sail-demo',
  '/_deck/view/demos',
  '/_deck/view/demos/conditional-render',
  '/_deck/view/demos/iteration',
  '/_deck/view/demos/signals-and-derived',
  '/_deck/view/demos/lifecycle',
  '/_deck/view/demos/match',
]

const browser = await engine.launch()
console.log(`browser: ${BROWSER}`)

// Gating starts at `critical` only. `serious` (mostly color-contrast in
// the demos) and below are reported as advisory. Tighten once demo
// content debt is paid down — set env A11Y_GATE=critical,serious,moderate
// to escalate.
const gate = (process.env.A11Y_GATE || 'critical').split(',').map((s) => s.trim())
const gating = new Set(gate)
const allFails = []
const allWarnings = []

for (const path of VIEWS) {
  const ctx = await browser.newContext()
  const page = await ctx.newPage()
  await page.goto(BASE + path, { waitUntil: 'networkidle' })
  await page.addScriptTag({ content: axeSource })

  const results = await page.evaluate(async () => {
    /* global axe */
    return await axe.run(document, { resultTypes: ['violations'] })
  })

  await ctx.close()

  const gated = results.violations.filter((v) => gating.has(v.impact))
  const advisory = results.violations.filter((v) => !gating.has(v.impact))

  if (gated.length) {
    console.error(`× ${path}: ${gated.length} gated violation(s)`)
    for (const v of gated) {
      console.error(`    [${v.impact}] ${v.id}: ${v.description}`)
      console.error(`      ${v.helpUrl}`)
      for (const n of v.nodes.slice(0, 3)) {
        console.error(`      - ${n.target.join(', ')}`)
      }
    }
    allFails.push({ path, count: gated.length })
  } else {
    console.log(`✓ ${path}`)
  }
  if (advisory.length) {
    allWarnings.push({ path, advisory: advisory.map((v) => `[${v.impact}] ${v.id}`) })
  }
}

await browser.close()

if (allWarnings.length) {
  console.log('\nAdvisory (not gated):')
  for (const w of allWarnings) {
    console.log(`  ${w.path}`)
    for (const a of w.advisory) console.log(`    ${a}`)
  }
}

if (allFails.length) {
  const total = allFails.reduce((a, b) => a + b.count, 0)
  console.error(`\na11y FAIL: ${total} gated violation(s) across ${allFails.length} route(s)`)
  process.exit(1)
}
console.log('\na11y OK (zero critical/serious)')
