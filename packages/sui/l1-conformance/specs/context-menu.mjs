// L1 conformance spec: <context-menu> (native [popover] + use:contextMenu + use:menu).
// RENDER: the menu surface in-flow across 7 themes. BEHAVIOR: right-click on the
// target opens it AT THE POINTER; Escape closes.

const menu = () =>
  `<div class="sui-context-menu" role="menu" style="position:static">` +
  `<button class="sui-menu-item" role="menuitem" value="cut">Cut</button>` +
  `<button class="sui-menu-item" role="menuitem" value="copy">Copy</button>` +
  `</div>`

export default {
  tag: 'context-menu',
  width: 640,
  rows() { return [menu()] },
  probe() {
    const bad = []
    document.querySelectorAll('.sui-context-menu').forEach((el, i) => {
      const cs = getComputedStyle(el)
      if (el.getAttribute('role') !== 'menu') bad.push({ i, why: 'menu-role' })
      if (cs.boxShadow === 'none') bad.push({ i, why: 'no-elevation' })
      const items = [...el.querySelectorAll('.sui-menu-item')]
      if (!items.length) bad.push({ i, why: 'no-items' })
      items.forEach((it) => {
        const r = it.getBoundingClientRect()
        if (r.width <= 0 || r.height <= 0) bad.push({ i, why: 'item-not-rendered' })
      })
    })
    return { count: document.querySelectorAll('.sui-context-menu').length, bad: bad.slice(0, 8) }
  },
  behavior: {
    rows() {
      return [
        `<div id="canvas" style="width:300px;height:80px;border:1px dashed var(--border);display:grid;place-items:center">right-click me</div>` +
        `<div class="sui-context-menu" id="cm1" popover for="canvas">` +
        `<button class="sui-menu-item" value="cut">Cut</button>` +
        `<button class="sui-menu-item" value="copy">Copy</button>` +
        `</div>`,
      ]
    },
    attach: [['.sui-context-menu', 'contextMenu'], ['.sui-context-menu', 'menu']],
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
