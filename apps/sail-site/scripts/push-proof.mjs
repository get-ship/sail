// Browser proof of query()'s PUSH path (data-landing-spec.md BROWSER gate).
// Drives a real headless Chromium against /push-demo: a client method
// returning LiveSource<Tick> makes query() subscribe, and the component
// re-renders on each interval push. Asserts the tick count strictly
// increases across pushes, with zero console errors.
import { chromium } from 'playwright'

const BASE = process.env.BASE_URL || 'http://127.0.0.1:7878'
const OUT = process.env.SHOT_DIR || '/tmp/claude-1000/-home-madvib-dev-Ship-worktrees-data-landing/4072ba8c-806a-4c0b-b1a1-82f41f587beb/scratchpad'

const sleep = (ms) => new Promise((r) => setTimeout(r, ms))

const browser = await chromium.launch()
const page = await browser.newPage()
const consoleErrors = []
page.on('console', (m) => { if (m.type() === 'error') consoleErrors.push(m.text()) })
page.on('pageerror', (e) => consoleErrors.push(String(e)))

const readTick = async () => {
  const el = await page.$('[data-testid="push-tick"]')
  if (!el) return null
  const t = await el.textContent()
  const m = t && t.match(/#(\d+)/)
  return m ? Number(m[1]) : null
}

let failed = false
try {
  await page.goto(`${BASE}/push-demo`, { waitUntil: 'domcontentloaded' })

  // 1. Initial: loading (before first push) → then a tick appears.
  await page.waitForSelector('[data-testid="push-tick"]', { timeout: 8000 })
  const t1 = await readTick()
  await page.screenshot({ path: `${OUT}/push-demo-first.png` })
  console.log(`first tick: #${t1}`)

  // 2. Wait for several more pushes; the SAME cache entry keeps updating.
  await sleep(1600) // ~4 more 400ms pushes
  const t2 = await readTick()
  await page.screenshot({ path: `${OUT}/push-demo-later.png` })
  console.log(`later tick: #${t2}`)

  // 3. The decisive assertions.
  if (t1 == null || t2 == null) { console.error('FAIL: no tick rendered'); failed = true }
  else if (!(t2 > t1)) { console.error(`FAIL: tick did not advance (${t1} -> ${t2})`); failed = true }
  else console.log(`PASS: push path re-rendered on each push (${t1} -> ${t2})`)

  if (consoleErrors.length) { console.error('FAIL: console errors:', consoleErrors); failed = true }
  else console.log('PASS: zero console errors')

  // 4. Sanity: the pull demo still works (query() Promise path).
  await page.goto(`${BASE}/query-demo`, { waitUntil: 'domcontentloaded' })
  await page.waitForSelector('[data-testid="deals-list"]', { timeout: 8000 })
  const rows = await page.$$eval('[data-testid="deal-row"]', (els) => els.length)
  console.log(`query-demo (pull) rows: ${rows}`)
  if (rows < 1) { console.error('FAIL: pull demo rendered no rows'); failed = true }
  else console.log('PASS: pull path still renders')
} catch (e) {
  console.error('FAIL: exception', e)
  failed = true
} finally {
  await browser.close()
}
process.exit(failed ? 1 : 0)
