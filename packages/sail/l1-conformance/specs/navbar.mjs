// L1 conformance spec: <navbar> (mechanism-1, compiles-away, structural).
// visual.cross: placement · sticky · bordered · autohide · offset.

const bar = (attrs, extra = '') =>
  `<header class="sail-navbar sail-row sail-nowrap sail-ai-center sail-g-4 s-wfull s-bgnavbar s-fgnavbarforeground s-minh14 s-py2 s-px4" data-sail-host="navbar" ${attrs} ${extra}>` +
  `<div class="sail-nav-brand sail-row-i sail-nowrap sail-ai-center sail-g-2 s-fontsemibold s-shrink0"><strong>Acme</strong></div>` +
  `<div class="sail-nav-links s-contents"><a href="#" class="s-nounderline">Home</a><a href="#" class="s-nounderline">Docs</a><a href="#" class="s-nounderline">Pricing</a></div>` +
  `<div class="sail-nav-actions sail-row-i sail-nowrap sail-ai-center sail-g-2 s-shrink0 s-msauto"><button class="sail-page">Sign in</button></div>` +
  `</header>`

export default {
  tag: 'navbar',
  width: 720,
  rows() {
    return [
      bar('class="l1-mark" placement="top"') +
        bar('placement="top" bordered') +
        bar('placement="top" bordered sticky') +
        bar('placement="bottom" bordered sticky') +
        bar('placement="rail" bordered') +
        bar('placement="top" sticky autohide') +
        bar('placement="top" sticky offset="8"'),
    ]
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
      }
      const placement = el.getAttribute('placement') || 'top'
      if (placement !== 'rail') {
        const brand = el.querySelector('.sail-nav-brand')?.getBoundingClientRect()
        const actions = el.querySelector('.sail-nav-actions')?.getBoundingClientRect()
        if (brand && actions && actions.left < brand.right - 1) bad.push({ i, why: 'layout-order' })
      }
    })
    if (!document.querySelector('.sail-navbar[placement="bottom"]')) bad.push({ why: 'bottom-missing' })
    if (!document.querySelector('.sail-navbar[placement="rail"]')) bad.push({ why: 'rail-missing' })
    if (!document.querySelector('.sail-navbar[sticky]')) bad.push({ why: 'sticky-missing' })
    if (!document.querySelector('.sail-navbar[autohide]')) bad.push({ why: 'autohide-missing' })
    if (!document.querySelector('.sail-navbar[offset="8"]')) bad.push({ why: 'offset-missing' })
    return { count: document.querySelectorAll('.sail-navbar').length, bad: bad.slice(0, 10) }
  },
}
