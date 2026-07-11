// L1 conformance spec: <div dropzone> (mechanism-2, native + DROPZONE_SKIN_CSS,
// use:dropzone). Probe: renders as a bordered drop area.
const dz = (a = '', txt = 'Drop files here or click to browse') => `<div dropzone ${a} style="width:280px">${txt}</div>`
export default {
  tag: 'dropzone',
  width: 640,
  rows() {
    return [dz(), dz('tone="primary"')]
  },
  probe() {
    const bad = []
    document.querySelectorAll('[dropzone]').forEach((el) => {
      const cs = getComputedStyle(el)
      const bordered = cs.borderTopStyle !== 'none' && parseFloat(cs.borderTopWidth) > 0
      if (!bordered) bad.push({ why: 'no-border' })
    })
    return { count: document.querySelectorAll('[dropzone]').length, bad: bad.slice(0, 8) }
  },
}
