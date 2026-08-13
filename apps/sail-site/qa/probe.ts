#!/usr/bin/env bun
// LAYOUT PROBE v2 — deterministic render-correctness gate for the component
// viewer. The detail page (/components/<tag>) now mounts the REAL component and
// DRIVES it into its interesting state (overlays open, disclosure expands), so
// the probe measures the real thing — no static fakes, no closed-at-rest blind
// spots.
//
//   node qa/probe.ts <tag> [--theme=all]       # one component, every catalog theme (default)
//   node qa/probe.ts --sweep [--theme=X,Y,Z]   # all components × themes → report.json
//   node qa/probe.ts --sweep --theme=ship,glass   # subset
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
import { chromium } from 'playwright'
import { readFileSync, writeFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { parseThemeList, isDarkTheme } from '../../../packages/sail/tokens/src/qa-themes.ts'

const HERE = dirname(fileURLToPath(import.meta.url))
const BASE = process.env.SAIL_PROBE_BASE ?? 'http://127.0.0.1:7890'
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
    r.setAttribute('theme', t); r.setAttribute('mode', d ? 'dark' : 'light'); r.classList.remove('dark')
  }, [theme, isDarkTheme(theme)])
  await page.waitForTimeout(350)
  return page.evaluate(({ tag, SKIP }) => {
    const skip = new Set(SKIP)
    // L1 language elements are light-DOM classed hosts (`.sail-chip`), not
    // custom-element tags. Pack components remain real CEs (`<sail-…>`).
    // Accept either so the probe is a real dissolve gate, not a false NOT-RENDERED.
    const findHost = (t) => {
      const byTag = document.querySelector(t)
      if (byTag) return byTag
      const bare = t.startsWith('sail-') ? t.slice('sail-'.length) : t
      return document.querySelector(`.sail-${CSS.escape(bare)}`)
        || document.querySelector(`.${CSS.escape(t)}`)
    }
    const el = findHost(tag)
    if (!el) return { defects: ['NOT-RENDERED'] }
    // Bounds reference: the detail stage. Overlays are trapped inside it, so an
    // escape shows as a rect outside these bounds even if visually clipped.
    const stageEl = document.querySelector('.detail-stage') || el.parentElement
    const S = stageEl.getBoundingClientRect()
    const cs = (e) => getComputedStyle(e)
    // Colours in SAIL are authored in oklch, and the browser returns
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
  const themeRaw = argv.find((a) => a.startsWith('--theme='))?.split('=')[1]
  const THEMES = parseThemeList(themeRaw) // default: every catalog theme
  const one = argv[0] && !argv[0].startsWith('--') ? argv[0] : null
  const isSweep = !one || argv.includes('--sweep')
  // Single-tag mode never needs the catalog map (and the map path has moved
  // across site refactors). Sweep still loads it for the full L1/pack list.
  let TAGS
  if (one && one !== '--sweep') {
    TAGS = [one]
  } else {
    // Prefer live L1 detail registry + pack tags when the old category-map is gone.
    let raw = ''
    try {
      raw = readFileSync(join(HERE, '../src/sail-catalog/category-map.ts'), 'utf8')
    } catch {
      try {
        raw = readFileSync(join(HERE, '../routes/components/registry.ts'), 'utf8')
      } catch {
        raw = ''
      }
    }
    const fromCat = [...raw.matchAll(/'(sail-[a-z0-9-]+)':/g)].map((m) => m[1])
    const fromL1 = [...raw.matchAll(/L\('([a-z0-9-]+)'/g)].map((m) => m[1])
    TAGS = [...new Set([...fromCat, ...fromL1])]
    if (!TAGS.length) {
      console.error('no component tags found — pass a tag or restore the catalog map')
      process.exit(2)
    }
  }

  console.log(`qa probe · ${TAGS.length} tag(s) · ${THEMES.length} theme(s): ${THEMES.join(',')}`)
  console.log(`base ${BASE}`)

  const b = await chromium.launch()
  const ctx = await b.newContext({ viewport: { width: 1280, height: 900 } })
  const cluster = {}, perComp = []
  for (const tag of TAGS) {
    const p = await ctx.newPage(); const errs = []
    p.on('pageerror', (e) => errs.push('ERR:' + String(e).slice(0, 60)))
    // Per-theme defect bags so a theme-local fail is not collapsed into one set.
    const byTheme = {}
    const found = new Set(); let host = null
    try {
      await p.goto(`${BASE}/components/${tag}`, { waitUntil: 'domcontentloaded', timeout: 20000 })
      await p.waitForTimeout(1600)
      for (const th of THEMES) {
        const r = await probe(p, tag, th)
        host = r.host
        const defs = [...(r.defects || [])]
        byTheme[th] = { defects: defs, host: r.host }
        defs.forEach((d) => found.add(`${th}:${d}`))
      }
    } catch (e) {
      found.add('LOAD_FAIL')
      for (const th of THEMES) byTheme[th] = { defects: ['LOAD_FAIL'], host: null }
    }
    errs.forEach((e) => found.add(e))
    // Expected suppressions apply to bare defect kinds (strip theme: prefix).
    if (EXPECTED_EMPTY.has(tag)) {
      for (const d of [...found]) if (d.endsWith('EMPTY-RENDER') || d === 'EMPTY-RENDER') found.delete(d)
    }
    for (const prefix of EXPECTED[tag] ?? []) {
      for (const d of [...found]) {
        const kind = d.includes(':') ? d.slice(d.indexOf(':') + 1) : d
        if (kind.startsWith(prefix)) found.delete(d)
      }
    }
    await p.close()
    if (one && one !== '--sweep') {
      console.log(JSON.stringify({ tag, themes: THEMES, host, byTheme, defects: [...found] }, null, 2))
    }
    if (found.size) {
      perComp.push({ tag, host, byTheme, defects: [...found] })
      ;[...found].forEach((d) => {
        const k = d.includes(':') ? d.slice(d.indexOf(':') + 1).split(':')[0] : d.split(':')[0]
        ;(cluster[k] ??= new Set()).add(tag)
      })
    }
    process.stdout.write(found.size ? '✗' : '·')
  }
  await b.close()
  console.log(`\n\n=== CLUSTERED: few-shared or many-unique? ===`)
  Object.entries(cluster).sort((a, b) => b[1].size - a[1].size).forEach(([k, set]) =>
    console.log(`${String(set.size).padStart(2)} components  ${k}   {${[...set].slice(0, 10).join(', ')}${set.size > 10 ? ', …' : ''}}`))
  console.log(`\n${perComp.length}/${TAGS.length} components flagged · ${THEMES.length} themes`)
  if (isSweep || !one || one === '--sweep') {
    writeFileSync(join(HERE, 'report.json'), JSON.stringify({ themes: THEMES, flagged: perComp }, null, 2))
    console.log('→ wrote qa/report.json')
  }
  process.exit(perComp.length ? 1 : 0)
}
