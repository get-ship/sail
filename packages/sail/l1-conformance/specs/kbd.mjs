// L1 conformance: <kbd> stdlib keycap (mechanism-1, Style product on native kbd).
// Native host folds sx into utility classes (no .sail-kbd wrapper).
// Paint stamps component.kbd tokens (not free muted / foreground / border).
const CLS =
  's-displayinlineblock s-padYpx s-padX15 s-bgkbd s-borderWidth1 s-borderkbdborder s-borderBWidth2 s-radiussm s-fgkbdforeground s-alignmiddle s-fontFamilymono s-fontSizexs s-leadingnone s-whitespacenowrap'
const k = (t) => `<kbd class="${CLS}">${t}</kbd>`
export default {
  tag: 'kbd',
  width: 480,
  rows() {
    return [`Press ${k('Ctrl')} + ${k('K')}.`]
  },
  probe() {
    const bad = []
    const clear = (v) => v === 'rgba(0, 0, 0, 0)' || v === 'transparent' || v === ''
    document.querySelectorAll('kbd').forEach((el, i) => {
      const cs = getComputedStyle(el)
      const r = el.getBoundingClientRect()
      if (r.width <= 0 || r.height <= 0) bad.push({ i, why: 'not-rendered' })
      if (clear(cs.backgroundColor)) {
        bad.push({ i, why: 'no-token-bg', bg: cs.backgroundColor })
      }
      if (clear(cs.color)) bad.push({ i, why: 'no-token-fg' })
      if (clear(cs.borderBottomColor) && clear(cs.borderTopColor)) {
        bad.push({ i, why: 'no-token-border' })
      }
    })
    return { count: document.querySelectorAll('kbd').length, bad: bad.slice(0, 8) }
  },
}
