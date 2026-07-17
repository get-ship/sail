// Inspect catalog: pierce shadow DOM to find cells.
import { chromium } from 'playwright'
import { mkdirSync } from 'node:fs'

const BASE = process.env.SAIL_PROBE_BASE ?? 'http://127.0.0.1:7879'
const OUT_DIR = process.argv[2] ?? '/tmp/job8-inspect'
mkdirSync(OUT_DIR, { recursive: true })

const browser = await chromium.launch()
const ctx = await browser.newContext({ viewport: { width: 1440, height: 1100 } })
const page = await ctx.newPage()
const errs = []
page.on('console', (m) => m.type() === 'error' && errs.push(m.text().slice(0, 200)))
page.on('pageerror', (e) => errs.push('ERR: ' + String(e).slice(0, 200)))
await page.goto(`${BASE}/components`, { waitUntil: 'domcontentloaded' })
await page.waitForTimeout(3000)

// Walk all shadow roots to find [data-tag] cells (which live inside <sail-component-catalog>)
const result = await page.evaluate(async () => {
  const findCells = (root, acc) => {
    if (!root) return
    if (root.querySelectorAll) {
      acc.push(...root.querySelectorAll('[data-tag]'))
      for (const el of root.querySelectorAll('*')) {
        if (el.shadowRoot) findCells(el.shadowRoot, acc)
      }
    }
  }
  const acc = []
  findCells(document, acc)
  const out = []
  for (const cell of acc) {
    const tag = cell.getAttribute('data-tag')
    const stage = cell.querySelector('[data-preview-for]')
    const child = stage?.firstElementChild
    if (!child) {
      out.push({ tag, childTag: null, stageChildren: stage?.children.length ?? 0 })
      continue
    }
    const rect = child.getBoundingClientRect()
    const tagName = child.tagName.toLowerCase()
    let visibleText = ''
    let shadowEmpty = true
    if (child.shadowRoot) {
      const allChildren = Array.from(child.shadowRoot.querySelectorAll('*'))
      const visibleCount = allChildren.filter((n) => {
        const r = n.getBoundingClientRect()
        return r.width > 1 && r.height > 1
      }).length
      shadowEmpty = visibleCount === 0
      for (const n of child.shadowRoot.querySelectorAll('*')) {
        if (n.children.length === 0 && (n.textContent ?? '').trim().length > 0) {
          visibleText = (n.textContent ?? '').trim().slice(0, 60)
          break
        }
      }
    }
    out.push({
      tag,
      stageChildren: stage.children.length,
      childTag: tagName,
      childSize: { w: Math.round(rect.width), h: Math.round(rect.height) },
      visibleText,
      shadowEmpty,
      transform: child.style.transform || '',
    })
  }
  return out
})
console.log(JSON.stringify({ count: result.length, blanks: result.filter(r => r.shadowEmpty).length, cells: result, errs: errs.slice(0, 10) }, null, 2))
await browser.close()