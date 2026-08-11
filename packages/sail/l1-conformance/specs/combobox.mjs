// L1 conformance spec: <combobox> (native [popover] + use:anchor + use:combobox).
// visual.cross: placeholder · value.
// RENDER: field + listbox with lowered Style stamps, forced in-flow.
// BEHAVIOR: type to filter, arrows rove, Enter commits, Escape closes.
//
// Note: --combobox aliases --popover on :root; section theme overrides --popover
// but inherited --combobox can stay resolved. List paint uses var(--popover)
// directly so light vs dark themes diverge (frozen-token guard).

// Lowered Style stamps — host / input / listbox packs from stdlib combobox.sail.
const HOST = 'sail-combobox s-displayblock s-positionrelative s-wfull s-fontFamilybody'
const INPUT =
  'input s-displayblock s-wfull s-minHcontrolhmd s-padY15 s-padX3 s-fgforeground s-bgbackground s-borderWidth1 s-borderborder s-radiusmd s-shadowxs s-fontSizebase'
// List paint: component.combobox.* maps to --combobox/--combobox-foreground which
// alias popover; for harness theme sections use popover vars so dark themes paint.
const LIST =
  'listbox s-borderWidth1 s-borderborder s-radiusmd s-shadowlg s-pad1 s-zpopover s-overflowYauto s-maxHpanelmaxhsm'
const LIST_PAINT =
  'background:var(--popover);color:var(--popover-foreground);border-style:solid'

const surface = (attrs = '', mark = false) =>
  `<div class="${HOST}${mark ? ' l1-mark' : ''}" data-sail-host="combobox" ${attrs} style="width:260px">` +
  `<input class="${INPUT}" type="text" role="combobox" aria-expanded="true" value="an" style="border-style:solid">` +
  // popover so sail-base option anatomy applies; position:static forces in-flow snapshot
  `<div class="${LIST}" role="listbox" popover style="position:static;display:block;min-width:0;${LIST_PAINT}">` +
  `<option value="banana" data-active aria-selected="true">Banana</option>` +
  `<option value="mango">Mango</option>` +
  `<option value="melon" disabled>Melon</option>` +
  `</div></div>`

export default {
  tag: 'combobox',
  width: 700,
  rows() {
    return [
      // default/rest
      surface('', true),
      // placeholder · value axes
      surface('placeholder="Pick a fruit" value="banana"'),
      surface('placeholder="Search packages" value="mango"'),
    ]
  },
  probe() {
    const bad = []
    const cs = (el, p) => getComputedStyle(el)[p]
    const clear = (v) => v === 'rgba(0, 0, 0, 0)' || v === 'transparent' || v === ''

    document.querySelectorAll('.sail-combobox').forEach((el, i) => {
      const input = el.querySelector('input')
      const list = el.querySelector('[role="listbox"]')
      if (!input || !list) return bad.push({ i, why: 'structure-missing' })
      // the field is a field — it obeys the same visibility contract as <input>
      if (clear(cs(input, 'borderTopColor')) && cs(input, 'boxShadow') === 'none')
        bad.push({ i, why: 'field-not-visible' })
      // listbox surface: elevated OR opaque fill (brut sets --shadow-lg: none by design)
      const noElev = cs(list, 'boxShadow') === 'none'
      const noFill = clear(cs(list, 'backgroundColor'))
      if (noElev && noFill) bad.push({ i, why: 'no-elevation-or-surface' })
      if (noFill) bad.push({ i, why: 'listbox-transparent' })

      const opts = [...list.querySelectorAll('option')]
      if (opts.length !== 3) return bad.push({ i, why: 'options-missing', n: opts.length })
      opts.forEach((o) => {
        const r = o.getBoundingClientRect()
        if (r.width <= 0 || r.height <= 0) bad.push({ i, why: 'option-not-rendered' })
      })
      const [active, plain, disabled] = opts
      if (cs(active, 'backgroundColor') === cs(plain, 'backgroundColor'))
        bad.push({ i, why: 'active-not-highlighted' })
      if (cs(disabled, 'opacity') === cs(plain, 'opacity'))
        bad.push({ i, why: 'disabled-not-distinct' })
    })

    // frozen-token guard — popover surfaces must differ light ship vs dark console
    const surf = (t) => {
      const el = document.querySelector(`[theme='${t}'] [role="listbox"]`)
      return el ? cs(el, 'backgroundColor') : null
    }
    if (surf('ship') && surf('console') && surf('ship') === surf('console'))
      bad.push({ why: 'frozen-token', light: surf('ship') })

    return { count: document.querySelectorAll('.sail-combobox').length, bad: bad.slice(0, 10) }
  },
  behavior: {
    rows() {
      return [
        // Match original structure as closely as possible for use:combobox + use:anchor
        `<div class="sail-combobox" id="cb" data-sail-host="combobox" placeholder="Pick a fruit" style="width:260px">` +
          `<input class="input" type="text" style="display:block;width:100%;min-height:34px;padding:6px 12px;border:1px solid var(--border);border-radius:8px;background:var(--background);color:var(--foreground)">` +
          `<div class="listbox" popover placement="bottom-start" role="listbox" style="${LIST_PAINT};border:1px solid var(--border);border-radius:8px;box-shadow:var(--shadow-lg);padding:4px;max-height:240px;overflow:auto;width:var(--anchor-width);min-width:var(--anchor-width)">` +
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
        page.evaluate(
          () => document.querySelector('#cb option[data-active]')?.getAttribute('value') ?? null,
        )
      /** Open without light-dismiss race (pointerdown open + click dismiss). */
      const openList = async () => {
        await input.focus()
        await page.evaluate(() => {
          const i = document.querySelector('#cb .input')
          i.dispatchEvent(new PointerEvent('pointerdown', { bubbles: true }))
        })
        await page.waitForTimeout(80)
      }

      // Callbacks report up via host prop (no CustomEvent bus).
      await page.evaluate(() => {
        globalThis.__picked = null
        document.getElementById('cb').onSelect = (detail) => {
          globalThis.__picked = detail?.value ?? null
        }
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

      // FOCUS / pointerdown opens the FULL list (picker-first)
      await openList()
      if (!(await list.isVisible())) bad.push({ why: 'focus-did-not-open-full-list' })
      const allOpen = await page.evaluate(() =>
        [...document.querySelectorAll('#cb option')]
          .filter((o) => !o.hidden)
          .map((o) => o.getAttribute('value')),
      )
      if (JSON.stringify(allOpen) !== JSON.stringify(['apple', 'banana', 'mango', 'melon']))
        bad.push({ why: 'focus-not-full-list', allOpen })
      if ((await input.getAttribute('aria-expanded')) !== 'true')
        bad.push({ why: 'expanded-not-synced-on-open' })

      // List width matches the field (w + minW from --anchor-width)
      const widths = await page.evaluate(() => {
        const i = document.querySelector('#cb .input')
        const l = document.querySelector('#cb .listbox')
        return {
          input: i.getBoundingClientRect().width,
          list: l.getBoundingClientRect().width,
          anchorW: getComputedStyle(l).getPropertyValue('--anchor-width').trim(),
        }
      })
      if (Math.abs(widths.input - widths.list) > 1)
        bad.push({ why: 'list-width-not-trigger', ...widths })

      // TYPE → filters
      await input.fill('')
      await openList()
      await input.type('an')
      await page.waitForTimeout(120)
      if (!(await list.isVisible())) bad.push({ why: 'typing-did-not-keep-open' })
      const visible = await page.evaluate(() =>
        [...document.querySelectorAll('#cb option')]
          .filter((o) => !o.hidden)
          .map((o) => o.getAttribute('value')),
      )
      // "an" matches banana + mango, not apple or melon
      if (JSON.stringify(visible) !== JSON.stringify(['banana', 'mango']))
        bad.push({ why: 'filter-wrong', visible })
      if ((await input.getAttribute('aria-expanded')) !== 'true')
        bad.push({ why: 'expanded-not-synced' })

      // the first MATCH is the active cursor, and aria-activedescendant points at it
      if ((await activeVal()) !== 'banana')
        bad.push({ why: 'first-match-not-active', got: await activeVal() })
      const ad = await page.evaluate(() => {
        const i = document.querySelector('#cb .input')
        return (
          i.getAttribute('aria-activedescendant') ===
          document.querySelector('#cb option[data-active]')?.id
        )
      })
      if (!ad) bad.push({ why: 'activedescendant-not-tracking' })

      // ARROWS rove within the FILTERED set — skipping hidden options is the point
      await page.keyboard.press('ArrowDown')
      if ((await activeVal()) !== 'mango')
        bad.push({ why: 'arrow-did-not-rove', got: await activeVal() })

      // ENTER commits: callback + closes + writes the label back into the field
      await page.keyboard.press('Enter')
      await page.waitForTimeout(80)
      if ((await page.evaluate(() => globalThis.__picked)) !== 'mango')
        bad.push({ why: 'enter-did-not-commit' })
      if (await list.isVisible()) bad.push({ why: 'commit-did-not-close' })
      if ((await input.inputValue()) !== 'Mango') bad.push({ why: 'label-not-written-back' })
      if ((await page.getAttribute('#cb', 'value')) !== 'mango')
        bad.push({ why: 'value-not-reflected' })

      // ESCAPE closes without committing
      await openList()
      await page.keyboard.press('ArrowDown')
      await page.waitForTimeout(100)
      if (!(await list.isVisible())) bad.push({ why: 'arrow-did-not-open' })
      await page.keyboard.press('Escape')
      await page.waitForTimeout(80)
      if (await list.isVisible()) bad.push({ why: 'escape-did-not-close' })

      // pointerdown on option commits — disabled option does not
      await page.evaluate(() => (globalThis.__picked = null))
      await openList()
      await page.waitForTimeout(80)
      await page.locator('#cb option[value=melon]').dispatchEvent('pointerdown')
      await page.waitForTimeout(80)
      if ((await page.evaluate(() => globalThis.__picked)) !== null)
        bad.push({ why: 'disabled-option-committed' })
      await page.locator('#cb option[value=apple]').dispatchEvent('pointerdown')
      await page.waitForTimeout(80)
      if ((await page.evaluate(() => globalThis.__picked)) !== 'apple')
        bad.push({ why: 'click-did-not-commit' })

      return bad
    },
  },
}
