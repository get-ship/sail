// L1 conformance spec: <popover> (native [popover] + use:anchor).
// visual.cross: placement (free string).

const POP =
  'sail-popover s-minWpanelmenuminw s-pad3 s-fgpopoverforeground s-bgpopover s-borderWidth1 s-bordersolid s-borderpopoverborder s-radiusmd s-shadowmd s-zpopover'

const surface = (placement = 'bottom', extra = '') =>
  `<div class="${POP}" data-sail-host="popover" placement="${placement}" ${extra} style="position:static">Popover surface — ${placement}.</div>`

export default {
  tag: 'popover',
  width: 640,
  rows() {
    return [
      surface('bottom', 'class="l1-mark"'),
      surface('top') + surface('left') + surface('right') + surface('bottom-start'),
    ]
  },
  probe() {
    const bad = []
    document.querySelectorAll('.sail-popover').forEach((el, i) => {
      const cs = getComputedStyle(el)
      const r = el.getBoundingClientRect()
      if (r.width <= 0 || r.height <= 0) bad.push({ i, why: 'not-rendered' })
      if (cs.borderTopWidth === '0px') bad.push({ i, why: 'no-border' })
      if (cs.boxShadow === 'none' && (cs.backgroundColor === 'rgba(0, 0, 0, 0)' || cs.backgroundColor === 'transparent')) bad.push({ i, why: 'no-elevation-or-surface' })
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
        `<div class="${POP}" id="p1" data-sail-host="popover" popover placement="bottom">Anchored content</div>`,
      ]
    },
    attach: [['.sail-popover', 'anchor']],
    async run(page) {
      const bad = []
      const pop = page.locator('#p1')
      if (await pop.isVisible()) bad.push({ why: 'open-before-trigger' })
      await page.click('#trg')
      await page.waitForTimeout(120)
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
