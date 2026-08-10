// L1 conformance: <legend> stdlib fieldset caption (mechanism-1, Style product on native legend).
// Native host folds sx into utility classes. Paint stamps component.legend.fg (--legend-foreground).
const CLS = 's-padX2 s-fglegendforeground s-fontSizesm s-fontWeightmedium'
const lg = (t) => `<legend class="${CLS}">${t}</legend>`
export default {
  tag: 'legend',
  width: 480,
  rows() {
    return [
      `<fieldset style="width:320px;border:1px solid var(--border);border-radius:var(--radius-md);padding:var(--space-4);margin:0">${lg('Account details')}<input placeholder="Name" style="width:100%"></fieldset>`,
      `<fieldset style="width:320px;border:1px solid var(--border);border-radius:var(--radius-md);padding:var(--space-4);margin:0">${lg('Billing options')}<p style="margin:0;font-size:var(--text-sm)">Caption for the group.</p></fieldset>`,
    ]
  },
  probe() {
    const bad = []
    const clear = (v) => v === 'rgba(0, 0, 0, 0)' || v === 'transparent' || v === ''
    document.querySelectorAll('legend').forEach((el, i) => {
      const cs = getComputedStyle(el)
      const r = el.getBoundingClientRect()
      if (r.width <= 0 || r.height <= 0) bad.push({ i, why: 'not-rendered' })
      if (!(el.textContent || '').trim()) bad.push({ i, why: 'empty-caption' })
      if (clear(cs.color)) bad.push({ i, why: 'no-token-fg', color: cs.color })
      // component.legend.fg → --legend-foreground (not free currentColor silence)
      const ship = el.closest("[theme='ship']")
      if (ship && clear(cs.color)) bad.push({ i, why: 'no-legend-foreground' })
    })
    return { count: document.querySelectorAll('legend').length, bad: bad.slice(0, 8) }
  },
}
