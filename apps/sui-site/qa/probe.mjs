// Lightweight LAYOUT PROBE — structured box-model data for agents (not pixels).
// Extracts the specimen's layout tree: box, overflow, key computed styles.
// Wrappable as an MCP tool: probe_layout(route, selector) -> JSON.
import { chromium } from '/home/madvib/dev/Ship/worktrees/sui/sui-sx/node_modules/.bun/playwright@1.61.0/node_modules/playwright/index.mjs'
const tag = process.argv[2] || 'sui-input'
const route = process.argv[3] || `/components/${tag}`
const b = await chromium.launch()
const p = await b.newPage({ viewport: { width: 1280, height: 900 } })
const errs = []
p.on('pageerror', e => errs.push(String(e).slice(0,80)))
await p.goto(`http://127.0.0.1:7878${route}`, { waitUntil:'domcontentloaded', timeout:20000 })
await p.waitForTimeout(1800)
const tree = await p.evaluate((tag) => {
  const el = document.querySelector(tag)
  if (!el) return { error: `no <${tag}> on page` }
  const host = el.getBoundingClientRect()
  const rows = []
  const cs = e => getComputedStyle(e)
  const rect = e => { const r = e.getBoundingClientRect(); return { x:Math.round(r.x), y:Math.round(r.y), w:Math.round(r.width), h:Math.round(r.height) } }
  const scan = (root, depth) => root.childNodes.forEach(n => {
    if (n.nodeType !== 1) return
    const s = cs(n), r = rect(n)
    const flags = []
    if (n.scrollWidth > n.clientWidth + 2) flags.push('overflowX')
    if (n.scrollHeight > n.clientHeight + 2) flags.push('overflowY')
    if (r.w === 0 || r.h === 0) flags.push('zero-size')
    if (s.opacity === '0' || s.visibility === 'hidden' || s.display === 'none') flags.push('invisible')
    if (r.y > host.bottom + 40 || r.x > host.right + 40) flags.push('OUTSIDE-HOST')
    rows.push({ d:depth, tag:n.tagName.toLowerCase(), cls:(n.className||'').toString().slice(0,20), box:r, disp:s.display, pos:s.position, flags })
    if (n.shadowRoot) scan(n.shadowRoot, depth+1)
    else scan(n, depth+1)
  })
  scan(el.shadowRoot || el, 0)
  return { host:{x:Math.round(host.x),y:Math.round(host.y),w:Math.round(host.width),h:Math.round(host.height)}, nodes: rows.length, flagged: rows.filter(r=>r.flags.length) }
}, tag)
console.log(JSON.stringify({ tag, pageErrors: errs.slice(0,3), host: tree.host, nodeCount: tree.nodes, flaggedNodes: tree.flagged?.slice(0,10) }, null, 2))
await b.close()
