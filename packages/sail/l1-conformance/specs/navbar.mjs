// L1 conformance spec: <navbar> (mechanism-1, compiles-away, structural).
// Renders the lowered bar (brand / links / actions regions). Probe asserts it
// renders as a horizontal bar with all three regions laid out.

const bar = (attrs) =>
  `<header class="sail-navbar" ${attrs}>` +
  `<div class="sail-nav-brand"><strong>Acme</strong></div>` +
  `<nav class="sail-nav-links"><a href="#">Home</a><a href="#">Docs</a><a href="#">Pricing</a></nav>` +
  `<div class="sail-nav-actions"><button class="sail-page">Sign in</button></div>` +
  `</header>`

export default {
  tag: 'navbar',
  width: 720,
  rows() {
    return [bar('bordered'), bar('bordered sticky')]
  },
  probe() {
    const bad = []
    document.querySelectorAll('.sail-navbar').forEach((el, i) => {
      const cs = getComputedStyle(el)
      const r = el.getBoundingClientRect()
      if (r.width <= 0 || r.height <= 0) bad.push({ i, why: 'not-rendered' })
      if (cs.display !== 'flex') bad.push({ i, why: 'not-flex', display: cs.display })
      for (const region of ['.sail-nav-brand', '.sail-nav-links', '.sail-nav-actions']) {
        const rgn = el.querySelector(region)
        if (!rgn) bad.push({ i, why: 'missing-region', region })
        else if (rgn.getBoundingClientRect().width <= 0) bad.push({ i, why: 'region-collapsed', region })
      }
      // actions must sit to the right of the brand (bar layout intact)
      const brand = el.querySelector('.sail-nav-brand')?.getBoundingClientRect()
      const actions = el.querySelector('.sail-nav-actions')?.getBoundingClientRect()
      if (brand && actions && actions.left < brand.right) bad.push({ i, why: 'layout-order' })
    })
    return { count: document.querySelectorAll('.sail-navbar').length, bad: bad.slice(0, 8) }
  },
}
