// L1 conformance spec: <breadcrumb> (mechanism-1, compiles-away, structural).
// Renders the lowered nav>ol>li>a trail. Probe asserts it renders, marks the
// current page, and draws a separator before every non-first crumb.

const crumb = (label, href, current) =>
  `<li>${current ? `<a aria-current="page">${label}</a>` : `<a href="${href}">${label}</a>`}</li>`

const trail = () =>
  `<nav class="sail-breadcrumb" aria-label="Breadcrumb"><ol>` +
  crumb('Home', '#', false) +
  crumb('Components', '#', false) +
  crumb('Tabs', '', true) +
  `</ol></nav>`

export default {
  tag: 'breadcrumb',
  width: 560,
  rows() {
    return [trail()]
  },
  probe() {
    const bad = []
    document.querySelectorAll('.sail-breadcrumb').forEach((el, i) => {
      const items = [...el.querySelectorAll('li')]
      if (items.length < 2) bad.push({ i, why: 'too-few-crumbs', n: items.length })
      const current = el.querySelectorAll('[aria-current="page"]')
      if (current.length !== 1) bad.push({ i, why: 'current-count', n: current.length })
      items.forEach((li, j) => {
        const r = li.getBoundingClientRect()
        if (r.width <= 0 || r.height <= 0) bad.push({ i, why: 'not-rendered' })
        // every non-first crumb draws a separator via ::before
        if (j > 0) {
          const before = getComputedStyle(li, '::before').content
          if (!before || before === 'none' || before === 'normal')
            bad.push({ i, why: 'missing-separator', crumb: j })
        }
      })
      // Token-backed trail paint: links + current must resolve (not transparent).
      const link = el.querySelector('a:not([aria-current])')
      if (link) {
        const cs = getComputedStyle(link)
        if (!cs.color || cs.color === 'rgba(0, 0, 0, 0)') bad.push({ i, why: 'no-token-link-fg' })
      }
      const cur = el.querySelector('[aria-current="page"]')
      if (cur) {
        const cs = getComputedStyle(cur)
        if (!cs.color || cs.color === 'rgba(0, 0, 0, 0)') bad.push({ i, why: 'no-token-current-fg' })
      }
    })
    return { count: document.querySelectorAll('.sail-breadcrumb').length, bad: bad.slice(0, 8) }
  },
}
