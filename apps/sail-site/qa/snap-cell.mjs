// Screenshot a specific cell by tag
import { chromium } from 'playwright'

const TAG = process.argv[2] ?? 'sail-slider'
const OUT = process.argv[3] ?? '/tmp/job8-cell.png'
const BASE = process.env.SAIL_PROBE_BASE ?? 'http://127.0.0.1:7879'

const browser = await chromium.launch()
const ctx = await browser.newContext({ viewport: { width: 1440, height: 1100 } })
const page = await ctx.newPage()
await page.goto(`${BASE}/components`, { waitUntil: 'domcontentloaded' })
await page.waitForTimeout(2500)

const handle = await page.evaluateHandle((tag) => {
  const find = (root) => {
    if (!root) return null
    if (root.querySelectorAll) {
      const els = root.querySelectorAll(`[data-tag="${tag}"]`)
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
}, TAG)

if (handle.asElement()) {
  await handle.asElement().scrollIntoViewIfNeeded()
  await page.waitForTimeout(300)
  await handle.asElement().screenshot({ path: OUT })
  console.log('Saved', OUT, 'for tag', TAG)
} else {
  console.log('Tag', TAG, 'not found')
}
await browser.close()