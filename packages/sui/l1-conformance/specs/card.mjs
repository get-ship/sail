// L1 conformance spec: <card> (mechanism-1, compiles-away). padding / fill /
// interactive / accent-bar. Probe asserts a resolved surface, monotonic
// padding, and the frozen-token guard on the card surface — the exact --card /
// --surface-1 class of bug that slipped through before.

const card = (a = '', content = 'Card content') => `<div class="sui-card" ${a} style="width:190px">${content}</div>`

export default {
  tag: 'card',
  width: 760,
  rows() {
    return [
      card('padding="sm"', 'Small pad') + card('padding="md"', 'Medium pad') + card('padding="lg"', 'Large pad'),
      card('fill tone="primary"', 'Fill') + card('interactive', 'Interactive') + card('accent-bar="static" tone="primary"', 'Accent bar'),
    ]
  },
  probe() {
    const bad = []
    const csv = (el, p) => getComputedStyle(el)[p]
    const clear = (v) => v === 'rgba(0, 0, 0, 0)' || v === 'transparent' || v === ''
    document.querySelectorAll('.sui-card').forEach((el) => {
      if (clear(csv(el, 'backgroundColor'))) bad.push({ why: 'no-surface', fill: el.hasAttribute('fill') })
    })
    const scope = "[data-theme-preview='ship-light'] "
    const pad = (v) => parseFloat(getComputedStyle(document.querySelector(`${scope}.sui-card[padding='${v}']`)).paddingTop)
    const [sm, md, lg] = [pad('sm'), pad('md'), pad('lg')]
    if (!(sm < md && md < lg)) bad.push({ why: 'padding-not-monotonic', sm, md, lg })
    const mark = (t) => getComputedStyle(document.querySelector(`[data-theme-preview='${t}'] .sui-card`)).backgroundColor
    if (mark('ship-light') === mark('ship-dark')) bad.push({ why: 'frozen-token', light: mark('ship-light'), dark: mark('ship-dark') })
    return { count: document.querySelectorAll('.sui-card').length, bad: bad.slice(0, 8) }
  },
}
