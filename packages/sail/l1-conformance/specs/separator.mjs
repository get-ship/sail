// L1 conformance spec: <separator> (mechanism-1, compiles-away). horizontal /
// tone / dashed / vertical. Probe asserts each renders as a thin line and is
// visible (background OR dashed border). Paint stamps separator-line (not free
// border) for default solid rest.

const SEP =
  'sail-separator s-displayblock s-wfull s-hborderwidth s-my2 s-pad0 s-borderWidth0 s-bgseparatorline'
const sep = (a = '') => `<div class="${SEP}" ${a}></div>`

export default {
  tag: 'separator',
  width: 640,
  rows() {
    return [
      sep(),
      sep('tone="muted"') +
        ' ' +
        sep('tone="primary"') +
        ' ' +
        sep('variant="dashed"') +
        ' ' +
        sep('tone="destructive"'),
      `<div style="display:flex;height:40px;align-items:center;gap:16px">A${sep('orientation="vertical"')}B${sep('orientation="vertical" tone="destructive"')}C</div>`,
    ]
  },
  probe() {
    const bad = []
    const clear = (v) => v === 'rgba(0, 0, 0, 0)' || v === 'transparent' || v === ''
    // Structural: every separator is a thin line, in every theme.
    document.querySelectorAll('.sail-separator').forEach((el) => {
      const vertical = el.getAttribute('orientation') === 'vertical'
      const r = el.getBoundingClientRect()
      const thin = vertical ? r.width <= 3 : r.height <= 3
      if (!thin) bad.push({ why: 'not-a-line', vertical, w: Math.round(r.width), h: Math.round(r.height) })
    })
    // Visibility is a theme aesthetic (halo/glass are deliberately borderless
    // and separate by space); assert the separator CAN show in a bordered theme.
    document.querySelectorAll("[theme='ship'] .sail-separator").forEach((el) => {
      const cs = getComputedStyle(el)
      const visible = !clear(cs.backgroundColor) || cs.borderTopStyle === 'dashed' || cs.borderLeftStyle === 'dashed'
      if (!visible) bad.push({ why: 'not-visible-in-light', bg: cs.backgroundColor })
      // Default solid rest: separator-line token resolves to a paint color.
      if ((el.getAttribute('variant') || 'solid') === 'solid' && !el.getAttribute('tone') && clear(cs.backgroundColor)) {
        bad.push({ why: 'no-token-line' })
      }
    })
    return { count: document.querySelectorAll('.sail-separator').length, bad: bad.slice(0, 8) }
  },
}
