// L1 conformance spec: <pagination> family (mechanism-1, compiles-away +
// use:pagination). Renders the lowered nav of .sail-page controls with the
// current page marked (what use:pagination maintains). Probe asserts a single
// aria-current matching value, rendered controls, and a distinct current page.

const page = (v, label, current, extra = '') =>
  `<button class="sail-page" value="${v}" type="button" ${current ? 'aria-current="page"' : ''} ${extra}>${label}</button>`

const pager = (value, attrs = '') => {
  const n = Number(value)
  return (
    `<nav class="sail-pagination" value="${value}" ${attrs} aria-label="Pagination">` +
    page(String(Math.max(1, n - 1)), '‹', false, n <= 1 ? 'disabled' : '') +
    [1, 2, 3, 4, 5].map((p) => page(String(p), String(p), String(p) === value)).join('') +
    page(String(Math.min(5, n + 1)), '›', false, n >= 5 ? 'disabled' : '') +
    `</nav>`
  )
}

export default {
  tag: 'pagination',
  width: 640,
  rows() {
    return [pager('3'), pager('1', 'size="sm"'), pager('5', 'size="lg"')]
  },
  probe() {
    const bad = []
    document.querySelectorAll('.sail-pagination').forEach((el, ti) => {
      const value = el.getAttribute('value')
      const pages = [...el.querySelectorAll('.sail-page')]
      const current = pages.filter((p) => p.getAttribute('aria-current') === 'page')
      if (current.length !== 1) bad.push({ ti, why: 'current-count', n: current.length })
      if (current[0] && current[0].getAttribute('value') !== value)
        bad.push({ ti, why: 'current-mismatch' })
      pages.forEach((p) => {
        const r = p.getBoundingClientRect()
        if (r.width <= 0 || r.height <= 0) bad.push({ ti, why: 'not-rendered' })
      })
      const on = current[0]
      const off = pages.find((p) => p.getAttribute('aria-current') !== 'page' && !p.hasAttribute('disabled'))
      if (on && off) {
        const a = getComputedStyle(on)
        const b = getComputedStyle(off)
        if (a.backgroundColor === b.backgroundColor && a.color === b.color)
          bad.push({ ti, why: 'current-not-distinct' })
      }
    })
    return { count: document.querySelectorAll('.sail-pagination').length, bad: bad.slice(0, 8) }
  },

  // Live behavior: attach the real use:pagination and drive it.
  behavior: {
    attach: [['.sail-pagination', 'pagination']],
    async run(page) {
      const bad = []
      const p = page.locator('.sail-pagination').first() // value="3"
      const pg = (v) => p.locator(`.sail-page[value="${v}"]`)

      // click a page → value syncs, aria-current moves
      await pg('5').click()
      if ((await p.getAttribute('value')) !== '5') bad.push({ why: 'value-not-set' })
      if ((await pg('5').getAttribute('aria-current')) !== 'page')
        bad.push({ why: 'aria-current-not-moved' })
      if ((await pg('3').getAttribute('aria-current')) === 'page')
        bad.push({ why: 'old-current-not-cleared' })

      // disabled control does not navigate (last pager is value=5, next disabled)
      const last = page.locator('.sail-pagination').last() // value="5", next disabled
      const before = await last.getAttribute('value')
      await last.locator('.sail-page[disabled]').first().click({ force: true })
      if ((await last.getAttribute('value')) !== before)
        bad.push({ why: 'disabled-control-navigated' })

      return bad
    },
  },
}
