#!/usr/bin/env bun
/**
 * Blocks browser probe — Playwright against the live composition viewer.
 *
 *   sail dev --project packages/sail/blocks --addr 127.0.0.1:7893
 *   bun packages/sail/blocks/e2e/probe.ts
 *   SAIL_PROBE_BASE=http://127.0.0.1:7893 bun packages/sail/blocks/e2e/probe.ts
 *
 * Evidence: $SHIP_MISSION_SCRATCH/web-qa/blocks/<run>/  (else .artifacts/…)
 */
import { chromium, type Page } from 'playwright'
import { mkdirSync, writeFileSync } from 'node:fs'
import { join } from 'node:path'

const BASE = process.env.SAIL_PROBE_BASE ?? 'http://127.0.0.1:7893'
const REPO = join(import.meta.dir, '../../../..')

const ROUTES = ['/', '/pricing', '/dashboard', '/settings'] as const
const VIEWPORTS = [
  { name: 'desktop', width: 1440, height: 900 },
  { name: 'mobile', width: 375, height: 812 },
] as const
const CELLS = [
  { theme: 'sail', mode: 'light' },
  { theme: 'sail', mode: 'dark' },
] as const

type Finding = { id: string; route: string; cell: string; detail: string }

function qaDir(): string {
  const run = new Date().toISOString().slice(0, 16).replace(/[:T]/g, '-')
  const root = process.env.SHIP_MISSION_SCRATCH
    ? join(process.env.SHIP_MISSION_SCRATCH, 'web-qa')
    : join(REPO, '.artifacts/web-qa')
  const dir = join(root, 'blocks', run)
  mkdirSync(join(dir, 'screenshots'), { recursive: true })
  return dir
}

async function setTheme(page: Page, theme: string, mode: string): Promise<void> {
  await page.evaluate(
    ({ theme, mode }) => {
      const r = document.documentElement
      r.setAttribute('theme', theme)
      r.setAttribute('mode', mode)
    },
    { theme, mode },
  )
  await page.waitForTimeout(200)
}

async function measureHosts(page: Page): Promise<{ tag: string; w: number; h: number; visible: boolean }[]> {
  return page.evaluate(() => {
    const nodes = [...document.querySelectorAll<HTMLElement>('[data-sail-host]')]
    return nodes.map((el) => {
      const r = el.getBoundingClientRect()
      const cs = getComputedStyle(el)
      return {
        tag: el.getAttribute('data-sail-host') || el.tagName.toLowerCase(),
        w: Math.round(r.width),
        h: Math.round(r.height),
        visible:
          r.width > 0 &&
          r.height > 0 &&
          cs.visibility !== 'hidden' &&
          cs.display !== 'none' &&
          parseFloat(cs.opacity) > 0,
      }
    })
  })
}

async function shot(page: Page, dir: string, name: string): Promise<string> {
  const path = join(dir, 'screenshots', `${name}.png`)
  await page.screenshot({ path, fullPage: true })
  return path
}

const DIR = qaDir()
const findings: Finding[] = []
const evidence: unknown[] = []
const consoleErrs: string[] = []
const consoleWarns: string[] = []
const networkFails: string[] = []

function note(route: string, cell: string, detail: string): void {
  findings.push({ id: `F-${String(findings.length + 1).padStart(3, '0')}`, route, cell, detail })
}

const browser = await chromium.launch({ headless: true })
const page = await browser.newPage({ viewport: { width: 1440, height: 900 } })
page.on('pageerror', (e) => consoleErrs.push(String(e).slice(0, 240)))
page.on('console', (msg) => {
  if (msg.type() === 'error') consoleErrs.push(msg.text().slice(0, 240))
  if (msg.type() === 'warning') consoleWarns.push(msg.text().slice(0, 240))
})
page.on('response', (res) => {
  if (res.status() >= 400) networkFails.push(`${res.status()} ${res.url()}`)
})

for (const vp of VIEWPORTS) {
  await page.setViewportSize({ width: vp.width, height: vp.height })
  for (const cell of CELLS) {
    const key = `${vp.name}-${cell.theme}-${cell.mode}`
    for (const route of ROUTES) {
      const label = `${route === '/' ? 'index' : route.slice(1)}-${key}`
      await page.goto(BASE + route, { waitUntil: 'domcontentloaded', timeout: 30_000 })
      await page.waitForTimeout(600)
      await setTheme(page, cell.theme, cell.mode)
      const applied = await page.evaluate(() => ({
        theme: document.documentElement.getAttribute('theme'),
        mode: document.documentElement.getAttribute('mode'),
      }))
      if (applied.theme !== cell.theme || applied.mode !== cell.mode) {
        note(route, key, `theme miss set=${cell.theme}/${cell.mode} got=${applied.theme}/${applied.mode}`)
      }
      const hosts = await measureHosts(page)
      const collapsed = hosts.filter((h) => !h.visible && h.tag !== 'dialog' && h.tag !== 'drawer' && h.tag !== 'hover-card' && h.tag !== 'dropdown' && h.tag !== 'toast' && h.tag !== 'toast-region')
      for (const h of collapsed) {
        note(route, key, `collapsed host ${h.tag} ${h.w}×${h.h}`)
      }
      evidence.push({ route, cell: key, hosts: hosts.length, collapsed: collapsed.length })
      await shot(page, DIR, label)
      process.stdout.write(collapsed.length ? '✗' : '·')
    }
  }
}

// Interactions — desktop light only. Overlays and state, not rest screenshots.
await page.setViewportSize({ width: 1440, height: 900 })
await page.goto(BASE + '/dashboard', { waitUntil: 'domcontentloaded' })
await page.waitForTimeout(500)
await setTheme(page, 'sail', 'light')

const newOrder = page.getByRole('button', { name: 'New order' }).first()
if ((await newOrder.count()) === 0) {
  note('/dashboard', 'interact', 'New order button missing')
} else {
  await newOrder.click()
  await page.waitForTimeout(300)
  const dlg = page.locator('dialog#new-order, [data-sail-host="dialog"]')
  const open = await dlg.evaluateAll((els) =>
    els.some((el) => (el as HTMLDialogElement).open || el.getAttribute('open') !== null),
  )
  if (!open) note('/dashboard', 'interact', 'New order did not open a dialog')
  await shot(page, DIR, 'dashboard-dialog-open')
  const cancel = page.getByRole('button', { name: 'Cancel' })
  if ((await cancel.count()) > 0) await cancel.click()
  await page.waitForTimeout(200)
}

const filters = page.getByRole('button', { name: 'Filters' })
if ((await filters.count()) === 0) {
  note('/dashboard', 'interact', 'Filters button missing')
} else {
  await filters.click()
  await page.waitForTimeout(300)
  const drawer = page.locator('dialog#order-filters, [data-sail-host="drawer"]')
  const open = await drawer.evaluateAll((els) =>
    els.some((el) => (el as HTMLDialogElement).open || el.getAttribute('open') !== null),
  )
  if (!open) note('/dashboard', 'interact', 'Filters did not open a drawer')
  await shot(page, DIR, 'dashboard-drawer-open')
  const done = page.getByRole('button', { name: 'Apply' })
  if ((await done.count()) > 0) await done.click()
  await page.waitForTimeout(200)
}

const customers = page.getByRole('tab', { name: 'Customers' })
if ((await customers.count()) === 0) {
  note('/dashboard', 'interact', 'Customers tab missing')
} else {
  await customers.click()
  await page.waitForTimeout(250)
  const empty = page.getByText('No customer selected')
  if ((await empty.count()) === 0) note('/dashboard', 'interact', 'Customers empty-state missing after tab')
  await shot(page, DIR, 'dashboard-customers-empty')
}

await page.goto(BASE + '/pricing', { waitUntil: 'domcontentloaded' })
await page.waitForTimeout(500)
await setTheme(page, 'sail', 'light')
const annual = page.getByRole('button', { name: 'Annually' })
if ((await annual.count()) === 0) {
  note('/pricing', 'interact', 'Annually toggle missing')
} else {
  await annual.click()
  await page.waitForTimeout(250)
  const saved = page.getByText('2 months free')
  if ((await saved.count()) === 0) note('/pricing', 'interact', 'annual badge did not appear')
  await shot(page, DIR, 'pricing-annual')
}

const faq = page.getByText('Can I switch plans later?')
if ((await faq.count()) === 0) {
  note('/pricing', 'interact', 'FAQ trigger missing')
} else {
  await faq.click()
  await page.waitForTimeout(250)
  const ans = page.getByText('Upgrade or downgrade from settings')
  if ((await ans.count()) === 0) note('/pricing', 'interact', 'FAQ body not visible after click')
  await shot(page, DIR, 'pricing-faq-open')
}

await page.goto(BASE + '/settings', { waitUntil: 'domcontentloaded' })
await page.waitForTimeout(500)
await setTheme(page, 'sail', 'light')
const team = page.getByRole('tab', { name: 'Team' })
if ((await team.count()) === 0) {
  note('/settings', 'interact', 'Team tab missing')
} else {
  await team.click()
  await page.waitForTimeout(250)
  if ((await page.getByText('Ada Lovelace').count()) === 0) {
    note('/settings', 'interact', 'Team table missing Ada Lovelace')
  }
  await shot(page, DIR, 'settings-team')
}

await browser.close()

const status =
  consoleErrs.length || networkFails.length || findings.length ? 'DONE_WITH_CONCERNS' : 'DONE'

writeFileSync(join(DIR, 'evidence.jsonl'), evidence.map((e) => JSON.stringify(e)).join('\n') + '\n')
writeFileSync(
  join(DIR, 'report.json'),
  JSON.stringify(
    {
      surface: 'blocks',
      base: BASE,
      status,
      findings,
      console_errors: consoleErrs,
      console_warnings: consoleWarns,
      network_failures: networkFails,
    },
    null,
    2,
  ),
)

const md = `---
surface: blocks
status: ${status}
run: ${DIR.split('/').pop()}
elements_probed: ${ROUTES.length * VIEWPORTS.length * CELLS.length}
themes_covered: sail-light, sail-dark
viewports_covered: 1440x900, 375x812
console_errors: ${consoleErrs.length}
network_failures: ${networkFails.length}
---

## Summary

Live composition viewer at ${BASE}. ${findings.length} finding(s).

## Findings

${
  findings.length
    ? findings.map((f) => `### ${f.id} — ${f.route} @ ${f.cell}\n- ${f.detail}`).join('\n\n')
    : '_none_'
}

## Console

${consoleErrs.length ? consoleErrs.map((e) => `- ${e}`).join('\n') : '_clean_'}

## Network

${networkFails.length ? networkFails.map((e) => `- ${e}`).join('\n') : '_clean_'}
`
writeFileSync(join(DIR, 'report.md'), md)
writeFileSync(join(DIR, 'plan.md'), `# Blocks probe

Routes: ${ROUTES.join(', ')}
Themes: sail light/dark
Viewports: desktop 1440x900, mobile 375x812
Interactions: New order dialog, Filters drawer, Customers tab, Annually toggle, FAQ, Team tab
`)

console.log(`\n${findings.length} finding(s) · ${consoleErrs.length} console · ${networkFails.length} net`)
console.log(`→ ${DIR}`)
process.exit(findings.length || consoleErrs.length || networkFails.length ? 1 : 0)
