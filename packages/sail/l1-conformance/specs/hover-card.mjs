// L1 conformance spec: <hover-card> (native [popover] + use:anchor + use:hoverCard).
// RENDER: the card surface in-flow across 7 themes. BEHAVIOR: hovering the trigger
// opens it after the dwell delay; leaving closes it.

const surface = () =>
  `<div class="sail-hover-card" style="position:static"><strong>@ada</strong><div>Mathematician. First programmer.</div></div>`

export default {
  tag: 'hover-card',
  width: 640,
  rows() { return [surface()] },
  probe() {
    const bad = []
    document.querySelectorAll('.sail-hover-card').forEach((el, i) => {
      const cs = getComputedStyle(el)
      const r = el.getBoundingClientRect()
      if (r.width <= 0 || r.height <= 0) bad.push({ i, why: 'not-rendered' })
      if (cs.boxShadow === 'none') bad.push({ i, why: 'no-elevation' })
      if (cs.borderTopWidth === '0px') bad.push({ i, why: 'no-border' })
    })
    return { count: document.querySelectorAll('.sail-hover-card').length, bad: bad.slice(0, 8) }
  },
  behavior: {
    rows() {
      return [
        `<button id="trg" popovertarget="hc1" style="margin-top:200px">@ada</button>` +
        `<div class="sail-hover-card" id="hc1" popover placement="top" open-delay="60" close-delay="40">Profile</div>`,
      ]
    },
    attach: [['.sail-hover-card', 'anchor'], ['.sail-hover-card', 'hoverCard']],
    async run(page) {
      const bad = []
      const hc = page.locator('#hc1')
      if (await hc.isVisible()) bad.push({ why: 'open-before-hover' })
      await page.hover('#trg')
      await page.waitForTimeout(260) // open-delay + rAF reposition
      if (!(await hc.isVisible())) bad.push({ why: 'hover-did-not-open' })
      // anchored above the trigger (placement=top)
      const tb = await page.locator('#trg').boundingBox()
      const hb = await hc.boundingBox()
      if (tb && hb && hb.y > tb.y) bad.push({ why: 'not-anchored-above' })
      // leaving closes it after the dwell
      await page.mouse.move(5, 5)
      await page.waitForTimeout(200)
      if (await hc.isVisible()) bad.push({ why: 'leave-did-not-close' })
      return bad
    },
  },
}
