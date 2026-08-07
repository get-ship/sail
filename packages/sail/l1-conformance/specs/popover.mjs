// L1 conformance spec: <popover> (mechanism-1 + native [popover] + use:anchor).
// RENDER: the surface in-flow across 7 themes (a real [popover] is top-layer +
// display:none, so the grid shows the skin). BEHAVIOR: the real popover, opened
// by its popovertarget trigger, anchored by use:anchor, closed by Escape.
// Paint classes mirror lowered Style: popover / popover-foreground / popover-border.

const POP =
  'sail-popover sx-minWpanelmenuminw sx-pad3 sx-fgpopoverforeground sx-bgpopover sx-borderWidth1 sx-borderpopoverborder sx-radiusmd sx-shadowmd sx-zpopover'

const surface = () =>
  `<div class="${POP}" style="position:static">Popover surface — anchored overlay content.</div>`

export default {
  tag: 'popover',
  width: 640,
  rows() { return [surface()] },
  probe() {
    const bad = []
    document.querySelectorAll('.sail-popover').forEach((el, i) => {
      const cs = getComputedStyle(el)
      const r = el.getBoundingClientRect()
      if (r.width <= 0 || r.height <= 0) bad.push({ i, why: 'not-rendered' })
      if (cs.borderTopWidth === '0px') bad.push({ i, why: 'no-border' })
      if (cs.boxShadow === 'none') bad.push({ i, why: 'no-elevation' })
      if (cs.backgroundColor === 'rgba(0, 0, 0, 0)' || cs.backgroundColor === 'transparent') {
        bad.push({ i, why: 'no-token-bg' })
      }
      if (!cs.color || cs.color === 'rgba(0, 0, 0, 0)') bad.push({ i, why: 'no-token-fg' })
    })
    return { count: document.querySelectorAll('.sail-popover').length, bad: bad.slice(0, 8) }
  },
  behavior: {
    rows() {
      return [
        `<button id="trg" popovertarget="p1">Open</button>` +
        `<div class="${POP}" id="p1" popover placement="bottom">Anchored content</div>`,
      ]
    },
    attach: [['.sail-popover', 'anchor']],
    async run(page) {
      const bad = []
      const pop = page.locator('#p1')
      if (await pop.isVisible()) bad.push({ why: 'open-before-trigger' })
      await page.click('#trg')
      await page.waitForTimeout(120) // use:anchor repositions on rAF
      if (!(await pop.isVisible())) bad.push({ why: 'trigger-did-not-open' })
      const tb = await page.locator('#trg').boundingBox()
      const pb = await pop.boundingBox()
      if (tb && pb && pb.y < tb.y) bad.push({ why: 'not-anchored-below-trigger' })
      await page.keyboard.press('Escape')
      if (await pop.isVisible()) bad.push({ why: 'escape-did-not-close' })
      return bad
    },
  },
}
