// L1 conformance: <kbd> stdlib keycap (mechanism-1, Style product on native kbd).
// Native host folds sx into utility classes (no .sail-kbd wrapper).
// Paint stamps component.kbd tokens (not free muted / foreground / border).
const CLS =
  'sx-displayinlineblock sx-padYpx sx-padX15 sx-bgkbd sx-borderWidth1 sx-borderkbdborder sx-borderBWidth2 sx-radiussm sx-fgkbdforeground sx-alignmiddle sx-fontFamilymono sx-fontSizexs sx-leadingnone sx-whitespacenowrap'
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
