// L1 conformance spec: <toast> + <toast-region> (mechanism-1 + use:autoDismiss
// + use:dismissControl). RENDER: toasts across tones in-flow (the region is
// position:fixed, so the grid shows the toasts themselves). BEHAVIOR:
// auto-dismiss fires, hovering PAUSES it, and the close control dismisses
// immediately — each emitting sail-toast-dismiss.

const toast = (tone, body, extra = '') =>
  `<div class="sail-toast" role="status" aria-live="polite" tone="${tone}" ${extra}>` +
  `<span>${body}</span><button data-toast-close aria-label="Dismiss">✕</button></div>`

export default {
  tag: 'toast',
  width: 720,
  rows() {
    return [
      toast('success', 'Saved your changes.'),
      toast('destructive', "Couldn't save — try again."),
      toast('info', 'A new version is available.') + toast('warning', 'Storage almost full.'),
    ]
  },
  probe() {
    const bad = []
    document.querySelectorAll('.sail-toast').forEach((el, i) => {
      const cs = getComputedStyle(el)
      const r = el.getBoundingClientRect()
      if (r.width <= 0 || r.height <= 0) bad.push({ i, why: 'not-rendered' })
      if (el.getAttribute('role') !== 'status') bad.push({ i, why: 'no-status-role' })
      if (el.getAttribute('aria-live') !== 'polite') bad.push({ i, why: 'no-live-region' })
      if (cs.boxShadow === 'none') bad.push({ i, why: 'no-elevation' })
      // the tone rail must actually paint per tone
      if (cs.borderInlineStartWidth === '0px') bad.push({ i, why: 'no-tone-rail' })
      if (!el.querySelector('[data-toast-close]')) bad.push({ i, why: 'no-close-control' })
    })
    // tones must be visually distinguishable from each other
    const els = [...document.querySelectorAll('.sail-toast')]
    const a = els.find((e) => e.getAttribute('tone') === 'success')
    const b = els.find((e) => e.getAttribute('tone') === 'destructive')
    if (a && b && getComputedStyle(a).borderInlineStartColor === getComputedStyle(b).borderInlineStartColor)
      bad.push({ why: 'tones-not-distinct' })
    return { count: els.length, bad: bad.slice(0, 8) }
  },
  behavior: {
    rows() {
      return [
        toast('info', 'Auto', 'id="t-auto" duration="400"'),
        toast('info', 'Hover-paused', 'id="t-hover" duration="1500"'),
        toast('info', 'Manual', 'id="t-close" duration="0"'),
      ]
    },
    attach: [['.sail-toast', 'autoDismiss'], ['.sail-toast', 'dismissControl']],
    async run(page) {
      const bad = []
      await page.evaluate(() => {
        globalThis.__dismissed = []
        document.addEventListener('sail-toast-dismiss', (e) =>
          globalThis.__dismissed.push(e.target.id),
        )
      })

      // hover BEFORE the timer would fire → must pause and survive
      await page.hover('#t-hover')

      // the un-hovered auto toast dismisses itself after its duration; the
      // hovered one's (much longer) timer is paused, so it must survive
      await page.waitForTimeout(800)
      if ((await page.locator('#t-auto').count()) !== 0) bad.push({ why: 'auto-dismiss-did-not-fire' })
      if ((await page.locator('#t-hover').count()) === 0) bad.push({ why: 'hover-did-not-pause' })

      // leaving resumes the timer → it then dismisses
      await page.mouse.move(5, 5)
      await page.waitForTimeout(1800)
      if ((await page.locator('#t-hover').count()) !== 0) bad.push({ why: 'leave-did-not-resume' })

      // duration=0 is sticky; its close control dismisses it
      if ((await page.locator('#t-close').count()) === 0) bad.push({ why: 'sticky-toast-auto-dismissed' })
      await page.click('#t-close [data-toast-close]')
      if ((await page.locator('#t-close').count()) !== 0) bad.push({ why: 'close-control-did-not-dismiss' })

      const fired = await page.evaluate(() => globalThis.__dismissed)
      for (const id of ['t-auto', 't-hover', 't-close'])
        if (!fired.includes(id)) bad.push({ why: 'no-dismiss-event', id })

      return bad
    },
  },
}
