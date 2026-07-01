// sui-site smoke test — drives the three spec surfaces (landing,
// /components, /tutorial/[slug]) and asserts no console errors fire on
// the page. `webServer.reuseExistingServer` lets the suite reuse an
// in-flight `bun run dev` (the canonical sui-site serving loop).

import { test, expect, type ConsoleMessage, type Page } from '@playwright/test'

interface Failure { url: string; type: string; text: string }
const failures: Failure[] = []

const KNOWN_BENIGN: ReadonlyArray<string> = [
  // <sui-theme-picker> (catalog-previewed on /components) requests the legacy
  // `/view-tokens.css` seam; the canonical route is `/api/assets/...`.
  // The browser logs a MIME error on the resulting 404. Same shape flagged
  // known-benign in ship-marketing's conformance run.
  'view-tokens.css',
]

function attach(page: Page, label: string): void {
  page.on('console', (msg: ConsoleMessage) => {
    if (msg.type() !== 'error') return
    const txt = msg.text()
    if (KNOWN_BENIGN.some((needle) => txt.includes(needle))) return
    failures.push({ url: page.url(), type: msg.type(), text: txt })
  })
  page.on('pageerror', (err: Error) => {
    const txt = err.message
    if (KNOWN_BENIGN.some((needle) => txt.includes(needle))) return
    failures.push({ url: page.url(), type: 'pageerror', text: txt })
  })
  test.info().annotations.push({ type: 'route', description: label })
}

test.afterAll(async () => {
  if (failures.length === 0) return
  console.error('--- console failures captured ---')
  for (const f of failures) console.error(`  [${f.type}] ${f.text.slice(0, 240)} (@ ${f.url})`)
})

test('landing renders + hero present', async ({ page }) => {
  attach(page, 'landing')
  await page.goto('/', { waitUntil: 'domcontentloaded' })
  await page.waitForSelector('sui-site-shell', { state: 'attached', timeout: 15_000 })
  const heroText = (await page.locator('h1').first().textContent({ timeout: 15_000 })) ?? ''
  expect(heroText.toLowerCase()).toContain('reactive')
})

test('/components renders the catalog', async ({ page }) => {
  attach(page, 'components')
  await page.goto('/components', { waitUntil: 'domcontentloaded' })
  await page.waitForSelector('[data-test="catalog-head"]', { timeout: 30_000 })
  const totalText = (await page.locator('.cat-sub').first().textContent({ timeout: 15_000 })) ?? ''
  expect(totalText).toMatch(/components\s*·\s*\d+\s*packs/)
})

test('/tutorial/[slug] mounts the editor panes + iframe', async ({ page }) => {
  attach(page, 'tutorial')
  await page.goto('/tutorial/01-hello-signal', { waitUntil: 'domcontentloaded' })
  await page.waitForSelector('iframe.tut-preview', { state: 'attached', timeout: 30_000 })
  await page.waitForFunction(
    () => document.querySelectorAll('sui-code-editor').length > 0,
    undefined,
    { timeout: 60_000 },
  )
  const editorCount = await page.locator('sui-code-editor').count()
  expect(editorCount).toBeGreaterThanOrEqual(2)
})

test('no console errors on any of the three surfaces', () => {
  const report = failures.map((f) => `${f.url} :: ${f.text.slice(0, 240)}`).join('\n')
  expect(failures, report || 'no console errors').toEqual([])
})
