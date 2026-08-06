// L1 conformance: <kbd> stdlib keycap (mechanism-1, Style product on native kbd).
// Native host folds sx into utility classes (no .sail-kbd wrapper).
const CLS =
  'sx-displayinlineblock sx-padYpx sx-padX15 sx-bgmuted sx-borderWidth1 sx-borderborder sx-borderBWidth2 sx-radiussm sx-fgforeground sx-alignmiddle sx-fontFamilymono sx-fontSizexs sx-leadingnone sx-whitespacenowrap'
const k = (t) => `<kbd class="${CLS}">${t}</kbd>`
export default {
  tag: 'kbd',
  width: 480,
  rows() {
    return [`Press ${k('Ctrl')} + ${k('K')}.`]
  },
  probe() {
    const bad = []
    document.querySelectorAll('kbd').forEach((el, i) => {
      const cs = getComputedStyle(el)
      const r = el.getBoundingClientRect()
      if (r.width <= 0 || r.height <= 0) bad.push({ i, why: 'not-rendered' })
      if (cs.backgroundColor === 'rgba(0, 0, 0, 0)' || cs.backgroundColor === 'transparent') {
        bad.push({ i, why: 'no-chip-fill', bg: cs.backgroundColor })
      }
    })
    return { count: document.querySelectorAll('kbd').length, bad: bad.slice(0, 8) }
  },
}
