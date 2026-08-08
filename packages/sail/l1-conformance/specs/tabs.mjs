// L1 conformance spec: <tabs> family (mechanism-1, compiles-away + use:tabs).
// Renders the lowered structure (<div class="sail-tabs"> → tab-list of .sail-tab
// buttons + .sail-tab-panel panels) across variants / sizes / orientation, with
// the active tab + shown panel baked in (the static shape use:tabs maintains).
// The probe asserts the tablist contract: exactly one selected tab, roving
// tabindex, roles, only the active panel renders, and component-token paint
// (tab* / tab-list*) on idle vs selected chrome.

// Lowered Style stamps — component.tab + component.tab-list packs (not free globals).
const TAB =
  'sail-tab s-padY2 s-padX3 s-fgtabforeground s-bgtransparent s-cursorpointer s-selectnone s-borderWidth0 s-borderBWidth2 s-borderBtransparent s-radiusTsm s-tapHighlightnone s-transitioncolors s-durationfast s-easeout s-fontFamilybody s-fontWeightmedium s-leadingnone s-whitespacenowrap s-fontSizesm s-hover-fgtabselectedforeground s-selected-fgtabselectedforeground s-selected-borderBprimary s-selected-fontWeightsemibold s-disabled-opacity50 s-disabled-cursornotallowed'
const TAB_PILL =
  `${TAB} s-variantpill-borderBWidth0 s-variantpill-radiussm s-variantpillselected-fgtabselectedforeground s-variantpillselected-bgtabselectedbg s-variantpillselected-shadowxs s-variantpillselected-borderBtransparent`
const TAB_ENCLOSED =
  `${TAB} s-variantenclosed-borderWidth1 s-variantenclosed-bordertransparent s-variantenclosed-borderBWidth0 s-variantenclosed-radiusTmd s-variantenclosedselected-fgtabselectedforeground s-variantenclosedselected-bordertabborder s-variantenclosedselected-bgtabselectedbg s-variantenclosedselected-translateYpx`
const LIST =
  'sail-tab-list s-borderBWidth1 s-bordertablistborder s-bgtablist'
const LIST_LINE = `${LIST} s-variantline-borderBWidth1 s-variantline-bordertablistborder`
const LIST_PILL =
  `${LIST} s-variantpill-pad1 s-variantpill-borderWidth0 s-variantpill-radiusmd s-variantpill-bgtablistrail s-variantpill-borderBWidth0`
const LIST_ENCLOSED = `${LIST} s-variantenclosed-borderBWidth1 s-variantenclosed-bordertablistborder`

const tabCls = (variant) =>
  variant === 'pill' ? TAB_PILL : variant === 'enclosed' ? TAB_ENCLOSED : TAB
const listCls = (variant) =>
  variant === 'pill' ? LIST_PILL : variant === 'enclosed' ? LIST_ENCLOSED : LIST_LINE

const tab = (name, label, sel, variant, extra = '') =>
  `<button class="${tabCls(variant)}" name="${name}" role="tab" aria-selected="${sel ? 'true' : 'false'}" tabindex="${sel ? '0' : '-1'}" ${extra}>${label}</button>`
const panel = (name, sel) =>
  `<div class="sail-tab-panel s-minW0 s-orientationhorizontal-padT4" name="${name}" role="tabpanel" ${sel ? '' : 'hidden'}>${name} panel</div>`

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
      tabs('line', 'class="l1-mark"', 'overview', set),
      tabs('pill', '', 'activity', set),
      tabs('enclosed', '', 'settings', set),
      tabs('line', '', 'overview', set.slice(0, 3).map(([n, l, x]) => [n, l, (x || '') + ' size="sm"'])) +
        tabs('line', '', 'activity', set.slice(0, 3).map(([n, l, x]) => [n, l, (x || '') + ' size="lg"'])),
      tabs('line', 'orientation="vertical"', 'overview', set.slice(0, 3)),
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
