#!/usr/bin/env node
// LAYOUT PROBE v2 — deterministic render-correctness gate for the component
// viewer. The detail page (/components/<tag>) now mounts the REAL component and
// DRIVES it into its interesting state (overlays open, disclosure expands), so
// the probe measures the real thing — no static fakes, no closed-at-rest blind
// spots.
//
//   node qa/probe.mjs <tag> [--theme=X]         # one component, verbose
//   node qa/probe.mjs --sweep [--theme=X,Y,Z]   # all, clustered + report.json
//
// Signals (all deterministic, scoped to the specimen subtree):
//   NOT-RENDERED   the real <tag> element is absent from the page
//   EMPTY-RENDER   element exists but paints nothing (host + all descendants
//                  zero-area / display:none) — a compiler/runtime failure
//   OUTSIDE-STAGE  a painted box escapes the .detail-stage bounds (rect math,
//                  so it flags even when overflow:hidden clips it visually)
//   OVERFLOW-X     a box scrolls horizontally with overflow-x:visible
//   LOW-CONTRAST   visible text below ~2.5:1 against its effective background
//   ERR:…          an uncaught page error (compiler/runtime break)
import { chromium } from '/home/madvib/dev/Ship/worktrees/sui/sui-sx/node_modules/.bun/playwright@1.61.0/node_modules/playwright/index.mjs'
import { readFileSync, writeFileSync } from 'node:fs'

const BASE = process.env.SAIL_PROBE_BASE ?? 'http://127.0.0.1:7878'
const DARK = new Set(['ship-dark', 'console', 'contrast', 'neutral-dark'])
// Empty-by-contract: these render nothing at the host BY DESIGN, so an
// EMPTY-RENDER on them is expected, not a defect. sail-portal teleports its
// children out of the host; sail-toast-host is an empty stack until a notify
// event fires; sail-render / sail-prop-knobs are viewer internals driven by a JS
// component descriptor the gallery can't supply via attributes.
const EXPECTED_EMPTY = new Set(['sail-portal', 'sail-toast-host', 'sail-render', 'sail-prop-knobs'])
// Defects that are EXPECTED for a given tag by design (not bugs), keyed by
// defect prefix. A drawer/sheet is an edge-anchored viewport modal (escapes the
// stage on purpose); a board/canvas is a pannable surface whose world layer is
// intentionally larger than the frame.
const EXPECTED = {
  'sail-drawer': ['OUTSIDE-STAGE'],
  'sail-sheet': ['OUTSIDE-STAGE'],
  'sail-board': ['OVERFLOW-X'],
  'sail-canvas': ['OVERFLOW-X'],
  'sail-draw': ['OVERFLOW-X'],
  'sail-graph-canvas': ['OVERFLOW-X'],
  // Tall dropdown: swatch grid + 3 HSL sliders + hex row renders correctly but
  // exceeds the compact preview frame (verified visually). Not a render defect.
  'sail-color-picker': ['OUTSIDE-STAGE'],
}
const SKIP = new Set(['style', 'slot', 'script', 'template', 'link', 'meta', 'defs', 'title', 'clippath', 'lineargradient', 'radialgradient', 'stop', 'path', 'svg', 'g'])

export async function probe(page, tag, theme) {
  await page.evaluate(([t, d]) => {
    const r = document.documentElement
    r.setAttribute('data-theme', t); r.classList.toggle('dark', d)
  }, [theme, DARK.has(theme)])
  await page.waitForTimeout(350)
  return page.evaluate(({ tag, SKIP }) => {
    const skip = new Set(SKIP)
    const el = document.querySelector(tag)
    if (!el) return { defects: ['NOT-RENDERED'] }
    // Bounds reference: the detail stage. Overlays are trapped inside it, so an
    // escape shows as a rect outside these bounds even if visually clipped.
    const stageEl = document.querySelector('.detail-stage') || el.parentElement
    const S = stageEl.getBoundingClientRect()
    const cs = (e) => getComputedStyle(e)
    // Colours in SUI are authored in oklch, and the browser returns
    // getComputedStyle().color AS oklch(...). A naive numeric regex garbles
    // that (treats L/C/H as R/G/B), breaking every contrast reading. Rasterise
    // any CSS colour (oklch / rgb / hex / named) to TRUE sRGB via a 1×1 canvas.
    const cx = document.createElement('canvas').getContext('2d', { willReadFrequently: true })
    const toRGBA = (str) => { if (!str) return null; cx.clearRect(0, 0, 1, 1); cx.fillStyle = '#000'; cx.fillStyle = str; if (cx.fillStyle === '#000' && !/^#0{3,6}$|black|rgba?\(0/.test(str)) { /* invalid → treat as transparent */ } cx.fillRect(0, 0, 1, 1); const d = cx.getImageData(0, 0, 1, 1).data; return [d[0], d[1], d[2], d[3] / 255] }
    const parse = (c) => { const p = toRGBA(c); return p ? p.slice(0, 3) : null }
    const alpha = (c) => { const p = toRGBA(c); return p ? p[3] : 1 }
    const lum = ([r, g, b]) => { const f = (v) => { v /= 255; return v <= 0.03928 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4) }; return 0.2126 * f(r) + 0.7152 * f(g) + 0.0722 * f(b) }
    // Page background fallback — the theme's true base, tracking the active theme.
    const pageBg = () => { for (const root of [document.body, document.documentElement]) { const bg = cs(root).backgroundColor; if (bg && alpha(bg) > 0.5) { const p = parse(bg); if (p) return p } } return [255, 255, 255] }
    const effBg = (n) => { let e = n; while (e) { const bg = cs(e).backgroundColor; if (bg && alpha(bg) > 0.5) return parse(bg); const rt = e.getRootNode(); e = e.parentElement || (rt instanceof ShadowRoot ? rt.host : null) } return pageBg() }
    const out = []
    let painted = 0
    const scan = (root) => root.childNodes.forEach((n) => {
      if (n.nodeType !== 1 || skip.has(n.tagName.toLowerCase())) return
      const s = cs(n)
      if (s.display === 'none' || s.visibility === 'hidden') { return }
      if (s.display === 'contents') { scan(n.shadowRoot || n); return }
      const b = n.getBoundingClientRect()
      const sig = n.tagName.toLowerCase() + (n.getAttribute?.('part') ? `[${n.getAttribute('part')}]` : '')
      if (b.width > 4 && b.height > 4) {
        painted++
        if (b.bottom > S.bottom + 48 || b.right > S.right + 48 || b.top < S.top - 48 || b.left < S.left - 48)
          out.push(`OUTSIDE-STAGE:${sig}`)
        if (n.scrollWidth > n.clientWidth + 6 && s.overflowX === 'visible')
          out.push(`OVERFLOW-X:${sig}`)
        const hasText = [...n.childNodes].some((c) => c.nodeType === 3 && c.textContent.trim())
        if (hasText && +s.opacity > 0.3) {
          const fg = parse(s.color), bg = effBg(n)
          if (fg && bg) { const L1 = lum(fg), L2 = lum(bg); const ratio = (Math.max(L1, L2) + 0.05) / (Math.min(L1, L2) + 0.05); if (ratio < 2.5) out.push(`LOW-CONTRAST:${sig}(${ratio.toFixed(1)})`) }
        }
      }
      scan(n.shadowRoot || n)
    })
    const H = el.getBoundingClientRect()
    if (H.width > 4 && H.height > 4) painted++
    scan(el.shadowRoot || el)
    if (painted === 0) out.unshift('EMPTY-RENDER')
    return { host: { w: Math.round(H.width), h: Math.round(H.height) }, painted, defects: [...new Set(out)] }
  }, { tag, SKIP: [...SKIP] })
}

// ── CLI ────────────────────────────────────────────────────────────────────
if (import.meta.url === `file://${process.argv[1]}`) {
  const argv = process.argv.slice(2)
  const themeArg = (argv.find((a) => a.startsWith('--theme=')) || '--theme=ship-light,glass,ship-dark').split('=')[1]
  const THEMES = themeArg.split(',')
  const catMap = readFileSync('src/sail-catalog/category-map.ts', 'utf8')
  const one = argv[0] && !argv[0].startsWith('--') ? argv[0] : null
  const TAGS = one ? [one]
    : [...catMap.matchAll(/'(sail-[a-z-]+)':\s*'(?!true)/g)].map((m) => m[1]).filter((t, i, a) => a.indexOf(t) === i)

  const b = await chromium.launch()
  const ctx = await b.newContext({ viewport: { width: 1280, height: 900 } })
  const cluster = {}, perComp = []
  for (const tag of TAGS) {
    const p = await ctx.newPage(); const errs = []
    p.on('pageerror', (e) => errs.push('ERR:' + String(e).slice(0, 60)))
    const found = new Set(); let host = null
    try {
      await p.goto(`${BASE}/components/${tag}`, { waitUntil: 'domcontentloaded', timeout: 20000 })
      await p.waitForTimeout(1600)
      for (const th of THEMES) { const r = await probe(p, tag, th); host = r.host; (r.defects || []).forEach((d) => found.add(d)) }
    } catch (e) { found.add('LOAD_FAIL') }
    errs.forEach((e) => found.add(e))
    if (EXPECTED_EMPTY.has(tag)) found.delete('EMPTY-RENDER')
    for (const prefix of EXPECTED[tag] ?? []) for (const d of [...found]) if (d.startsWith(prefix)) found.delete(d)
    await p.close()
    if (one) console.log(JSON.stringify({ tag, host, defects: [...found] }, null, 2))
    if (found.size) { perComp.push({ tag, host, defects: [...found] }); [...found].forEach((d) => { const k = d.split(':')[0]; (cluster[k] ??= new Set()).add(tag) }) }
    process.stdout.write(found.size ? '✗' : '·')
  }
  await b.close()
  console.log(`\n\n=== CLUSTERED: few-shared or many-unique? ===`)
  Object.entries(cluster).sort((a, b) => b[1].size - a[1].size).forEach(([k, set]) =>
    console.log(`${String(set.size).padStart(2)} components  ${k}   {${[...set].slice(0, 10).join(', ')}${set.size > 10 ? ', …' : ''}}`))
  console.log(`\n${perComp.length}/${TAGS.length} components flagged`)
  if (!one) {
    writeFileSync('qa/report.json', JSON.stringify({ themes: THEMES, flagged: perComp }, null, 2))
    console.log('→ wrote qa/report.json')
  }
}
