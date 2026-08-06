// L1 conformance: <skeleton> stdlib placeholder (mechanism-1 + SKELETON_MAP sheen).
// shapes: text / line / block / circle. Probe: rendered, filled, shimmer animated.
const sk = (a = '', st = '') =>
  `<span class="sail-skeleton" data-sail-host="skeleton" ${a} style="${st}" role="status" aria-label="Loading"></span>`
export default {
  tag: 'skeleton',
  width: 640,
  rows() {
    return [
      `${sk('shape="text" size="md"', 'width:180px')} ${sk('shape="line" size="sm"', 'width:120px')}`,
      `${sk('shape="block"', 'width:120px')} ${sk('shape="circle"', 'width:40px;height:40px')}`,
    ]
  },
  probe() {
    const bad = []
    const clear = (v) => v === 'rgba(0, 0, 0, 0)' || v === 'transparent' || v === ''
    document.querySelectorAll('.sail-skeleton').forEach((el) => {
      const cs = getComputedStyle(el)
      const r = el.getBoundingClientRect()
      if (r.width <= 0 || r.height <= 0) bad.push({ why: 'not-rendered', shape: el.getAttribute('shape') })
      if (clear(cs.backgroundColor) && (!cs.backgroundImage || cs.backgroundImage === 'none'))
        bad.push({ why: 'no-fill', shape: el.getAttribute('shape') })
      if (cs.animationName === 'none') bad.push({ why: 'not-animated', shape: el.getAttribute('shape') })
    })
    return { count: document.querySelectorAll('.sail-skeleton').length, bad: bad.slice(0, 8) }
  },
}
