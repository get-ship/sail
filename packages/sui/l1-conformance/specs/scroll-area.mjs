// L1 conformance spec: <scroll-area> (mechanism-1, compiles-away, pure skin).
// A constrained box whose content overflows → it must be scrollable (overflow
// computed to auto/scroll on the active axis) and render at its constrained size.

const filler = (n) =>
  Array.from({ length: n }, (_, i) => `<div style="padding:4px 0">row ${i + 1}</div>`).join('')

export default {
  tag: 'scroll-area',
  width: 640,
  rows() {
    return [
      `<div class="sui-scroll-area" orientation="vertical" style="height:120px;width:220px;border:1px solid var(--border);border-radius:var(--radius-md);padding:8px">${filler(20)}</div>` +
        `<div class="sui-scroll-area" orientation="horizontal" style="width:220px;white-space:nowrap;border:1px solid var(--border);border-radius:var(--radius-md);padding:8px"><div style="display:inline-block;width:600px">wide content that overflows horizontally →</div></div>`,
    ]
  },
  probe() {
    const bad = []
    document.querySelectorAll('.sui-scroll-area').forEach((el, i) => {
      const cs = getComputedStyle(el)
      const r = el.getBoundingClientRect()
      if (r.width <= 0 || r.height <= 0) bad.push({ i, why: 'not-rendered' })
      const o = el.getAttribute('orientation')
      const axisOverflow = o === 'horizontal' ? cs.overflowX : cs.overflowY
      if (!/(auto|scroll)/.test(axisOverflow)) bad.push({ i, why: 'not-scrollable', axisOverflow })
      const overflows = o === 'horizontal' ? el.scrollWidth > el.clientWidth + 1 : el.scrollHeight > el.clientHeight + 1
      if (!overflows) bad.push({ i, why: 'content-not-overflowing' })
    })
    return { count: document.querySelectorAll('.sui-scroll-area').length, bad: bad.slice(0, 8) }
  },
}
