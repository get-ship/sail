// viewer-polish snapshot + console-error spec — captures before/after
// screenshots of the /components catalog and the per-component detail
// view at the two representative tags (sail-badge — small, sail-calendar
// — tall) AND asserts no NEW console errors land on those surfaces
// after the polish.
//
// Selector notes: every <sail-component-catalog> / <sail-component-viewer>
// renders into a SHADOW root — `page.waitForSelector` with a
// `[data-test="..."]` selector never finds them from the light DOM.
// The helpers below walk into `el.shadowRoot` explicitly.
//
// Known-benign: `<sail-theme-picker>` (catalog-previewed on /components)
// may request `/view-tokens.css`; the browser logs a MIME error on the
// resulting 404. Same shape flagged known-benign in
// ship-marketing's conformance run and in apps/sail-site/test/smoke.spec.ts.
//
// Not wired into `bun run test` — it's a manual artifact for the
// operator's before/after comparison, not a gate. Run with:
//   bunx playwright test test/viewer-polish.spec.ts --reporter=list
//
// Screenshots land in test/viewer-polish-screenshots/ alongside the test.

import { test, expect, type ConsoleMessage, type Page } from '@playwright/test'
import { mkdirSync } from 'node:fs'
import { join, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'

const OUT = join(dirname(fileURLToPath(import.meta.url)), 'viewer-polish-screenshots')
mkdirSync(OUT, { recursive: true })

interface Failure { url: string; type: string; text: string }
const failures: Failure[] = []

const KNOWN_BENIGN: ReadonlyArray<string> = [
  // <sail-theme-picker> (catalog-previewed on /components) may request
  // `/view-tokens.css`; the canonical route is `/api/assets/...`.
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

const ROUTES = [
  { name: 'catalog', path: '/components', expect: 'catalog' as const },
  { name: 'detail-sail-badge', path: '/components/sail-badge', expect: 'detail' as const },
  { name: 'detail-sail-calendar', path: '/components/sail-calendar', expect: 'detail' as const },
] as const

for (const r of ROUTES) {
  test(`screenshot ${r.name} (${r.path})`, async ({ page }) => {
    attach(page, r.name)
    await page.goto(r.path, { waitUntil: 'domcontentloaded' })
    // Walk into the shadow root — every viewer primitive renders there.
    await page.waitForFunction(
      (kind) => {
        const host = document.querySelector(
          kind === 'catalog' ? 'sail-component-catalog' : 'div[slot="route"] [data-test="detail-stage"]',
        )
        if (!host) return false
        // Catalog: shadowRoot lists the cells. Detail: the slotted div
        // holds the data-test attribute on a light-DOM element.
        if (kind === 'catalog') {
          return host.shadowRoot?.querySelectorAll('[data-test="catalog-cell"]')?.length > 30
        }
        return host.getAttribute('data-test') === 'detail-stage'
      },
      r.expect,
      { timeout: 30_000 },
    )
    // Give every preview's mount + bindHtml + raf a frame to settle.
    await page.waitForTimeout(1200)
    await page.screenshot({
      path: join(OUT, `${r.name}.png`),
      fullPage: true,
    })
  })
}

test.afterAll(async () => {
  if (failures.length === 0) return
  console.error('--- console failures captured ---')
  for (const f of failures) console.error(`  [${f.type}] ${f.text.slice(0, 240)} (@ ${f.url})`)
})

test('no NEW console errors on the polished surfaces', () => {
  const report = failures.map((f) => `${f.url} :: ${f.text.slice(0, 240)}`).join('\n')
  expect(failures, report || 'no console errors').toEqual([])
})
