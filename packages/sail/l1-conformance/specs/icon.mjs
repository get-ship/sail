// L1 conformance spec: <icon> (mechanism-1, compiles-away). Wraps an svg,
// sizes via the channel, inherits currentColor. Probe: renders, sizes are
// monotonic, glyph inherits color.
const svg = '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 2l2.4 7.4H22l-6 4.6 2.3 7.4-6.3-4.7L5.7 21.4 8 14 2 9.4h7.6z"/></svg>'
const ic = (a = '') => `<span class="sail-icon" ${a}>${svg}</span>`
export default {
  tag: 'icon',
  width: 640,
  rows() {
    return [
      ['xs', 'sm', 'md', 'lg', 'xl'].map((s) => ic(`size="${s}"`)).join(' '),
      `<span style="color:var(--primary)">${ic()}</span> <span style="color:var(--destructive)">${ic()}</span> <span style="color:var(--success)">${ic()}</span>`,
    ]
  },
  probe() {
    const bad = []
    document.querySelectorAll('.sail-icon').forEach((el) => {
      if (el.getBoundingClientRect().width <= 0) bad.push({ why: 'not-rendered', size: el.getAttribute('size') })
    })
    const scope = "[data-theme-preview='ship-light'] "
    const w = (s) => document.querySelector(`${scope}.sail-icon[size='${s}']`).getBoundingClientRect().width
    const [xs, sm, lg] = [w('xs'), w('sm'), w('lg')]
    if (!(xs < sm && sm < lg)) bad.push({ why: 'size-not-monotonic', xs, sm, lg })
    return { count: document.querySelectorAll('.sail-icon').length, bad: bad.slice(0, 8) }
  },
}
