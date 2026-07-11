// L1 conformance spec: <collapsible> (mechanism-1, native <details>). Probe
// asserts the native open/closed contract: open → content laid out; closed →
// content removed (height 0). Rendered closed + open side by side.

const chev = '<svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M6 9l6 6 6-6"/></svg>'
const trig = 'display:flex;align-items:center;justify-content:space-between;gap:8px;padding:12px 14px;font-weight:600;font-size:14px;color:var(--foreground)'
const body = 'padding:0 14px 14px;font-size:14px;color:var(--muted-foreground);line-height:1.5'
const box = 'border:1px solid var(--border);border-radius:10px;background:var(--card);overflow:hidden;width:340px'
const coll = (open) =>
  `<details class="sui-collapsible"${open ? ' open' : ''} style="${box}"><summary class="trigger-wrap" style="${trig}"><span>What is SUI?</span>${chev}</summary><div class="content-wrap"><div class="content-inner" style="${body}">A trustworthy compiler and a beautiful component language — L1 primitives that compile away to static DOM with zero runtime.</div></div></details>`

export default {
  tag: 'collapsible',
  width: 780,
  rows() {
    return [coll(false) + coll(true)]
  },
  probe() {
    const bad = []
    // Measure whether the content ADDS to the details' height — true only when
    // open. Robust to the collapse mechanism (display:none OR clip-to-0 via
    // ::details-content both give detailsH ≈ summaryH when closed).
    document.querySelectorAll('.sui-collapsible').forEach((el) => {
      const open = el.hasAttribute('open')
      const detailsH = el.getBoundingClientRect().height
      const summaryH = el.querySelector('summary').getBoundingClientRect().height
      const contentContributes = detailsH > summaryH + 2
      if (contentContributes !== open) bad.push({ open, detailsH: Math.round(detailsH), summaryH: Math.round(summaryH) })
    })
    return { count: document.querySelectorAll('.sui-collapsible').length, bad }
  },
}
