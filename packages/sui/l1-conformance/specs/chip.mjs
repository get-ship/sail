// L1 conformance spec: <chip> (mechanism-1, compiles-away via element mode +
// use:chip). States: every tone, every variant, and the behavioural gates
// (selected/removable/icon/disabled/size). The probe runs the use:chip essentials
// and asserts the ARIA contract + the remove/leading visibility gates.

const dot = '<svg viewBox="0 0 8 8" aria-hidden="true"><circle cx="4" cy="4" r="3" fill="currentColor"/></svg>'
const x = '<button class="remove" type="button" aria-label="Remove" tabindex="-1"><svg viewBox="0 0 10 10" aria-hidden="true"><path d="M2 2 L8 8 M8 2 L2 8"/></svg></button>'
const chip = (label, attrs = '', icon = false) =>
  `<span class="sui-chip" ${attrs}><span class="leading">${icon ? dot : ''}</span>${label}${x}</span>`

export default {
  tag: 'chip',
  width: 760,
  rows() {
    return [
      ['default', 'primary', 'info', 'success', 'warning', 'destructive', 'accent']
        .map((tn) => chip(tn, `tone="${tn}"`)).join(''),
      ['solid', 'soft', 'outline', 'ghost']
        .map((v) => chip(v, `tone="primary" variant="${v}"`)).join(''),
      [
        chip('selected', 'tone="primary" selected'),
        chip('removable', 'tone="info" removable'),
        chip('with icon', 'tone="success"', true),
        chip('disabled', 'tone="primary" disabled'),
        chip('sm', 'tone="primary" size="sm"'),
        chip('lg', 'tone="primary" size="lg"'),
      ].join(''),
    ]
  },
  probe() {
    const bad = []
    document.querySelectorAll('.sui-chip').forEach((el) => {
      // use:chip essentials
      if (!el.hasAttribute('role')) el.setAttribute('role', 'button')
      const disabled = el.hasAttribute('disabled')
      el.setAttribute('aria-pressed', el.hasAttribute('selected') ? 'true' : 'false')
      if (disabled) { el.setAttribute('aria-disabled', 'true'); el.setAttribute('tabindex', '-1') }
      else el.setAttribute('tabindex', '0')
      // invariants
      const roleOk = el.getAttribute('role') === 'button'
      const pressedOk = el.getAttribute('aria-pressed') === (el.hasAttribute('selected') ? 'true' : 'false')
      const tabOk = el.getAttribute('tabindex') === (disabled ? '-1' : '0')
      const removeShown = getComputedStyle(el.querySelector('.remove')).display !== 'none'
      const removeExpected = el.hasAttribute('removable')
      const leadShown = getComputedStyle(el.querySelector('.leading')).display !== 'none'
      const leadExpected = !!el.querySelector('.leading svg')
      if (!roleOk || !pressedOk || !tabOk) bad.push({ why: 'aria', roleOk, pressedOk, tabOk })
      if (removeShown !== removeExpected) bad.push({ why: 'remove-gate', removeShown, removeExpected })
      if (leadShown !== leadExpected) bad.push({ why: 'lead-gate', leadShown, leadExpected })
    })
    return { count: document.querySelectorAll('.sui-chip').length, bad: bad.slice(0, 8) }
  },
}
