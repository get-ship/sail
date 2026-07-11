// L1 conformance spec: <combobox> (native [popover] + use:anchor + use:combobox).
// The one genuinely composite input: the platform ALMOST has it (`<input list>` +
// `<datalist>` is native typeahead, zero JS) but the UA draws that popup and it
// cannot be styled or themed — that gap is the whole reason this is a primitive.
//
// RENDER: the field + listbox surface + the active/disabled option states, forced
// in-flow across 7 themes (a real popover is top-layer, so it can't sit in a grid).
// BEHAVIOR: the real thing — type to filter, arrows rove, Enter commits, Escape
// closes, click commits, and the ARIA the pattern demands actually gets wired.

const surface = () =>
  `<div class="sui-combobox" style="width:260px">` +
  `<input class="input" type="text" role="combobox" aria-expanded="true" value="an">` +
  `<div class="listbox" role="listbox" style="position:static;min-width:0">` +
  `<option value="banana" data-active aria-selected="true">Banana</option>` +
  `<option value="mango">Mango</option>` +
  `<option value="melon" disabled>Melon</option>` +
  `</div></div>`

export default {
  tag: 'combobox',
  width: 700,
  rows() {
    return [surface()]
  },
  probe() {
    const bad = []
    const cs = (el, p) => getComputedStyle(el)[p]
    const clear = (v) => v === 'rgba(0, 0, 0, 0)' || v === 'transparent' || v === ''

    document.querySelectorAll('.sui-combobox').forEach((el, i) => {
      const input = el.querySelector('.input')
      const list = el.querySelector('.listbox')
      // the field is a field — it obeys the same visibility contract as <input>
      if (clear(cs(input, 'borderTopColor')) && cs(input, 'boxShadow') === 'none')
        bad.push({ i, why: 'field-not-visible' })
      // the listbox is a floating surface: elevated + its own background, or it
      // reads as part of the page rather than on top of it
      if (cs(list, 'boxShadow') === 'none') bad.push({ i, why: 'listbox-no-elevation' })
      if (clear(cs(list, 'backgroundColor'))) bad.push({ i, why: 'listbox-transparent' })

      const opts = [...list.querySelectorAll('option')]
      if (opts.length !== 3) return bad.push({ i, why: 'options-missing', n: opts.length })
      opts.forEach((o) => {
        const r = o.getBoundingClientRect()
        // <option> is display:none in a UA listbox — the skin MUST re-show it,
        // which is exactly the thing a styled combobox exists to do
        if (r.width <= 0 || r.height <= 0) bad.push({ i, why: 'option-not-rendered' })
      })
      // the ACTIVE option is the keyboard cursor. Focus never leaves the input in
      // this pattern, so if the highlight isn't drawn there is no cursor at all.
      const [active, plain, disabled] = opts
      if (cs(active, 'backgroundColor') === cs(plain, 'backgroundColor'))
        bad.push({ i, why: 'active-not-highlighted' })
      if (cs(disabled, 'opacity') === cs(plain, 'opacity'))
        bad.push({ i, why: 'disabled-not-distinct' })
    })

    // frozen-token guard — on the listbox surface, which is the themed part
    const surf = (t) =>
      cs(document.querySelector(`[data-theme-preview='${t}'] .listbox`), 'backgroundColor')
    if (surf('ship-light') === surf('ship-dark'))
      bad.push({ why: 'frozen-token', light: surf('ship-light') })

    return { count: document.querySelectorAll('.sui-combobox').length, bad: bad.slice(0, 10) }
  },
  behavior: {
    rows() {
      return [
        `<div class="sui-combobox" id="cb" placeholder="Pick a fruit" style="width:260px">` +
          `<input class="input" type="text">` +
          `<div class="listbox" popover placement="bottom-start">` +
          `<option value="apple">Apple</option>` +
          `<option value="banana">Banana</option>` +
          `<option value="mango">Mango</option>` +
          `<option value="melon" disabled>Melon</option>` +
          `</div></div>`,
      ]
    },
    attach: [
      ['#cb .listbox', 'anchor'],
      ['#cb', 'combobox'],
    ],
    async run(page) {
      const bad = []
      const input = page.locator('#cb .input')
      const list = page.locator('#cb .listbox')
      const activeVal = () =>
        page.evaluate(() => document.querySelector('#cb option[data-active]')?.getAttribute('value') ?? null)

      await page.evaluate(() => {
        globalThis.__picked = null
        document
          .getElementById('cb')
          .addEventListener('sui-combobox-select', (e) => (globalThis.__picked = e.detail.value))
      })

      // the root's `placeholder` reaches the inner field (element mode is static
      // structure — the action forwards it)
      if ((await input.getAttribute('placeholder')) !== 'Pick a fruit')
        bad.push({ why: 'placeholder-not-forwarded' })

      // ARIA the pattern demands, wired by the action and not by the author
      const aria = await page.evaluate(() => {
        const i = document.querySelector('#cb .input')
        const l = document.querySelector('#cb .listbox')
        return {
          role: i.getAttribute('role'),
          controls: i.getAttribute('aria-controls'),
          listId: l.id,
          listRole: l.getAttribute('role'),
          expanded: i.getAttribute('aria-expanded'),
        }
      })
      if (aria.role !== 'combobox') bad.push({ why: 'no-combobox-role', got: aria.role })
      if (aria.listRole !== 'listbox') bad.push({ why: 'no-listbox-role' })
      if (!aria.controls || aria.controls !== aria.listId)
        bad.push({ why: 'aria-controls-not-wired', ...aria })
      if (aria.expanded !== 'false') bad.push({ why: 'expanded-not-false-at-rest' })

      // TYPE → filters + opens
      await input.click()
      await input.type('an')
      await page.waitForTimeout(120)
      if (!(await list.isVisible())) bad.push({ why: 'typing-did-not-open' })
      const visible = await page.evaluate(() =>
        [...document.querySelectorAll('#cb option')].filter((o) => !o.hidden).map((o) => o.getAttribute('value')),
      )
      // "an" matches banana + mango, not apple or melon
      if (JSON.stringify(visible) !== JSON.stringify(['banana', 'mango']))
        bad.push({ why: 'filter-wrong', visible })
      if ((await input.getAttribute('aria-expanded')) !== 'true')
        bad.push({ why: 'expanded-not-synced' })

      // the first MATCH is the active cursor, and aria-activedescendant points at it
      if ((await activeVal()) !== 'banana') bad.push({ why: 'first-match-not-active', got: await activeVal() })
      const ad = await page.evaluate(() => {
        const i = document.querySelector('#cb .input')
        return i.getAttribute('aria-activedescendant') === document.querySelector('#cb option[data-active]')?.id
      })
      if (!ad) bad.push({ why: 'activedescendant-not-tracking' })

      // ARROWS rove within the FILTERED set — skipping hidden options is the point
      await page.keyboard.press('ArrowDown')
      if ((await activeVal()) !== 'mango') bad.push({ why: 'arrow-did-not-rove', got: await activeVal() })

      // ENTER commits: emits, closes, and writes the label back into the field
      await page.keyboard.press('Enter')
      await page.waitForTimeout(80)
      if ((await page.evaluate(() => globalThis.__picked)) !== 'mango')
        bad.push({ why: 'enter-did-not-commit' })
      if (await list.isVisible()) bad.push({ why: 'commit-did-not-close' })
      if ((await input.inputValue()) !== 'Mango') bad.push({ why: 'label-not-written-back' })
      if ((await page.getAttribute('#cb', 'value')) !== 'mango') bad.push({ why: 'value-not-reflected' })

      // ESCAPE closes without committing
      await input.click()
      await page.keyboard.press('ArrowDown')
      await page.waitForTimeout(100)
      if (!(await list.isVisible())) bad.push({ why: 'arrow-did-not-open' })
      await page.keyboard.press('Escape')
      await page.waitForTimeout(80)
      if (await list.isVisible()) bad.push({ why: 'escape-did-not-close' })

      // CLICK commits — and a disabled option does not
      await page.evaluate(() => (globalThis.__picked = null))
      await input.click()
      await page.keyboard.press('ArrowDown')
      await page.waitForTimeout(120)
      await page.click('#cb option[value=melon]', { force: true })
      await page.waitForTimeout(80)
      if ((await page.evaluate(() => globalThis.__picked)) !== null)
        bad.push({ why: 'disabled-option-committed' })
      await page.click('#cb option[value=apple]', { force: true })
      await page.waitForTimeout(80)
      if ((await page.evaluate(() => globalThis.__picked)) !== 'apple')
        bad.push({ why: 'click-did-not-commit' })

      return bad
    },
  },
}
