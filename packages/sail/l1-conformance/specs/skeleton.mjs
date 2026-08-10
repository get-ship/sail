// L1 conformance: <skeleton> stdlib placeholder (mechanism-1 + SKELETON_MAP sheen).
// shapes: text / line / block / circle × size sm|md|lg. Probe: rendered, filled, shimmer.

const shapeSizeCls = {
  text: {
    sm: 's-shapetext-wfull s-shapetext-h3 s-shapetext-minH0 s-shapetextsizesm-h2',
    md: 's-shapetext-wfull s-shapetext-h3 s-shapetext-minH0 s-shapetextsizemd-h3',
    lg: 's-shapetext-wfull s-shapetext-h3 s-shapetext-minH0 s-shapetextsizelg-h4',
  },
  line: {
    sm: 's-shapeline-wfull s-shapeline-h3 s-shapeline-minH0 s-shapelinesizesm-h2',
    md: 's-shapeline-wfull s-shapeline-h3 s-shapeline-minH0 s-shapelinesizemd-h3',
    lg: 's-shapeline-wfull s-shapeline-h3 s-shapeline-minH0 s-shapelinesizelg-h4',
  },
  block: {
    sm: 's-shapeblock-displayblock s-shapeblock-wfull s-shapeblock-minH20',
    md: 's-shapeblock-displayblock s-shapeblock-wfull s-shapeblock-minH20',
    lg: 's-shapeblock-displayblock s-shapeblock-wfull s-shapeblock-minH20',
  },
  circle: {
    sm: 's-shapecircle-radiusfull s-shapecircle-aspectsquare s-shapecirclesizesm-sizeiconsize2xl s-shapecirclesizesm-minW0 s-shapecirclesizesm-minH0',
    md: 's-shapecircle-radiusfull s-shapecircle-aspectsquare s-shapecirclesizemd-size7 s-shapecirclesizemd-minW0 s-shapecirclesizemd-minH0',
    lg: 's-shapecircle-radiusfull s-shapecircle-aspectsquare s-shapecirclesizelg-size12 s-shapecirclesizelg-minW0 s-shapecirclesizelg-minH0',
  },
}

const sk = (shape = 'text', size = 'md', extra = '') => {
  const sc = shapeSizeCls[shape]?.[size] || shapeSizeCls.text.md
  return `<span class="sail-skeleton s-displayinlineblock s-minW8 s-minH3 s-alignmiddle s-radiussm ${sc}" data-sail-host="skeleton" shape="${shape}" size="${size}" style="${extra}" role="status" aria-label="Loading"></span>`
}

export default {
  tag: 'skeleton',
  width: 640,
  rows() {
    return [
      // default-ish rest: text md (zero-config geometry)
      `${sk('text', 'md', 'width:180px')} ${sk('line', 'sm', 'width:120px')} ${sk('line', 'lg', 'width:140px')}`,
      `${sk('block', 'md', 'width:120px')} ${sk('circle', 'sm')} ${sk('circle', 'md')} ${sk('circle', 'lg')}`,
      // size ladder on text
      `${sk('text', 'sm', 'width:100px')} ${sk('text', 'md', 'width:100px')} ${sk('text', 'lg', 'width:100px')}`,
    ]
  },
  probe() {
    const bad = []
    const clear = (v) => v === 'rgba(0, 0, 0, 0)' || v === 'transparent' || v === ''
    document.querySelectorAll('.sail-skeleton').forEach((el) => {
      const cs = getComputedStyle(el)
      const r = el.getBoundingClientRect()
      if (r.width <= 0 || r.height <= 0)
        bad.push({ why: 'not-rendered', shape: el.getAttribute('shape'), size: el.getAttribute('size') })
      if (clear(cs.backgroundColor) && (!cs.backgroundImage || cs.backgroundImage === 'none'))
        bad.push({ why: 'no-fill', shape: el.getAttribute('shape') })
      if (cs.animationName === 'none')
        bad.push({ why: 'not-animated', shape: el.getAttribute('shape') })
    })
    return { count: document.querySelectorAll('.sail-skeleton').length, bad: bad.slice(0, 8) }
  },
}
