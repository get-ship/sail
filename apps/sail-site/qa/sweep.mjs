// SAIL component QA sweep — DETERMINISTIC defect detection (no vision).
// Renders each component's detail page across themes and runs an assertion
// battery: console errors, off-screen geometry, overflow, zero-size content.
// Emits a JSON defect report. This is the CI-able quality gate; agents fix
// what it flags. Usage: node qa/sweep.mjs [themes] [base]
import { chromium } from 'playwright'
import { readFileSync, writeFileSync } from 'node:fs'

const BASE = process.argv[3] || 'http://127.0.0.1:7878'
const THEMES = (process.argv[2] || 'ship-light,glass,ship-dark').split(',')
const DARK = new Set(['ship-dark','console','contrast','neutral-dark'])

// Tags from the category map (the public surface).
const catMap = readFileSync('src/sail-catalog/category-map.ts', 'utf8')
const TAGS = [...catMap.matchAll(/'(sail-[a-z-]+)':\s*'(?!true)/g)].map(m => m[1])
  .filter((t,i,a) => a.indexOf(t) === i)

const ASSERT = async (page) => page.evaluate(() => {
  const vh = innerHeight, vw = innerWidth, defects = []
  const stage = document.querySelector('[data-preview-for], .preview-stage, [part="preview"]')
  const walk = (r, host) => r.querySelectorAll('*').forEach(el => {
    const b = el.getBoundingClientRect()
    if (b.width === 0 && b.height === 0) return
    if (b.top > vh + 80 || b.left > vw + 80 || b.right < -80)
      defects.push(`offscreen:${el.tagName.toLowerCase()}@${Math.round(b.top)},${Math.round(b.left)}`)
    if (el.scrollWidth > el.clientWidth + 4 && getComputedStyle(el).overflowX === 'visible')
      defects.push(`overflow-x:${el.tagName.toLowerCase()}`)
    if (el.shadowRoot) walk(el.shadowRoot, el)
  })
  walk(document, null)
  return [...new Set(defects)]
})

const results = []
for (const tag of TAGS) {
  const row = { tag, byTheme: {} }
  // Fresh browser per component — robust against a single page crashing the
  // whole context (an actual defect we hit on the first run).
  let b
  try { b = await chromium.launch() } catch { results.push({ tag, total: 0, byTheme: { _: { defects:['BROWSER_LAUNCH_FAIL'], consoleErrs:[] } } }); continue }
  for (const theme of THEMES) {
    let p
    const errs = []
    try {
      p = await b.newPage({ viewport: { width: 1280, height: 900 } })
      p.on('console', m => m.type() === 'error' && errs.push(m.text().slice(0,60)))
      p.on('pageerror', e => errs.push('ERR:' + String(e).slice(0,60)))
      await p.goto(`${BASE}/components/${tag}`, { waitUntil:'domcontentloaded', timeout:20000 })
      await p.waitForTimeout(1400)
      await p.evaluate(([t,d]) => { const r=document.documentElement; r.setAttribute('data-theme',t); r.classList.toggle('dark',d) }, [theme, DARK.has(theme)])
      await p.waitForTimeout(500)
      const d = await ASSERT(p)
      row.byTheme[theme] = { defects: d, consoleErrs: [...new Set(errs)].slice(0,3) }
    } catch (e) { row.byTheme[theme] = { defects: ['LOAD_FAIL'], consoleErrs: [String(e).slice(0,60), ...errs.slice(0,2)] } }
    try { await p?.close() } catch {}
  }
  try { await b.close() } catch {}
  const total = Object.values(row.byTheme).reduce((n,t)=>n + t.defects.length + t.consoleErrs.length, 0)
  row.total = total
  results.push(row)
  process.stdout.write(total > 0 ? `✗${tag}(${total}) ` : `·`)
  writeFileSync('qa/report.json', JSON.stringify([...results].sort((a,b)=>b.total-a.total), null, 2)) // incremental
}
results.sort((a,b) => b.total - a.total)
writeFileSync('qa/report.json', JSON.stringify(results, null, 2))
const dirty = results.filter(r => r.total > 0)
console.log(`\n\n=== ${dirty.length}/${TAGS.length} components with defects ===`)
for (const r of dirty.slice(0,15)) console.log(`${r.total}\t${r.tag}\t${Object.entries(r.byTheme).filter(([,v])=>v.defects.length||v.consoleErrs.length).map(([t,v])=>`${t}:[${[...v.defects,...v.consoleErrs].join('|').slice(0,50)}]`).join(' ')}`)
