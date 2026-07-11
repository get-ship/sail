// L1 conformance spec: <dropdown> + <menu-item> (native [popover] + use:anchor +
// use:menu). RENDER: the menu surface + items in-flow across 7 themes.
// BEHAVIOR: trigger opens it, first item focuses, arrows rove, select closes + emits.

const menu = () =>
  `<div class="sui-dropdown" role="menu" style="position:static">` +
  `<button class="sui-menu-item" role="menuitem" value="edit">Edit</button>` +
  `<button class="sui-menu-item" role="menuitem" value="dup">Duplicate</button>` +
  `<button class="sui-menu-item" role="menuitem" value="del" disabled aria-disabled="true">Delete</button>` +
  `</div>`

export default {
  tag: 'dropdown',
  width: 640,
  rows() { return [menu()] },
  probe() {
    const bad = []
    document.querySelectorAll('.sui-dropdown').forEach((el, i) => {
      const cs = getComputedStyle(el)
      if (el.getAttribute('role') !== 'menu') bad.push({ i, why: 'menu-role' })
      if (cs.boxShadow === 'none') bad.push({ i, why: 'no-elevation' })
      const items = [...el.querySelectorAll('.sui-menu-item')]
      if (items.length < 2) bad.push({ i, why: 'too-few-items' })
      items.forEach((it) => {
        const r = it.getBoundingClientRect()
        if (r.width <= 0 || r.height <= 0) bad.push({ i, why: 'item-not-rendered' })
        if (it.getAttribute('role') !== 'menuitem') bad.push({ i, why: 'item-role' })
      })
      const dis = el.querySelector('.sui-menu-item[disabled]')
      const en = el.querySelector('.sui-menu-item:not([disabled])')
      if (dis && en && getComputedStyle(dis).opacity === getComputedStyle(en).opacity)
        bad.push({ i, why: 'disabled-not-distinct' })
    })
    return { count: document.querySelectorAll('.sui-dropdown').length, bad: bad.slice(0, 8) }
  },
  behavior: {
    rows() {
      return [
        `<button id="trg" popovertarget="m1">Actions</button>` +
        `<div class="sui-dropdown" id="m1" popover placement="bottom-start">` +
        `<button class="sui-menu-item" value="edit">Edit</button>` +
        `<button class="sui-menu-item" value="dup">Duplicate</button>` +
        `<button class="sui-menu-item" value="del" disabled>Delete</button>` +
        `</div>`,
      ]
    },
    attach: [['.sui-dropdown', 'anchor'], ['.sui-dropdown', 'menu']],
    async run(page) {
      const bad = []
      const dd = page.locator('#m1')
      await page.evaluate(() => {
        globalThis.__selected = null
        document.getElementById('m1').addEventListener('sui-menu-select', (e) => {
          globalThis.__selected = e.detail.value
        })
      })
      await page.click('#trg')
      await page.waitForTimeout(120) // popover `toggle` (→ first-item focus) is async
      if (!(await dd.isVisible())) bad.push({ why: 'trigger-did-not-open' })
      // first item is focused on open
      let focused = await page.evaluate(() => document.activeElement?.getAttribute('value'))
      if (focused !== 'edit') bad.push({ why: 'first-item-not-focused', focused })
      // ArrowDown roves
      await page.keyboard.press('ArrowDown')
      focused = await page.evaluate(() => document.activeElement?.getAttribute('value'))
      if (focused !== 'dup') bad.push({ why: 'arrow-did-not-rove', focused })
      // select → emits + closes
      await page.keyboard.press('Enter')
      const sel = await page.evaluate(() => globalThis.__selected)
      if (sel !== 'dup') bad.push({ why: 'select-did-not-emit', sel })
      if (await dd.isVisible()) bad.push({ why: 'select-did-not-close' })
      return bad
    },
  },
}
