// L1 conformance spec: <hover-card> (native [popover] + use:anchor + use:hoverSurface).
// visual.cross: placement.

const HC =
  'sail-hover-card s-wmax s-maxWpanelmaxwsm s-pad3 s-fghovercardforeground s-bghovercard s-borderWidth1 s-bordersolid s-borderhovercardborder s-radiusmd s-shadowmd s-zpopover s-fontFamilybody s-fontSizesm'

const surface = (placement = 'top', extra = '') =>
  `<div class="${HC}" data-sail-host="hover-card" placement="${placement}" ${extra} style="position:static"><strong>@ada</strong><div>Mathematician. First programmer.</div></div>`

export default {
  tag: 'hover-card',
  width: 640,
  rows() {
    return [
      surface('top', 'class="l1-mark"'),
      surface('bottom') + surface('left') + surface('right'),
    ]
  },
  probe() {
    const bad = []
    document.querySelectorAll('.sail-hover-card').forEach((el, i) => {
      const cs = getComputedStyle(el)
      const r = el.getBoundingClientRect()
      if (r.width <= 0 || r.height <= 0) bad.push({ i, why: 'not-rendered' })
      if (cs.boxShadow === 'none' && (cs.backgroundColor === 'rgba(0, 0, 0, 0)' || cs.backgroundColor === 'transparent')) bad.push({ i, why: 'no-elevation-or-surface' })
      if (cs.borderTopWidth === '0px') bad.push({ i, why: 'no-border' })
      if (cs.backgroundColor === 'rgba(0, 0, 0, 0)' || cs.backgroundColor === 'transparent') {
        bad.push({ i, why: 'no-token-bg' })
      }
      if (!cs.color || cs.color === 'rgba(0, 0, 0, 0)') bad.push({ i, why: 'no-token-fg' })
    })
    return { count: document.querySelectorAll('.sail-hover-card').length, bad: bad.slice(0, 8) }
  },
  behavior: {
    rows() {
      return [
        `<button id="trg" popovertarget="hc1" style="margin-top:200px">@ada</button>` +
        `<div class="${HC}" id="hc1" data-sail-host="hover-card" popover placement="top" open-delay="60" close-delay="40">Profile</div>`,
      ]
    },
    attach: [['.sail-hover-card', 'anchor'], ['.sail-hover-card', 'hoverSurface']],
    async run(page) {
      const bad = []
      const hc = page.locator('#hc1')
      if (await hc.isVisible()) bad.push({ why: 'open-before-hover' })
      await page.hover('#trg')
      await page.waitForTimeout(260)
      if (!(await hc.isVisible())) bad.push({ why: 'hover-did-not-open' })
      // placement geometry is use:anchor depth — scaffold asserts open/close only
      await page.mouse.move(5, 5)
      await page.waitForTimeout(200)
      if (await hc.isVisible()) bad.push({ why: 'leave-did-not-close' })
      return bad
    },
  },
}
