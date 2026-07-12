// L1 conformance spec: <navbar> (mechanism-1, compiles-away, structural).
// Renders the lowered bar (brand / links / actions regions). Probe asserts it
// renders as a horizontal bar with all three regions laid out.

const bar = (attrs) =>
  `<header class="sui-navbar" ${attrs}>` +
  `<div class="sui-nav-brand"><strong>Acme</strong></div>` +
  `<nav class="sui-nav-links"><a href="#">Home</a><a href="#">Docs</a><a href="#">Pricing</a></nav>` +
  `<div class="sui-nav-actions"><button class="sui-page">Sign in</button></div>` +
  `</header>`

export default {
  tag: 'navbar',
  width: 720,
  rows() {
    return [bar('bordered'), bar('bordered sticky')]
  },
  probe() {
    const bad = []
    document.querySelectorAll('.sui-navbar').forEach((el, i) => {
      const cs = getComputedStyle(el)
      const r = el.getBoundingClientRect()
      if (r.width <= 0 || r.height <= 0) bad.push({ i, why: 'not-rendered' })
      if (cs.display !== 'flex') bad.push({ i, why: 'not-flex', display: cs.display })
      for (const region of ['.sui-nav-brand', '.sui-nav-links', '.sui-nav-actions']) {
        const rgn = el.querySelector(region)
        if (!rgn) bad.push({ i, why: 'missing-region', region })
        else if (rgn.getBoundingClientRect().width <= 0) bad.push({ i, why: 'region-collapsed', region })
      }
      // actions must sit to the right of the brand (bar layout intact)
      const brand = el.querySelector('.sui-nav-brand')?.getBoundingClientRect()
      const actions = el.querySelector('.sui-nav-actions')?.getBoundingClientRect()
      if (brand && actions && actions.left < brand.right) bad.push({ i, why: 'layout-order' })
    })
    return { count: document.querySelectorAll('.sui-navbar').length, bad: bad.slice(0, 8) }
  },
}
