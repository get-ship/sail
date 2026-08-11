// L1 conformance spec: <toast> (mechanism-1 + use:autoDismiss + use:dismissControl).
// visual.cross: tone · duration.

const HOST =
  'sail-toast sail-row sail-g-3 sail-ai-start s-padY3 s-padX4 s-fgtoastforeground s-bgtoast s-radiusmd s-shadowmd s-borderWidth1 s-bordersolid s-bordertoastborder s-borderSWidth3 s-borderStoaststripe s-pointerEventsauto s-fontFamilybody s-fontSizesm'

const toast = (tone, body, extra = '') => {
  const toneAttr = tone ? `tone="${tone}"` : ''
  // toned instances spend s-borderStone via --tone (IACVT); rest uses toast-stripe
  const stripe = tone ? ' s-borderStone' : ''
  const toneVar =
    tone === 'muted'
      ? 'var(--muted-foreground)'
      : tone
        ? `var(--${tone})`
        : ''
  const style = toneVar ? `style="--tone: ${toneVar}"` : ''
  return (
    `<div class="${HOST}${stripe}" data-sail-host="toast" role="status" aria-live="polite" ${toneAttr} ${style} ${extra}>` +
    `<span>${body}</span><button data-toast-close aria-label="Dismiss">✕</button></div>`
  )
}

export default {
  tag: 'toast',
  width: 720,
  rows() {
    return [
      toast('', 'Rest tokens from component.toast.*', 'class="l1-mark" duration="0"'),
      toast('success', 'Saved your changes.', 'duration="0"'),
      toast('destructive', "Couldn't save — try again.", 'duration="4000"'),
      toast('info', 'A new version is available.', 'duration="2000"') +
        toast('warning', 'Storage almost full.') +
        toast('primary', 'Primary') +
        toast('accent', 'Accent') +
        toast('muted', 'Muted'),
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
      if (cs.boxShadow === 'none' && (cs.backgroundColor === 'rgba(0, 0, 0, 0)' || cs.backgroundColor === 'transparent')) bad.push({ i, why: 'no-elevation-or-surface' })
      if (cs.borderInlineStartWidth === '0px') bad.push({ i, why: 'no-tone-rail' })
      if (!el.querySelector('[data-toast-close]')) bad.push({ i, why: 'no-close-control' })
    })
    const els = [...document.querySelectorAll('.sail-toast')]
    const a = els.find((e) => e.getAttribute('tone') === 'success')
    const b = els.find((e) => e.getAttribute('tone') === 'destructive')
    if (a && b && getComputedStyle(a).borderInlineStartColor === getComputedStyle(b).borderInlineStartColor)
      bad.push({ why: 'tones-not-distinct' })
    if (!document.querySelector('.sail-toast[duration="0"]')) bad.push({ why: 'duration-0-missing' })
    if (!document.querySelector('.sail-toast[duration="4000"]')) bad.push({ why: 'duration-4000-missing' })
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
      // Runtime dismiss path is callback props (onDismiss) + remove — not CustomEvents.
      await page.evaluate(() => {
        globalThis.__dismissed = []
        for (const id of ['t-auto', 't-hover', 't-close']) {
          const el = document.getElementById(id)
          if (el) el.onDismiss = () => globalThis.__dismissed.push(id)
        }
      })

      await page.hover('#t-hover')

      await page.waitForTimeout(800)
      if ((await page.locator('#t-auto').count()) !== 0) bad.push({ why: 'auto-dismiss-did-not-fire' })
      if ((await page.locator('#t-hover').count()) === 0) bad.push({ why: 'hover-did-not-pause' })

      await page.mouse.move(5, 5)
      await page.waitForTimeout(1800)
      if ((await page.locator('#t-hover').count()) !== 0) bad.push({ why: 'leave-did-not-resume' })

      if ((await page.locator('#t-close').count()) === 0) bad.push({ why: 'sticky-toast-auto-dismissed' })
      await page.click('#t-close [data-toast-close]')
      if ((await page.locator('#t-close').count()) !== 0) bad.push({ why: 'close-control-did-not-dismiss' })

      const fired = await page.evaluate(() => globalThis.__dismissed)
      for (const id of ['t-auto', 't-hover', 't-close'])
        if (!fired.includes(id)) bad.push({ why: 'no-dismiss-callback', id })

      return bad
    },
  },
}
