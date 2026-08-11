// L1 conformance spec: <scroll-area> (mechanism-1, compiles-away, pure skin).
// A constrained box whose content overflows → it must be scrollable (overflow
// computed to auto/scroll on the active axis) and render at its constrained size.
// visual.cross: orientation vertical | horizontal | both.

const filler = (n) =>
  Array.from({ length: n }, (_, i) => `<div style="padding:4px 0">row ${i + 1}</div>`).join('')

// Lowered host classes from stdlib scroll-area.sail + element-skins.
const BASE =
  'sail-scroll-area s-overscrollcontain s-scrollbarThin s-scrollbarThumbscrollareathumb s-scrollbarTracktransparent s-hover-scrollbarThumbborderstrong'
const V = 's-orientationvertical-overflowXhidden s-orientationvertical-overflowYauto'
const H = 's-orientationhorizontal-overflowXauto s-orientationhorizontal-overflowYhidden'
const B = 's-orientationboth-overflowauto'

const box = (orientation, extraClass, body, size) =>
  `<div class="${BASE} ${extraClass}" orientation="${orientation}" style="${size};border:1px solid var(--border);border-radius:var(--radius-md);padding:8px">${body}</div>`

export default {
  tag: 'scroll-area',
  width: 640,
  rows() {
    return [
      box('vertical', `${V} l1-mark`, filler(20), 'height:120px;width:220px') +
        box(
          'horizontal',
          H,
          '<div style="display:inline-block;width:600px">wide content that overflows horizontally →</div>',
          'width:220px;white-space:nowrap',
        ) +
        box('both', B, `<div style="width:480px">${filler(16)}</div>`, 'height:120px;width:220px'),
    ]
  },
  probe() {
    const bad = []
    document.querySelectorAll('.sail-scroll-area').forEach((el, i) => {
      const cs = getComputedStyle(el)
      const r = el.getBoundingClientRect()
      if (r.width <= 0 || r.height <= 0) bad.push({ i, why: 'not-rendered' })
      const o = el.getAttribute('orientation') || 'vertical'
      if (o === 'both') {
        if (!/(auto|scroll)/.test(cs.overflowX) && !/(auto|scroll)/.test(cs.overflow)) {
          bad.push({ i, why: 'both-not-scrollable-x', overflowX: cs.overflowX, overflow: cs.overflow })
        }
        if (!/(auto|scroll)/.test(cs.overflowY) && !/(auto|scroll)/.test(cs.overflow)) {
          bad.push({ i, why: 'both-not-scrollable-y', overflowY: cs.overflowY, overflow: cs.overflow })
        }
        const overflows =
          el.scrollWidth > el.clientWidth + 1 || el.scrollHeight > el.clientHeight + 1
        if (!overflows) bad.push({ i, why: 'both-content-not-overflowing' })
      } else {
        const axisOverflow = o === 'horizontal' ? cs.overflowX : cs.overflowY
        if (!/(auto|scroll)/.test(axisOverflow)) bad.push({ i, why: 'not-scrollable', axisOverflow })
        const overflows =
          o === 'horizontal'
            ? el.scrollWidth > el.clientWidth + 1
            : el.scrollHeight > el.clientHeight + 1
        if (!overflows) bad.push({ i, why: 'content-not-overflowing' })
      }
    })
    return { count: document.querySelectorAll('.sail-scroll-area').length, bad: bad.slice(0, 8) }
  },
}
