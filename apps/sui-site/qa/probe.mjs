#!/usr/bin/env node
// LAYOUT PROBE — deterministic box-model defect detection, agent-native.
//   node qa/probe.mjs <tag> [--theme=X]            # one component, verbose
//   node qa/probe.mjs --sweep [--theme=X,Y]        # all, CLUSTERED by defect
// Scoped to the SPECIMEN subtree (not the page) so docs/props-table never
// pollute results. Flags only REAL signals. Also importable for the MCP wrapper.
import { chromium } from '/home/madvib/dev/Ship/worktrees/sui/sui-sx/node_modules/.bun/playwright@1.61.0/node_modules/playwright/index.mjs'
import { readFileSync } from 'node:fs'

const BASE = 'http://127.0.0.1:7878'
const DARK = new Set(['ship-dark','console','contrast','neutral-dark'])
const SKIP = new Set(['style','slot','script','template','link','meta','defs','title','clippath','lineargradient'])

export async function probe(page, tag, theme) {
  await page.evaluate(([t,d]) => { const r=document.documentElement; r.setAttribute('data-theme',t); r.classList.toggle('dark',d) }, [theme, DARK.has(theme)])
  await page.waitForTimeout(400)
  return page.evaluate(({tag, SKIP}) => {
    const skip = new Set(SKIP)
    const el = document.querySelector(tag)
    if (!el) return { error: 'not-found' }
    const H = el.getBoundingClientRect()
    const stage = el.closest('[part],[class*="preview"],[class*="stage"]') || el.parentElement
    const S = (stage||el).getBoundingClientRect()
    const cs = e => getComputedStyle(e)
    // WCAG contrast — the deterministic "glass breaks inputs" check: text that
    // has gone (near-)invisible against its effective background.
    const parse = c => { const m = (c||'').match(/[\d.]+/g); return m ? m.slice(0,3).map(Number) : null }
    const alpha = c => { const m = (c||'').match(/[\d.]+/g); return m && m.length > 3 ? +m[3] : 1 }
    const lum = ([r,g,b]) => { const f = v => { v/=255; return v <= 0.03928 ? v/12.92 : Math.pow((v+0.055)/1.055, 2.4) }; return 0.2126*f(r)+0.7152*f(g)+0.0722*f(b) }
    const effBg = el => { let e = el; while (e) { const bg = cs(e).backgroundColor; if (bg && alpha(bg) > 0.5) return parse(bg); const rt = e.getRootNode(); e = e.parentElement || (rt instanceof ShadowRoot ? rt.host : null) } return [255,255,255] }
    const out = []
    const scan = root => root.childNodes.forEach(n => {
      if (n.nodeType !== 1 || skip.has(n.tagName.toLowerCase())) return
      const s = cs(n); if (s.display === 'none' || s.display === 'contents') { (n.shadowRoot||n) && scan(n.shadowRoot||n); return }
      const b = n.getBoundingClientRect()
      const sig = n.tagName.toLowerCase() + (n.getAttribute?.('part') ? `[${n.getAttribute('part')}]` : '')
      // REAL signals only:
      if (b.width > 4 && b.height > 4) {
        if (b.bottom > S.bottom + 48 || b.right > S.right + 48 || b.top < S.top - 48 || b.left < S.left - 48)
          out.push(`OUTSIDE-STAGE:${sig}`)
        if (n.scrollWidth > n.clientWidth + 6 && s.overflowX === 'visible')
          out.push(`OVERFLOW-X:${sig}`)
        // Contrast on elements bearing direct visible text.
        const hasText = [...n.childNodes].some(c => c.nodeType === 3 && c.textContent.trim())
        if (hasText && +s.opacity > 0.3) {
          const fg = parse(s.color), bg = effBg(n)
          if (fg && bg) { const L1 = lum(fg), L2 = lum(bg); const ratio = (Math.max(L1,L2)+0.05)/(Math.min(L1,L2)+0.05); if (ratio < 2.5) out.push(`LOW-CONTRAST:${sig}(${ratio.toFixed(1)})`) }
        }
      }
      scan(n.shadowRoot || n)
    })
    scan(el.shadowRoot || el)
    return { host:{w:Math.round(H.width),h:Math.round(H.height)}, defects:[...new Set(out)] }
  }, { tag, SKIP: [...SKIP] })
}

const argv = process.argv.slice(2)
const themeArg = (argv.find(a=>a.startsWith('--theme='))||'--theme=ship-light,glass,ship-dark').split('=')[1]
const THEMES = themeArg.split(',')
const catMap = readFileSync('src/sui-catalog/category-map.ts','utf8')
const TAGS = argv[0] && !argv[0].startsWith('--') ? [argv[0]]
  : [...catMap.matchAll(/'(sui-[a-z-]+)':\s*'(?!true)/g)].map(m=>m[1]).filter((t,i,a)=>a.indexOf(t)===i)

const b = await chromium.launch()
const ctx = await b.newContext({ viewport:{width:1280,height:900} })
const cluster = {}, perComp = []
for (const tag of TAGS) {
  const p = await ctx.newPage(); const errs=[]
  p.on('pageerror', e=>errs.push('ERR:'+String(e).slice(0,50)))
  const found = new Set()
  try {
    await p.goto(`${BASE}/components/${tag}`,{waitUntil:'domcontentloaded',timeout:20000})
    await p.waitForTimeout(1400)
    for (const th of THEMES) { const r = await probe(p, tag, th); (r.defects||[]).forEach(d=>found.add(`${d}`)) }
  } catch(e){ found.add('LOAD_FAIL') }
  errs.forEach(e=>found.add(e))
  await p.close()
  if (found.size) { perComp.push({tag, defects:[...found]}); [...found].forEach(d=>{ const k=d.split(':')[0]; (cluster[k]??=new Set()).add(tag) }) }
  process.stdout.write(found.size?`✗`:`·`)
}
await b.close()
console.log(`\n\n=== CLUSTERED: is it few-shared or many-unique? ===`)
Object.entries(cluster).sort((a,b)=>b[1].size-a[1].size).forEach(([k,set])=>
  console.log(`${String(set.size).padStart(2)} components  ${k}   {${[...set].slice(0,8).join(', ')}${set.size>8?', …':''}}`))
console.log(`\n${perComp.length}/${TAGS.length} components flagged`)
