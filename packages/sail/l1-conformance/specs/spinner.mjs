// L1 conformance spec: <spinner> (mechanism-1, compiles-away). sizes + tones.
// Probe asserts every spinner renders (non-zero) and is actually animated
// (spin), on itself or a descendant.

const TONES = ['primary', 'info', 'success', 'warning', 'destructive', 'accent']
const RING = 's-displayblock s-sizefull s-borderWidthcontrolstroke s-bordersolid s-borderspinnertrack s-borderRcurrentColor s-radiusfull s-animatespin'
const sp = (a) => `<span class="sail-spinner" ${a}><span aria-hidden="true" class="${RING}"></span></span>`

export default {
  tag: 'spinner',
  width: 640,
  rows() {
    return [
      [sp('size="xs" tone="primary"'), sp('size="sm" tone="primary"'), sp('size="md" tone="primary"'), sp('size="lg" tone="primary"')].join(' '),
      TONES.map((t) => sp(`tone="${t}"`)).join(' '),
      // rest (no tone) + centered bool
      [sp(''), sp('centered tone="primary"')].join(' '),
    ]
  },
  probe() {
    const bad = []
    document.querySelectorAll('.sail-spinner').forEach((el) => {
      if (el.getBoundingClientRect().width <= 0) bad.push({ why: 'not-rendered', tone: el.getAttribute('tone') })
      const spins = getComputedStyle(el).animationName !== 'none' ||
        [...el.querySelectorAll('*')].some((c) => getComputedStyle(c).animationName !== 'none')
      if (!spins) bad.push({ why: 'not-animated', tone: el.getAttribute('tone') })
    })
    return { count: document.querySelectorAll('.sail-spinner').length, bad: bad.slice(0, 8) }
  },
}
