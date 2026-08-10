// L1 conformance: <skeleton> stdlib placeholder (mechanism-1 + SKELETON_MAP sheen).
// shapes: text / line / block / circle. Probe: rendered, filled, shimmer animated.
// Harness mirrors freeform shape/size utility stamps from element-skins.

const shapeCls = {
  text: 's-shapetext-wfull s-shapetext-h3 s-shapetext-minH0 s-shapetextsizemd-h3',
  line: 's-shapeline-wfull s-shapeline-h3 s-shapeline-minH0 s-shapelinesizemd-h3',
  block: 's-shapeblock-displayblock s-shapeblock-wfull s-shapeblock-minH20',
  circle: 's-shapecircle-minW7 s-shapecircle-minH7 s-shapecircle-radiusfull s-shapecircle-aspectsquare s-shapecirclesizemd-size7 s-shapecirclesizemd-minW0 s-shapecirclesizemd-minH0',
}

const sk = (shape = 'text', extra = '') => {
  const sc = shapeCls[shape] || shapeCls.text
  return `<span class="sail-skeleton s-displayinlineblock s-minW8 s-minH3 s-alignmiddle s-radiussm ${sc}" data-sail-host="skeleton" shape="${shape}" size="md" style="${extra}" role="status" aria-label="Loading"></span>`
}

export default {
  tag: 'skeleton',
  width: 640,
  rows() {
    return [
      `${sk('text', 'width:180px')} ${sk('line', 'width:120px')}`,
      `${sk('block', 'width:120px')} ${sk('circle', 'width:40px;height:40px')}`,
    ]
  },
  probe() {
    const bad = []
    const clear = (v) => v === 'rgba(0, 0, 0, 0)' || v === 'transparent' || v === ''
    document.querySelectorAll('.sail-skeleton').forEach((el) => {
      const cs = getComputedStyle(el)
      const r = el.getBoundingClientRect()
      if (r.width <= 0 || r.height <= 0)
        bad.push({ why: 'not-rendered', shape: el.getAttribute('shape') })
      // Sheen may ride background-image only (transparent solid bg).
      if (clear(cs.backgroundColor) && (!cs.backgroundImage || cs.backgroundImage === 'none'))
        bad.push({ why: 'no-fill', shape: el.getAttribute('shape') })
      if (cs.animationName === 'none')
        bad.push({ why: 'not-animated', shape: el.getAttribute('shape') })
    })
    return { count: document.querySelectorAll('.sail-skeleton').length, bad: bad.slice(0, 8) }
  },
}
