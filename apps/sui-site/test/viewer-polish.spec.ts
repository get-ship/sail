// viewer-polish snapshot spec — captures before/after screenshots of the
// /components catalog and the per-component detail view at the two
// representative tags (sui-button — small, sui-calendar — tall). Used
// for the operator's "compare before/after" acceptance check.
//
// Not part of the production smoke suite. Run with:
//   bunx playwright test test/viewer-polish.spec.ts --reporter=list
//
// Screenshots land in test/viewer-polish-screenshots/ alongside the test.

import { test } from '@playwright/test'
import { mkdirSync } from 'node:fs'
import { join } from 'node:path'

const OUT = join(__dirname, 'viewer-polish-screenshots')
mkdirSync(OUT, { recursive: true })

const ROUTES = [
  { name: 'catalog', path: '/components' },
  { name: 'detail-sui-button', path: '/components/sui-button' },
  { name: 'detail-sui-calendar', path: '/components/sui-calendar' },
] as const

for (const r of ROUTES) {
  test(`screenshot ${r.name} (${r.path})`, async ({ page }) => {
    await page.goto(r.path, { waitUntil: 'domcontentloaded' })
    // Catalog: wait for the first cell. Detail: wait for the stage.
    if (r.name === 'catalog') {
      await page.waitForSelector('[data-test="catalog-cell"]', { timeout: 30_000 })
      // Give every preview's mount + bindHtml a frame to settle.
      await page.waitForTimeout(800)
    } else {
      await page.waitForSelector('[data-test="detail-stage"]', { timeout: 30_000 })
      await page.waitForTimeout(800)
    }
    await page.screenshot({
      path: join(OUT, `${r.name}.png`),
      fullPage: true,
    })
  })
}
