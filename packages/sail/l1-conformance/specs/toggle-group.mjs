// L1 conformance spec: <toggle-group> family (mechanism-1, compiles-away +
// use:toggleGroup). Renders the lowered structure across variants / sizes /
// single+multiple with pressed state baked in (what use:toggleGroup maintains).
// Probe asserts the group ARIA, aria-pressed↔value, roving tabindex, and that
// a pressed cell is visually distinct.

const toggle = (val, label, pressed, focusable, dis = '') =>
  `<button class="sail-toggle" value="${val}" role="button" aria-pressed="${pressed ? 'true' : 'false'}" tabindex="${focusable ? '0' : '-1'}" ${dis}>${label}</button>`

const group = (variant, attrs, value, items) => {
  const active = new Set(value.split(/\s+/).filter(Boolean))
  const enabled = items.filter(([, , d]) => !d)
  const focus = enabled.find(([v]) => active.has(v))?.[0] ?? enabled[0]?.[0]
  return (
    `<div class="sail-toggle-group" variant="${variant}" value="${value}" ${attrs} role="group" aria-orientation="horizontal">` +
    items.map(([v, l, d]) => toggle(v, l, active.has(v), v === focus, d ? 'disabled' : '')).join('') +
    `</div>`
  )
}

const items = [['grid', 'Grid'], ['list', 'List'], ['board', 'Board']]

export default {
  tag: 'toggle-group',
  width: 720,
  rows() {
    return [
      group('outline', 'type="single"', 'list', items),
      group('ghost', 'type="single"', 'grid', items),
      group('segmented', 'type="single"', 'board', items),
      group('outline', 'type="multiple"', 'grid board', items),
      group('outline', 'size="sm"', 'list', items) + ' ' + group('outline', 'size="lg"', 'list', items),
    ]
  },
  probe() {
    const bad = []
    document.querySelectorAll('.sail-toggle-group').forEach((el, ti) => {
      if (el.getAttribute('role') !== 'group') bad.push({ ti, why: 'group-role' })
      const toggles = [...el.querySelectorAll('.sail-toggle')]
      const active = new Set((el.getAttribute('value') || '').split(/\s+/).filter(Boolean))
      const single = el.getAttribute('type') !== 'multiple'
      const pressed = toggles.filter((t) => t.getAttribute('aria-pressed') === 'true')
      if (single && pressed.length > 1) bad.push({ ti, why: 'single-over-pressed', n: pressed.length })
      const enabled = toggles.filter((t) => !t.hasAttribute('disabled'))
      const focusable = enabled.filter((t) => t.getAttribute('tabindex') === '0')
      if (enabled.length && focusable.length !== 1) bad.push({ ti, why: 'roving-tabindex', n: focusable.length })
      toggles.forEach((t) => {
        const on = active.has(t.getAttribute('value'))
        if ((t.getAttribute('aria-pressed') === 'true') !== on)
          bad.push({ ti, why: 'pressed-mismatch', v: t.getAttribute('value') })
        if (t.getAttribute('role') !== 'button') bad.push({ ti, why: 'toggle-role' })
        const r = t.getBoundingClientRect()
        if (r.width <= 0 || r.height <= 0) bad.push({ ti, why: 'not-rendered' })
      })
      const on = pressed[0]
      const off = toggles.find((t) => t.getAttribute('aria-pressed') !== 'true' && !t.hasAttribute('disabled'))
      if (on && off) {
        const a = getComputedStyle(on)
        const b = getComputedStyle(off)
        if (a.backgroundColor === b.backgroundColor && a.color === b.color)
          bad.push({ ti, why: 'pressed-not-distinct' })
      }
    })
    return { count: document.querySelectorAll('.sail-toggle-group').length, bad: bad.slice(0, 8) }
  },

  // Live behavior: attach the real use:toggleGroup and drive it.
  behavior: {
    attach: [['.sail-toggle-group', 'toggleGroup']],
    async run(page) {
      const bad = []
      const g = page.locator('.sail-toggle-group').first() // outline, single, value=list
      const cell = (v) => g.locator(`.sail-toggle[value="${v}"]`)

      // single: click selects exclusively + syncs value
      await cell('grid').click()
      if ((await cell('grid').getAttribute('aria-pressed')) !== 'true')
        bad.push({ why: 'click-did-not-press' })
      if ((await cell('list').getAttribute('aria-pressed')) !== 'false')
        bad.push({ why: 'single-not-exclusive' })
      if ((await g.getAttribute('value')) !== 'grid') bad.push({ why: 'value-not-synced' })

      // single is deselectable: clicking the pressed cell clears it
      await cell('grid').click()
      if ((await cell('grid').getAttribute('aria-pressed')) !== 'false')
        bad.push({ why: 'single-not-deselectable' })
      if ((await g.getAttribute('value')) !== '') bad.push({ why: 'value-not-cleared' })

      // keyboard roving: ArrowRight moves focus without pressing
      await cell('grid').focus()
      await page.keyboard.press('ArrowRight')
      const focused = await page.evaluate(() => document.activeElement?.getAttribute('value'))
      if (focused !== 'list') bad.push({ why: 'arrow-right-did-not-move-focus', focused })

      // multiple: clicks accumulate into the value set
      const m = page.locator('.sail-toggle-group[type="multiple"]').first() // value="grid board"
      await m.locator('.sail-toggle[value="list"]').click()
      const v = (await m.getAttribute('value')) ?? ''
      if (!v.includes('list') || !v.includes('grid')) bad.push({ why: 'multi-did-not-accumulate', v })
      await m.locator('.sail-toggle[value="grid"]').click()
      const v2 = (await m.getAttribute('value')) ?? ''
      if (v2.includes('grid')) bad.push({ why: 'multi-did-not-remove', v: v2 })

      return bad
    },
  },
}
