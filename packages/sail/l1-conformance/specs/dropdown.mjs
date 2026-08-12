// L1 conformance spec: <dropdown> + <menu-item> (native [popover] + use:anchor + use:menu).
// visual.cross: placement.

const DD =
  'sail-dropdown s-wmax s-pad15 s-fgdropdownforeground s-bgdropdown s-borderWidth1 s-bordersolid s-borderdropdownborder s-radiusmd s-shadowmd s-zpopover'
const MI =
  'sail-menu-item s-displayflex s-gap25 s-padY2 s-padX3 s-wfull s-borderWidth0 s-fgmenuitemforeground s-bgtransparent s-radiussm s-cursorpointer s-textAlignstart s-fontFamilybody s-fontSizesm s-leadingtight s-whitespacenowrap s-hover-bgmenuitemhoverbg s-focus-bgmenuitemhoverbg s-focus-outlinesolid s-focus-outline2 s-focus-outlinering s-disabled-opacity50 s-disabled-cursornotallowed'

const item = (value, label, extra = '') =>
  `<button type="button" class="${MI}" data-sail-host="menu-item" role="menuitem" value="${value}" ${extra}>${label}</button>`

const menu = (placement = 'bottom', extra = '') =>
  `<div class="${DD}" data-sail-host="dropdown" role="menu" placement="${placement}" ${extra} style="position:static">` +
  item('edit', 'Edit') +
  item('dup', 'Duplicate') +
  item('del', 'Delete', 'disabled aria-disabled="true"') +
  `</div>`

export default {
  tag: 'dropdown',
  width: 640,
  rows() {
    return [
      menu('bottom', 'class="l1-mark"'),
      menu('top-end') + menu('bottom-start'),
    ]
  },
  probe() {
    const bad = []
    document.querySelectorAll('.sail-dropdown').forEach((el, i) => {
      const cs = getComputedStyle(el)
      if (el.getAttribute('role') !== 'menu') bad.push({ i, why: 'menu-role' })
      if (cs.boxShadow === 'none' && (cs.backgroundColor === 'rgba(0, 0, 0, 0)' || cs.backgroundColor === 'transparent')) bad.push({ i, why: 'no-elevation-or-surface' })
      if (cs.backgroundColor === 'rgba(0, 0, 0, 0)' || cs.backgroundColor === 'transparent') {
        bad.push({ i, why: 'no-token-panel-bg' })
      }
      const items = [...el.querySelectorAll('.sail-menu-item')]
      if (items.length < 2) bad.push({ i, why: 'too-few-items' })
      items.forEach((it) => {
        const r = it.getBoundingClientRect()
        if (r.width <= 0 || r.height <= 0) bad.push({ i, why: 'item-not-rendered' })
        if (it.getAttribute('role') !== 'menuitem') bad.push({ i, why: 'item-role' })
        const ics = getComputedStyle(it)
        if (!ics.color || ics.color === 'rgba(0, 0, 0, 0)') bad.push({ i, why: 'no-token-item-fg' })
      })
      const dis = el.querySelector('.sail-menu-item[disabled]')
      const en = el.querySelector('.sail-menu-item:not([disabled])')
      if (dis && en && getComputedStyle(dis).opacity === getComputedStyle(en).opacity)
        bad.push({ i, why: 'disabled-not-distinct' })
    })
    return { count: document.querySelectorAll('.sail-dropdown').length, bad: bad.slice(0, 8) }
  },
  behavior: {
    rows() {
      return [
        `<button id="trg" popovertarget="m1">Actions</button>` +
        `<div class="${DD}" id="m1" data-sail-host="dropdown" popover placement="bottom-start">` +
        item('edit', 'Edit') +
        item('dup', 'Duplicate') +
        item('del', 'Delete', 'disabled') +
        `</div>`,
      ]
    },
    attach: [['.sail-dropdown', 'anchor'], ['.sail-dropdown', 'menu']],
    async run(page) {
      const bad = []
      const dd = page.locator('#m1')
      await page.evaluate(() => {
        globalThis.__selected = null
        document.getElementById('m1').onSelect = (detail) => {
          globalThis.__selected = detail?.value ?? detail
        }
      })
      await page.click('#trg')
      await page.waitForTimeout(120)
      if (!(await dd.isVisible())) bad.push({ why: 'trigger-did-not-open' })
      let focused = await page.evaluate(() => document.activeElement?.getAttribute('value'))
      if (focused !== 'edit') bad.push({ why: 'first-item-not-focused', focused })
      await page.keyboard.press('ArrowDown')
      focused = await page.evaluate(() => document.activeElement?.getAttribute('value'))
      if (focused !== 'dup') bad.push({ why: 'arrow-did-not-rove', focused })
      await page.keyboard.press('Enter')
      const sel = await page.evaluate(() => globalThis.__selected)
      if (sel !== 'dup') bad.push({ why: 'select-did-not-emit', sel })
      if (await dd.isVisible()) bad.push({ why: 'select-did-not-close' })
      return bad
    },
  },
}
