// L1 conformance spec: <tabs> family (mechanism-1, compiles-away + use:tabs).
// Renders the lowered structure (<div class="sail-tabs"> → tab-list of .sail-tab
// buttons + .sail-tab-panel panels) across variants / sizes / orientation, with
// the active tab + shown panel baked in (the static shape use:tabs maintains).
// The probe asserts the tablist contract: exactly one selected tab, roving
// tabindex, roles, only the active panel renders, and component-token paint
// (tab* / tab-list*) on idle vs selected chrome.

// Lowered Style stamps — component.tab + component.tab-list packs (not free globals).
const TAB =
  'sail-tab sx-padY2 sx-padX3 sx-fgtabforeground sx-bgtransparent sx-cursorpointer sx-selectnone sx-borderWidth0 sx-borderBWidth2 sx-borderBtransparent sx-radiusTsm sx-tapHighlightnone sx-transitioncolors sx-durationfast sx-easeout sx-fontFamilybody sx-fontWeightmedium sx-leadingnone sx-whitespacenowrap sx-fontSizesm sx-hover-fgtabselectedforeground sx-selected-fgtabselectedforeground sx-selected-borderBprimary sx-selected-fontWeightsemibold sx-disabled-opacity50 sx-disabled-cursornotallowed'
const TAB_PILL =
  `${TAB} sx-variantpill-borderBWidth0 sx-variantpill-radiussm sx-variantpillselected-fgtabselectedforeground sx-variantpillselected-bgtabselectedbg sx-variantpillselected-shadowxs sx-variantpillselected-borderBtransparent`
const TAB_ENCLOSED =
  `${TAB} sx-variantenclosed-borderWidth1 sx-variantenclosed-bordertransparent sx-variantenclosed-borderBWidth0 sx-variantenclosed-radiusTmd sx-variantenclosedselected-fgtabselectedforeground sx-variantenclosedselected-bordertabborder sx-variantenclosedselected-bgtabselectedbg sx-variantenclosedselected-translateYpx`
const LIST =
  'sail-tab-list sx-borderBWidth1 sx-bordertablistborder sx-bgtablist'
const LIST_LINE = `${LIST} sx-variantline-borderBWidth1 sx-variantline-bordertablistborder`
const LIST_PILL =
  `${LIST} sx-variantpill-pad1 sx-variantpill-borderWidth0 sx-variantpill-radiusmd sx-variantpill-bgtablistrail sx-variantpill-borderBWidth0`
const LIST_ENCLOSED = `${LIST} sx-variantenclosed-borderBWidth1 sx-variantenclosed-bordertablistborder`

const tabCls = (variant) =>
  variant === 'pill' ? TAB_PILL : variant === 'enclosed' ? TAB_ENCLOSED : TAB
const listCls = (variant) =>
  variant === 'pill' ? LIST_PILL : variant === 'enclosed' ? LIST_ENCLOSED : LIST_LINE

const tab = (name, label, sel, variant, extra = '') =>
  `<button class="${tabCls(variant)}" name="${name}" role="tab" aria-selected="${sel ? 'true' : 'false'}" tabindex="${sel ? '0' : '-1'}" ${extra}>${label}</button>`
const panel = (name, sel) =>
  `<div class="sail-tab-panel sx-minW0 sx-orientationhorizontal-padT4" name="${name}" role="tabpanel" ${sel ? '' : 'hidden'}>${name} panel</div>`

const tabs = (variant, attrs, active, items) =>
  `<div class="sail-tabs" active="${active}" ${attrs}>` +
  `<div class="${listCls(variant)}" role="tablist" variant="${variant}">` +
  items.map(([n, l, x]) => tab(n, l, n === active, variant, x || '')).join('') +
  `</div>` +
  items.map(([n]) => panel(n, n === active)).join('') +
  `</div>`

const set = [
  ['overview', 'Overview'],
  ['activity', 'Activity'],
  ['settings', 'Settings'],
  ['off', 'Disabled', 'disabled'],
]

export default {
  tag: 'tabs',
  width: 760,
  rows() {
    return [
      tabs('line', 'tone="primary" class="l1-mark"', 'overview', set),
      tabs('pill', 'tone="primary"', 'activity', set),
      tabs('enclosed', 'tone="primary"', 'settings', set),
      tabs('line', 'size="sm"', 'overview', set.slice(0, 3)) +
        tabs('line', 'size="lg"', 'activity', set.slice(0, 3)),
      tabs('line', 'orientation="vertical" tone="info"', 'overview', set.slice(0, 3)),
    ]
  },
  probe() {
    const bad = []
    const clear = (v) => v === 'rgba(0, 0, 0, 0)' || v === 'transparent' || v === ''
    document.querySelectorAll('.sail-tabs').forEach((el, ti) => {
      const tabs = [...el.querySelectorAll('.sail-tab')]
      const panels = [...el.querySelectorAll('.sail-tab-panel')]
      const list = el.querySelector('.sail-tab-list')
      const selected = tabs.filter((t) => t.getAttribute('aria-selected') === 'true')
      if (selected.length !== 1) bad.push({ ti, why: 'selected-count', n: selected.length })
      tabs.forEach((t) => {
        const sel = t.getAttribute('aria-selected') === 'true'
        if (t.getAttribute('role') !== 'tab') bad.push({ ti, why: 'role' })
        if (t.getAttribute('tabindex') !== (sel ? '0' : '-1')) bad.push({ ti, why: 'roving-tabindex' })
        const r = t.getBoundingClientRect()
        if (r.width <= 0 || r.height <= 0) bad.push({ ti, why: 'tab-not-rendered' })
        // Idle tabs paint via tab-foreground (token); color must resolve.
        if (clear(getComputedStyle(t).color)) bad.push({ ti, why: 'no-token-fg', name: t.getAttribute('name') })
      })
      const activeName = selected[0]?.getAttribute('name')
      panels.forEach((p) => {
        const shouldShow = p.getAttribute('name') === activeName
        const shown = getComputedStyle(p).display !== 'none'
        if (shown !== shouldShow)
          bad.push({ ti, why: 'panel-visibility', name: p.getAttribute('name'), shown, shouldShow })
      })
      // Active tab must be visually distinct from an inactive one (underline /
      // fill / border — some paint property differs).
      const active = selected[0]
      const inactive = tabs.find((t) => t.getAttribute('aria-selected') !== 'true' && !t.hasAttribute('disabled'))
      if (active && inactive) {
        const a = getComputedStyle(active)
        const b = getComputedStyle(inactive)
        const same =
          a.color === b.color &&
          a.borderBottomColor === b.borderBottomColor &&
          a.backgroundColor === b.backgroundColor
        if (same) bad.push({ ti, why: 'active-not-distinct' })
      }
      // tab-list strip border channel (tab-list-border) on line/enclosed.
      if (list && list.getAttribute('variant') !== 'pill') {
        const bw = getComputedStyle(list).borderBottomWidth
        if (bw === '0px' && list.getAttribute('orientation') !== 'vertical')
          bad.push({ ti, why: 'list-no-token-border' })
      }
    })
    return { count: document.querySelectorAll('.sail-tabs').length, bad: bad.slice(0, 8) }
  },

  // Live behavior: attach the real use:tabs and drive it.
  behavior: {
    attach: [['.sail-tabs', 'tabs']],
    async run(page) {
      const bad = []
      const t = page.locator('.sail-tabs').first()
      const tab = (n) => t.locator(`.sail-tab[name="${n}"]`)
      const panel = (n) => t.locator(`.sail-tab-panel[name="${n}"]`)

      // click a tab → it selects, the old one deselects, panels swap
      await tab('activity').click()
      if ((await tab('activity').getAttribute('aria-selected')) !== 'true')
        bad.push({ why: 'click-did-not-select' })
      if ((await tab('overview').getAttribute('aria-selected')) !== 'false')
        bad.push({ why: 'previous-still-selected' })
      if (!(await panel('activity').isVisible())) bad.push({ why: 'active-panel-hidden' })
      if (await panel('overview').isVisible()) bad.push({ why: 'inactive-panel-visible' })
      if ((await t.getAttribute('active')) !== 'activity') bad.push({ why: 'active-attr-not-synced' })

      // roving tabindex followed the selection
      if ((await tab('activity').getAttribute('tabindex')) !== '0')
        bad.push({ why: 'roving-tabindex-not-moved' })

      // keyboard: ArrowRight advances, Home returns to first
      await tab('activity').focus()
      await page.keyboard.press('ArrowRight')
      if ((await tab('settings').getAttribute('aria-selected')) !== 'true')
        bad.push({ why: 'arrow-right-did-not-advance' })
      await page.keyboard.press('Home')
      if ((await tab('overview').getAttribute('aria-selected')) !== 'true')
        bad.push({ why: 'home-did-not-select-first' })

      // disabled tab is not activated by click
      await tab('off').click({ force: true })
      if ((await tab('off').getAttribute('aria-selected')) === 'true')
        bad.push({ why: 'disabled-tab-activated' })

      return bad
    },
  },
}
