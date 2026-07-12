// L1 conformance spec: <theme-picker> (mechanism-1, use:themePicker + use:anchor).
//
// A trigger that opens a native [popover] of theme cards. Each card previews its
// theme through [data-theme-preview], so the swatch is rendered by the real
// stylesheet and cannot drift from the theme it advertises.
//
// The markup below MIRRORS the element's lowered output (stdlib/theme-picker.sui).
// Hand-keeping it is the weakness of this harness — a spec that types its own
// markup cannot catch an element whose template changed underneath it. The
// behavior block is what has teeth: it drives the real action.

const THEMES = [
  ['ship-light', 'SHIP Light'],
  ['ship-dark', 'SHIP Dark'],
  ['paper', 'Paper'],
  ['halo', 'Halo'],
  ['console', 'Console'],
  ['contrast', 'High Contrast'],
  ['glass', 'Glass'],
  ['neutral-light', 'Neutral Light'],
  ['neutral-dark', 'Neutral Dark'],
]

const PALETTE =
  '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true"><circle cx="13.5" cy="6.5" r="1.5"></circle><circle cx="17.5" cy="10.5" r="1.5"></circle><circle cx="8.5" cy="7.5" r="1.5"></circle><circle cx="6.5" cy="12.5" r="1.5"></circle><path d="M12 2a10 10 0 0 0 0 20 2 2 0 0 0 2-2v-1a2 2 0 0 1 2-2h1a4 4 0 0 0 4-4 10 10 0 0 0-9-11Z"></path></svg>'

const option = ([v, l], selected) =>
  `<button class="sui-theme-option" type="button" role="option" value="${v}"` +
  ` aria-selected="${v === selected ? 'true' : 'false'}" tabindex="${v === selected ? '0' : '-1'}">` +
  `<span class="sui-theme-swatch" data-theme-preview="${v}">` +
  `<i class="sw-surface"></i><i class="sw-accent"></i><i class="sw-muted"></i></span>${l}</button>`

const picker = (id, selected) =>
  `<div class="sui-theme-picker" id="${id}" value="${selected}">` +
  `<button class="sui-theme-trigger" type="button" popovertarget="${id}-panel" aria-label="Choose theme">` +
  `${PALETTE}<span class="sui-theme-label">${THEMES.find(([v]) => v === selected)?.[1] ?? 'Theme'}</span>` +
  `<svg class="sui-theme-caret" viewBox="0 0 10 6" aria-hidden="true"><path d="M1 1l4 4 4-4" fill="none" stroke="currentColor" stroke-width="1.5"></path></svg>` +
  `</button>` +
  `<div class="sui-theme-panel" id="${id}-panel" popover role="listbox" aria-label="Theme">` +
  THEMES.map((t) => option(t, selected)).join('') +
  `</div></div>`

export default {
  tag: 'theme-picker',
  width: 420,
  rows() {
    // The panel is a popover: it lives in the top layer and will not appear in a
    // static snapshot. Show the trigger (the whole visible surface), plus the
    // card grid on its own so the theme previews are captured.
    return [
      picker('tp-a', 'ship-light'),
      picker('tp-b', 'console'),
      `<div class="sui-theme-picker" style="display:grid;grid-template-columns:1fr 1fr;gap:6px;width:280px">` +
        THEMES.map((t) => option(t, 'paper')).join('') +
        `</div>`,
    ]
  },
  probe() {
    const bad = []
    const cs = (el, p) => (el ? getComputedStyle(el)[p] : null)

    // Every swatch must render its OWN theme, not the ambient one. If
    // [data-theme-preview] were inert — as [data-theme] on a <span> is — every
    // swatch would come out identical, which is what a hand-picked hex gives you.
    const surfaces = []
    document.querySelectorAll('.sui-theme-swatch').forEach((sw) => {
      const id = sw.getAttribute('data-theme-preview')
      const bg = cs(sw.querySelector('.sw-surface'), 'backgroundColor')
      if (!bg) return bad.push({ id, why: 'no-surface' })
      surfaces.push(bg)
    })
    const distinct = new Set(surfaces)
    if (distinct.size < 3)
      bad.push({
        why: 'previews-are-inert',
        distinct: distinct.size,
        note: 'every swatch renders the same color — data-theme-preview is not applying',
      })

    document.querySelectorAll('.sui-theme-trigger').forEach((t) => {
      if (!t.getAttribute('popovertarget')) bad.push({ why: 'trigger-has-no-popovertarget' })
      const label = t.querySelector('.sui-theme-label')
      if (!label || !(label.textContent ?? '').trim()) bad.push({ why: 'trigger-has-no-label' })
    })

    if (!document.querySelectorAll('.sui-theme-option[aria-selected="true"]').length)
      bad.push({ why: 'nothing-selected' })

    return { count: document.querySelectorAll('.sui-theme-option').length, bad: bad.slice(0, 8) }
  },

  behavior: {
    rows() {
      return [picker('tp', 'ship-light')]
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
