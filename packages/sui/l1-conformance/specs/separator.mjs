// L1 conformance spec: <separator> (mechanism-1, compiles-away). horizontal /
// tone / dashed / vertical. Probe asserts each renders as a thin line and is
// visible (background OR dashed border).

const sep = (a = '') => `<div class="sui-separator" ${a}></div>`

export default {
  tag: 'separator',
  width: 640,
  rows() {
    return [
      sep(),
      sep('tone="primary"') + ' ' + sep('variant="dashed"'),
      `<div style="display:flex;height:40px;align-items:center;gap:16px">A${sep('orientation="vertical"')}B${sep('orientation="vertical" tone="destructive"')}C</div>`,
    ]
  },
  probe() {
    const bad = []
    const clear = (v) => v === 'rgba(0, 0, 0, 0)' || v === 'transparent' || v === ''
    // Structural: every separator is a thin line, in every theme.
    document.querySelectorAll('.sui-separator').forEach((el) => {
      const vertical = el.getAttribute('orientation') === 'vertical'
      const r = el.getBoundingClientRect()
      const thin = vertical ? r.width <= 3 : r.height <= 3
      if (!thin) bad.push({ why: 'not-a-line', vertical, w: Math.round(r.width), h: Math.round(r.height) })
    })
    // Visibility is a theme aesthetic (halo/glass are deliberately borderless
    // and separate by space); assert the separator CAN show in a bordered theme.
    document.querySelectorAll("[data-theme-preview='ship-light'] .sui-separator").forEach((el) => {
      const cs = getComputedStyle(el)
      const visible = !clear(cs.backgroundColor) || cs.borderTopStyle === 'dashed' || cs.borderLeftStyle === 'dashed'
      if (!visible) bad.push({ why: 'not-visible-in-light', bg: cs.backgroundColor })
    })
    return { count: document.querySelectorAll('.sui-separator').length, bad: bad.slice(0, 8) }
  },
}
