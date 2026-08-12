// L1 conformance spec: <context-menu> (native [popover] + use:contextMenu + use:menu).

const CM =
  'sail-context-menu s-pad1 s-fgcontextmenuforeground s-bgcontextmenu s-borderWidth1 s-bordersolid s-bordercontextmenuborder s-radiusmd s-shadowmd s-zpopover'
const MI =
  'sail-menu-item s-displayflex s-gap25 s-padY2 s-padX3 s-wfull s-borderWidth0 s-fgmenuitemforeground s-bgtransparent s-radiussm s-cursorpointer s-textAlignstart s-fontFamilybody s-fontSizesm s-leadingtight s-whitespacenowrap s-hover-bgmenuitemhoverbg s-focus-bgmenuitemhoverbg s-focus-outlinesolid s-focus-outline2 s-focus-outlinering s-disabled-opacity50 s-disabled-cursornotallowed'

const item = (value, label) =>
  `<button type="button" class="${MI}" data-sail-host="menu-item" role="menuitem" value="${value}">${label}</button>`

const menu = () =>
  `<div class="${CM}" data-sail-host="context-menu" role="menu" style="position:static">` +
  item('cut', 'Cut') +
  item('copy', 'Copy') +
  `</div>`

export default {
  tag: 'context-menu',
  width: 640,
  rows() {
    return [menu()]
  },
  probe() {
    const bad = []
    document.querySelectorAll('.sail-context-menu').forEach((el, i) => {
      const cs = getComputedStyle(el)
      if (el.getAttribute('role') !== 'menu') bad.push({ i, why: 'menu-role' })
      if (cs.boxShadow === 'none' && (cs.backgroundColor === 'rgba(0, 0, 0, 0)' || cs.backgroundColor === 'transparent')) bad.push({ i, why: 'no-elevation-or-surface' })
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
        `<div class="${CM}" id="cm1" data-sail-host="context-menu" popover for="canvas">` +
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
      // pointer geometry is use:contextMenu depth — scaffold asserts open/escape only
      await page.keyboard.press('Escape')
      await page.waitForTimeout(60)
      if (await cm.isVisible()) bad.push({ why: 'escape-did-not-close' })
      return bad
    },
  },
}
