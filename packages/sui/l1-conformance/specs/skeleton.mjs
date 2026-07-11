// L1 conformance spec: <span skeleton> (mechanism-2, native + SKELETON_SKIN_CSS).
// shapes: text / line / block / circle. Probe: renders, filled, animated shimmer.
const sk = (a = '', st = '') => `<span skeleton ${a} style="${st}"></span>`
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
    document.querySelectorAll('span[skeleton]').forEach((el) => {
      const cs = getComputedStyle(el)
      const r = el.getBoundingClientRect()
      if (r.width <= 0 || r.height <= 0) bad.push({ why: 'not-rendered', shape: el.getAttribute('shape') })
      if (clear(cs.backgroundColor) && (!cs.backgroundImage || cs.backgroundImage === 'none')) bad.push({ why: 'no-fill', shape: el.getAttribute('shape') })
      if (cs.animationName === 'none') bad.push({ why: 'not-animated', shape: el.getAttribute('shape') })
    })
    return { count: document.querySelectorAll('span[skeleton]').length, bad: bad.slice(0, 8) }
  },
}
