// L1 conformance spec: <theme-picker> (mechanism-1, compiles-away +
// use:themePicker). Renders the lowered radiogroup of the 7 framework themes
// with the current one checked (what use:themePicker maintains). Probe asserts
// the radiogroup contract: role, single aria-checked, roving tabindex, distinct.

const themes = [
  ['ship-light', 'Light'], ['ship-dark', 'Dark'], ['paper', 'Paper'], ['halo', 'Halo'],
  ['console', 'Console'], ['contrast', 'Contrast'], ['glass', 'Glass'],
]

const picker = (value) => {
  const focus = themes.find(([v]) => v === value)?.[0] ?? themes[0][0]
  return (
    `<div class="sui-theme-picker" value="${value}" role="radiogroup">` +
    themes
      .map(
        ([v, l]) =>
          `<button type="button" class="sui-theme-option" value="${v}" role="radio" aria-checked="${v === value ? 'true' : 'false'}" tabindex="${v === focus ? '0' : '-1'}"><span class="sui-theme-swatch"></span>${l}</button>`,
      )
      .join('') +
    `</div>`
  )
}

export default {
  tag: 'theme-picker',
  width: 640,
  rows() {
    return [picker('paper')]
  },
  probe() {
    const bad = []
    document.querySelectorAll('.sui-theme-picker').forEach((el, ti) => {
      if (el.getAttribute('role') !== 'radiogroup') bad.push({ ti, why: 'group-role' })
      const opts = [...el.querySelectorAll('.sui-theme-option')]
      const checked = opts.filter((o) => o.getAttribute('aria-checked') === 'true')
      if (checked.length !== 1) bad.push({ ti, why: 'checked-count', n: checked.length })
      const focusable = opts.filter((o) => o.getAttribute('tabindex') === '0')
      if (focusable.length !== 1) bad.push({ ti, why: 'roving-tabindex', n: focusable.length })
      opts.forEach((o) => {
        if (o.getAttribute('role') !== 'radio') bad.push({ ti, why: 'opt-role' })
        const r = o.getBoundingClientRect()
        if (r.width <= 0 || r.height <= 0) bad.push({ ti, why: 'not-rendered' })
      })
      const on = checked[0]
      const off = opts.find((o) => o.getAttribute('aria-checked') !== 'true')
      if (on && off) {
        const a = getComputedStyle(on)
        const b = getComputedStyle(off)
        if (a.backgroundColor === b.backgroundColor && a.boxShadow === b.boxShadow)
          bad.push({ ti, why: 'checked-not-distinct' })
      }
    })
    return { count: document.querySelectorAll('.sui-theme-picker').length, bad: bad.slice(0, 8) }
  },

  // Live behavior: attach the real use:themePicker and drive it.
  behavior: {
    attach: [['.sui-theme-picker', 'themePicker']],
    async run(page) {
      const bad = []
      const tp = page.locator('.sui-theme-picker').first()
      const opt = (v) => tp.locator(`.sui-theme-option[value="${v}"]`)

      // click a theme → checks it exclusively, syncs value, and applies the
      // theme to <html> (the whole point of the element)
      await opt('console').click()
      if ((await opt('console').getAttribute('aria-checked')) !== 'true')
        bad.push({ why: 'click-did-not-check' })
      if ((await opt('paper').getAttribute('aria-checked')) !== 'false')
        bad.push({ why: 'not-exclusive' })
      if ((await tp.getAttribute('value')) !== 'console') bad.push({ why: 'value-not-synced' })
      const applied = await page.evaluate(() => document.documentElement.getAttribute('data-theme'))
      if (applied !== 'console') bad.push({ why: 'theme-not-applied-to-html', applied })

      // keyboard: ArrowRight moves to the next theme and applies it
      await opt('console').focus()
      await page.keyboard.press('ArrowRight')
      if ((await opt('contrast').getAttribute('aria-checked')) !== 'true')
        bad.push({ why: 'arrow-did-not-advance' })
      const applied2 = await page.evaluate(() => document.documentElement.getAttribute('data-theme'))
      if (applied2 !== 'contrast') bad.push({ why: 'arrow-theme-not-applied', applied: applied2 })

      return bad
    },
  },
}
