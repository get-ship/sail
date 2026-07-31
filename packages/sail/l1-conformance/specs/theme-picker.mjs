// L1 conformance spec: <theme-picker> (mechanism-1, use:themeSelect + use:anchor).
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
  if (!html) return '<div class="sail-theme-picker">compiler not built</div>'
  return html
    .replace(/sail-root /g, '')
    .replace(/id="sail-theme-panel"/g, `id="tp${n}-panel"`)
    .replace(/popovertarget="sail-theme-panel"/g, `popovertarget="tp${n}-panel"`)
    .replace(/^<div class="sail-theme-picker"/, `<div id="tp${n}" class="sail-theme-picker"`)
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
    document.querySelectorAll('.sail-theme-swatch').forEach((sw) => {
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

    document.querySelectorAll('.sail-theme-trigger').forEach((t) => {
      if (!t.getAttribute('popovertarget')) bad.push({ why: 'trigger-has-no-popovertarget' })
      // Icon-only trigger: the accessible name is the label.
      if (!(t.getAttribute('aria-label') ?? '').trim()) bad.push({ why: 'trigger-has-no-accessible-name' })
    })

    return { count: document.querySelectorAll('.sail-theme-option').length, bad: bad.slice(0, 8) }
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
      const rootMode = () => page.evaluate(() => document.documentElement.dataset.mode || '')
      const panelOpen = () =>
        page.evaluate(() => document.querySelector('.sail-theme-panel')?.matches(':popover-open') ?? false)

      if (await panelOpen()) bad.push({ why: 'panel-open-before-click' })
      await page.locator('.sail-theme-trigger').click()
      if (!(await panelOpen())) bad.push({ why: 'trigger-did-not-open-panel' })

      // Selecting must route through applyTheme: data-theme + data-mode
      // AND the panel closing.
      await page.locator('.sail-theme-option[value="console"]').click()

      const t1 = await rootTheme()
      if (t1 !== 'console') bad.push({ why: 'data-theme-not-applied', got: t1 })
      const m1 = await rootMode()
      if (m1 !== 'dark')
        bad.push({ why: 'data-mode-not-dark', got: m1, note: 'console defaultMode is dark' })
      if (await panelOpen()) bad.push({ why: 'panel-did-not-close-on-select' })

      // The icon trigger's accessible name must track the selection.
      const name = await page.evaluate(
        () => document.querySelector('.sail-theme-trigger')?.getAttribute('aria-label') ?? '',
      )
      if (!/console/i.test(name)) bad.push({ why: 'trigger-name-not-updated', got: name })

      const checked = await page.evaluate(() =>
        document.querySelector('.sail-theme-option[aria-selected="true"]')?.getAttribute('value'),
      )
      if (checked !== 'console') bad.push({ why: 'selection-not-reflected', got: checked })

      // ...and back to a light theme sets data-mode light.
      await page.locator('.sail-theme-trigger').click()
      await page.locator('.sail-theme-option[value="paper"]').click()
      const t2 = await rootTheme()
      if (t2 !== 'paper') bad.push({ why: 'second-switch-failed', got: t2 })
      const m2 = await rootMode()
      if (m2 !== 'light') bad.push({ why: 'data-mode-not-light', got: m2 })

      return bad
    },
  },
}
