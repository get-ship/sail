// L1 conformance spec: <hr> (mechanism-2, native + HR_SKIN_CSS). horizontal /
// tone / dashed / vertical. Same contract as <separator>: thin line
// everywhere, visible in a bordered theme (halo/glass are borderless).

export default {
  tag: 'hr',
  width: 640,
  rows() {
    return [
      '<hr>',
      '<hr tone="primary">',
      '<hr variant="dashed">',
      '<div style="display:flex;height:32px;align-items:center;gap:14px">A<hr orientation="vertical">B<hr orientation="vertical" tone="destructive">C</div>',
    ]
  },
  probe() {
    const bad = []
    const clear = (v) => v === 'rgba(0, 0, 0, 0)' || v === 'transparent' || v === ''
    document.querySelectorAll('hr').forEach((el) => {
      const vertical = el.getAttribute('orientation') === 'vertical'
      const r = el.getBoundingClientRect()
      const thin = vertical ? r.width <= 3 : r.height <= 3
      if (!thin) bad.push({ why: 'not-a-line', vertical, w: Math.round(r.width), h: Math.round(r.height) })
    })
    document.querySelectorAll("[data-theme-preview='ship-light'] hr").forEach((el) => {
      const cs = getComputedStyle(el)
      const borderShows = (s, w) => s !== 'none' && parseFloat(w) > 0
      const visible = !clear(cs.backgroundColor) ||
        borderShows(cs.borderTopStyle, cs.borderTopWidth) || borderShows(cs.borderLeftStyle, cs.borderLeftWidth)
      if (!visible) bad.push({ why: 'not-visible-in-light', bg: cs.backgroundColor, variant: el.getAttribute('variant') })
    })
    return { count: document.querySelectorAll('hr').length, bad: bad.slice(0, 8) }
  },
}
