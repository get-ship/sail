// L1 conformance spec: <theme-picker> (mechanism-1, use:themePicker + use:anchor).
//
// A trigger that opens a native [popover] of theme cards. Each card previews its
// theme through [data-theme-preview], so the swatch is painted by the real
// stylesheet and cannot drift from the theme it advertises.
//
// The rows are the compiler's OWN lowered output (see lowered.mjs), not markup
// typed here. A spec that types its own markup tests the author's idea of the
// element; this one tests the element.

import { lowered } from '../lowered.mjs'

// One instance per row, with unique ids so the popovers don't collide.
const picker = (n) => {
  const html = lowered('theme-picker')
  if (!html) return '<div class="sui-theme-picker">compiler not built</div>'
  return html
    .replace(/sui-root /g, '')
    .replace(/id="sui-theme-panel"/g, `id="tp${n}-panel"`)
    .replace(/popovertarget="sui-theme-panel"/g, `popovertarget="tp${n}-panel"`)
    .replace(/^<div class="sui-theme-picker"/, `<div id="tp${n}" class="sui-theme-picker"`)
}

export default {
  tag: 'theme-picker',
  width: 460,
  rows() {
    // The panel is a popover — it lives in the top layer and won't show in a
    // static snapshot. Render the trigger, plus one panel forced open so the
    // theme previews are captured.
    return [
      picker(1),
      picker(2).replace('popover ', 'popover style="position:static;display:grid;grid-template-columns:1fr 1fr;gap:6px" '),
    ]
  },
  probe() {
    const bad = []
    const cs = (el, p) => (el ? getComputedStyle(el)[p] : null)

    // Every swatch must render its OWN theme. If [data-theme-preview] were inert
    // — as [data-theme] on a <span> is — every card would come out identical,
    // which is exactly what a hand-picked hex gives you.
    const surfaces = [], cards = [], ctas = []
    document.querySelectorAll('.sui-theme-swatch').forEach((sw) => {
      surfaces.push(cs(sw, 'backgroundColor'))
      const c = sw.querySelector('.tc-card')
      const a = sw.querySelector('.tc-cta')
      if (!c) return bad.push({ id: sw.getAttribute('data-theme-preview'), why: 'no-card' })
      cards.push(cs(c, 'backgroundColor'))
      if (a) ctas.push(cs(a, 'backgroundColor'))
    })
    if (new Set(surfaces).size < 3)
      bad.push({ why: 'previews-are-inert', distinct: new Set(surfaces).size })
    if (new Set(cards).size < 3)
      bad.push({ why: 'card-surfaces-identical', distinct: new Set(cards).size })
    if (new Set(ctas).size < 3)
      bad.push({ why: 'accents-identical', distinct: new Set(ctas).size })

    document.querySelectorAll('.sui-theme-trigger').forEach((t) => {
      if (!t.getAttribute('popovertarget')) bad.push({ why: 'trigger-has-no-popovertarget' })
      const label = t.querySelector('.sui-theme-label')
      if (!label || !(label.textContent ?? '').trim()) bad.push({ why: 'trigger-has-no-label' })
    })

    return { count: document.querySelectorAll('.sui-theme-option').length, bad: bad.slice(0, 8) }
  },

  behavior: {
    rows() {
      return [picker('')]
    },
    attach: [
      ['#tp-panel', 'anchor'],
      ['#tp', 'themePicker'],
    ],
    async run(page) {
      const bad = []
      const rootTheme = () => page.evaluate(() => document.documentElement.dataset.theme)
      const rootDark = () => page.evaluate(() => document.documentElement.classList.contains('dark'))
      const panelOpen = () =>
        page.evaluate(() => document.querySelector('.sui-theme-panel')?.matches(':popover-open') ?? false)

      if (await panelOpen()) bad.push({ why: 'panel-open-before-click' })
      await page.locator('.sui-theme-trigger').click()
      if (!(await panelOpen())) bad.push({ why: 'trigger-did-not-open-panel' })

      // Selecting must route through applyTheme: data-theme AND the .dark class
      // AND the panel closing. Setting the attribute by hand does the first and
      // skips the rest, which leaves the page half-switched.
      await page.locator('.sui-theme-option[value="console"]').click()

      const t1 = await rootTheme()
      if (t1 !== 'console') bad.push({ why: 'data-theme-not-applied', got: t1 })
      if (!(await rootDark()))
        bad.push({ why: 'dark-class-not-toggled', note: 'console is dark; applyTheme owns this' })
      if (await panelOpen()) bad.push({ why: 'panel-did-not-close-on-select' })

      const label = await page.locator('.sui-theme-label').innerText()
      if (!/console/i.test(label)) bad.push({ why: 'trigger-label-not-updated', got: label })

      const checked = await page.evaluate(() =>
        document.querySelector('.sui-theme-option[aria-selected="true"]')?.getAttribute('value'),
      )
      if (checked !== 'console') bad.push({ why: 'selection-not-reflected', got: checked })

      // ...and back to a light theme drops .dark again.
      await page.locator('.sui-theme-trigger').click()
      await page.locator('.sui-theme-option[value="paper"]').click()
      const t2 = await rootTheme()
      if (t2 !== 'paper') bad.push({ why: 'second-switch-failed', got: t2 })
      if (await rootDark()) bad.push({ why: 'dark-class-not-cleared-for-light-theme' })

      return bad
    },
  },
}
