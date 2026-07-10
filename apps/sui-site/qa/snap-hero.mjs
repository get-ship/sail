// Focused screenshot of the L1 hero.
import { chromium } from 'playwright'
import { mkdirSync } from 'node:fs'

const BASE = process.env.SUI_PROBE_BASE ?? 'http://127.0.0.1:7879'
const OUT = process.argv[2] ?? '/tmp/job8-hero.png'
mkdirSync('/tmp/job8', { recursive: true })

const browser = await chromium.launch()
const ctx = await browser.newContext({ viewport: { width: 1440, height: 1100 } })
const page = await ctx.newPage()
const errs = []
page.on('console', (m) => m.type() === 'error' && errs.push(m.text().slice(0, 200)))
page.on('pageerror', (e) => errs.push('ERR: ' + String(e).slice(0, 200)))
await page.goto(`${BASE}/components`, { waitUntil: 'domcontentloaded' })
await page.waitForTimeout(3000)

// Screenshot the lang-section
const handle = await page.evaluateHandle(() => {
  const find = (root) => {
    if (!root) return null
    if (root.querySelectorAll) {
      const els = root.querySelectorAll('[data-test="lang-section"]')
      if (els.length > 0) return els[0]
      for (const el of root.querySelectorAll('*')) {
        if (el.shadowRoot) {
          const r = find(el.shadowRoot)
          if (r) return r
        }
      }
    }
    return null
  }
  return find(document)
})

if (handle.asElement()) {
  await handle.asElement().screenshot({ path: OUT })
  console.log('Saved', OUT)
} else {
  console.log('Lang section not found')
}
console.log('Errors:', errs)
await browser.close()