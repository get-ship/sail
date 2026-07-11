// L1 conformance spec: <table> (mechanism-2, native + skin, zero JS). The
// STRUCTURAL primitive only — sorting, selection and virtualization make a data
// grid, which is L2 built on this. Variants ride on the table: [dense] [striped]
// [hoverable] [bordered], plus <thead sticky> and [numeric] columns (right-aligned,
// tabular figures). Probe: header is divided from the body, rows are separated,
// every variant channel actually does something, and the tokens aren't frozen.

const rows = (n = 3) =>
  Array.from({ length: n }, (_, i) =>
    `<tr><td>Row ${i + 1}</td><td>user${i + 1}@ship.dev</td><td numeric>${(i + 1) * 1250}</td></tr>`,
  ).join('')

const table = (attrs = '') =>
  `<table ${attrs}><thead><tr><th>Name</th><th>Email</th><th numeric>Amount</th></tr></thead>` +
  `<tbody>${rows()}</tbody></table>`

export default {
  tag: 'table',
  width: 760,
  rows() {
    return [table('class="l1-mark"'), table('striped'), table('dense bordered')]
  },
  probe() {
    const bad = []
    const cs = (el, p) => getComputedStyle(el)[p]
    const clear = (v) => v === 'rgba(0, 0, 0, 0)' || v === 'transparent' || v === ''

    document.querySelectorAll('table').forEach((el, i) => {
      if (el.getBoundingClientRect().height <= 0) return bad.push({ i, why: 'not-rendered' })
      if (cs(el, 'borderCollapse') !== 'collapse') bad.push({ i, why: 'not-collapsed' })
      // the header must read as a header: divided from the body + de-emphasized
      const th = el.querySelector('th')
      const td = el.querySelector('td')
      if (cs(th, 'borderBottomWidth') === '0px' || clear(cs(th, 'borderBottomColor')))
        bad.push({ i, why: 'header-not-divided', color: cs(th, 'borderBottomColor') })
      if (cs(th, 'color') === cs(td, 'color')) bad.push({ i, why: 'header-not-deemphasized' })
      if (cs(th, 'fontWeight') === cs(td, 'fontWeight')) bad.push({ i, why: 'header-not-weighted' })
      // rows are separated from each other (all but the last)
      const first = el.querySelector('tbody tr:first-child td')
      if (clear(cs(first, 'borderBottomColor')) || cs(first, 'borderBottomWidth') === '0px')
        bad.push({ i, why: 'rows-not-separated' })
      // a numeric column right-aligns on tabular figures
      const num = el.querySelector('td[numeric]')
      if (cs(num, 'textAlign') !== 'right') bad.push({ i, why: 'numeric-not-right' })
      if (!cs(num, 'fontVariantNumeric').includes('tabular-nums'))
        bad.push({ i, why: 'numeric-not-tabular' })
    })

    const scope = "[data-theme-preview='ship-light'] "
    const q = (s) => document.querySelector(scope + s)
    // [striped] actually stripes: an even row differs from an odd one
    const odd = cs(q('table[striped] tbody tr:nth-child(1)'), 'backgroundColor')
    const even = cs(q('table[striped] tbody tr:nth-child(2)'), 'backgroundColor')
    if (odd === even) bad.push({ why: 'striped-not-striped', odd, even })
    // [dense] actually tightens
    const h = (s) => q(s).getBoundingClientRect().height
    if (!(h('table[dense] td') < h('table.l1-mark td')))
      bad.push({ why: 'dense-not-tighter', dense: h('table[dense] td'), base: h('table.l1-mark td') })
    // [bordered] boxes every cell
    if (cs(q('table[bordered] td'), 'borderLeftWidth') === '0px') bad.push({ why: 'bordered-no-box' })
    // frozen-token guard
    const mark = (t) => cs(document.querySelector(`[data-theme-preview='${t}'] .l1-mark th`), 'color')
    if (mark('ship-light') === mark('ship-dark')) bad.push({ why: 'frozen-token', light: mark('ship-light') })

    return { count: document.querySelectorAll('table').length, bad: bad.slice(0, 10) }
  },
}
