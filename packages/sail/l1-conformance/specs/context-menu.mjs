// L1 conformance spec: <context-menu> (native [popover] + use:contextMenu + use:menu).
// RENDER: the menu surface in-flow across 7 themes. BEHAVIOR: right-click on the
// target opens it AT THE POINTER; Escape closes.
// Paint classes mirror lowered Style: context-menu* panel + menu-item* rows.

const CM =
  'sail-context-menu sx-pad1 sx-fgcontextmenuforeground sx-bgcontextmenu sx-borderWidth1 sx-bordercontextmenuborder sx-radiusmd sx-shadowmd sx-zpopover'
const MI =
  'sail-menu-item sx-displayflex sx-gap25 sx-padY2 sx-padX3 sx-wfull sx-borderWidth0 sx-fgmenuitemforeground sx-bgtransparent sx-radiussm sx-cursorpointer sx-textAlignstart sx-fontFamilybody sx-fontSizesm sx-leadingtight sx-whitespacenowrap sx-hover-bgmenuitemhoverbg sx-focus-bgmenuitemhoverbg sx-focus-outlinesolid sx-focus-outline2 sx-focus-outlinering sx-disabled-opacity50 sx-disabled-cursornotallowed'

const item = (value, label) =>
  `<button type="button" class="${MI}" role="menuitem" value="${value}">${label}</button>`

const menu = () =>
  `<div class="${CM}" role="menu" style="position:static">` +
  item('cut', 'Cut') +
  item('copy', 'Copy') +
  `</div>`

export default {
  tag: 'context-menu',
  width: 640,
  rows() { return [menu()] },
  probe() {
    const bad = []
    document.querySelectorAll('.sail-context-menu').forEach((el, i) => {
      const cs = getComputedStyle(el)
      if (el.getAttribute('role') !== 'menu') bad.push({ i, why: 'menu-role' })
      if (cs.boxShadow === 'none') bad.push({ i, why: 'no-elevation' })
      if (cs.backgroundColor === 'rgba(0, 0, 0, 0)' || cs.backgroundColor === 'transparent') {
        bad.push({ i, why: 'no-token-panel-bg' })
      }
      const items = [...el.querySelectorAll('.sail-menu-item')]
      if (!items.length) bad.push({ i, why: 'no-items' })
      items.forEach((it) => {
        const r = it.getBoundingClientRect()
        if (r.width <= 0 || r.height <= 0) bad.push({ i, why: 'item-not-rendered' })
        const ics = getComputedStyle(it)
        if (!ics.color || ics.color === 'rgba(0, 0, 0, 0)') bad.push({ i, why: 'no-token-item-fg' })
      })
    })
    return { count: document.querySelectorAll('.sail-context-menu').length, bad: bad.slice(0, 8) }
  },
  behavior: {
    rows() {
      return [
        `<div id="canvas" style="width:300px;height:80px;border:1px dashed var(--border);display:grid;place-items:center">right-click me</div>` +
        `<div class="${CM}" id="cm1" popover for="canvas">` +
        item('cut', 'Cut') +
        item('copy', 'Copy') +
        `</div>`,
      ]
    },
    attach: [['.sail-context-menu', 'contextMenu'], ['.sail-context-menu', 'menu']],
    async run(page) {
      const bad = []
      const cm = page.locator('#cm1')
      if (await cm.isVisible()) bad.push({ why: 'open-before-rightclick' })
      const box = await page.locator('#canvas').boundingBox()
      await page.click('#canvas', { button: 'right', position: { x: 40, y: 40 } })
      await page.waitForTimeout(100)
      if (!(await cm.isVisible())) bad.push({ why: 'rightclick-did-not-open' })
      // opened at the pointer (within a tolerance of where we clicked)
      const mb = await cm.boundingBox()
      if (mb && (Math.abs(mb.x - (box.x + 40)) > 24 || Math.abs(mb.y - (box.y + 40)) > 24))
        bad.push({ why: 'not-opened-at-pointer', at: { x: mb.x, y: mb.y } })
      await page.keyboard.press('Escape')
      await page.waitForTimeout(60)
      if (await cm.isVisible()) bad.push({ why: 'escape-did-not-close' })
      return bad
    },
  },
}
