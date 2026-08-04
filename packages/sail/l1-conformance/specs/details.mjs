// L1 conformance spec: <details>/<summary> (mechanism-2, native + skin, zero JS).
// The platform's disclosure: open/close, the toggle event, keyboard and
// find-in-page expansion all come free. The skin's job is to make the summary read
// as a control (pointer, marker, hover) and the panel read as its content.
// BEHAVIOR: no `use:` action to attach — clicking the summary toggles it, and that
// the assertion passes with an EMPTY attach list is the point: this costs no JS.

const DT = 'sail-details sx-borderWidth1 sx-borderborder sx-radiuslg sx-bgsurface1'
const SM =
  'sail-summary sx-displayflex sx-padY3 sx-padX4 sx-fontSizesm sx-fontWeightsemibold sx-fgforeground sx-cursorpointer sx-listnone sx-selectnone sx-radiuslg sx-transitioncolors sx-durationfast sx-easeout sx-hover-bghoverbg sx-after-contentempty sx-after-msauto sx-after-size4 sx-after-bgmutedforeground'

const d = (attrs = '', label = 'Details') => {
  const m = attrs.match(/class="([^"]*)"/)
  const rest = attrs.replace(/class="[^"]*"\s*/, '').trim()
  const cls = m ? `${DT} ${m[1]}` : DT
  return `<details class="${cls}" ${rest} style="width:280px"><summary class="${SM}">${label}</summary><p>Panel content.</p></details>`
}

export default {
  tag: 'details',
  width: 700,
  rows() {
    return [d('class="l1-mark"'), d('open'), d('disabled')]
  },
  probe() {
    const bad = []
    const cs = (el, p) => getComputedStyle(el)[p]

    document.querySelectorAll('details').forEach((el, i) => {
      const sum = el.querySelector('summary')
      if (!sum) return bad.push({ i, why: 'no-summary' })
      const r = sum.getBoundingClientRect()
      if (r.width <= 0 || r.height <= 0) return bad.push({ i, why: 'summary-not-rendered' })
      // a summary is a CONTROL — it must say so on hover
      if (cs(sum, 'cursor') !== 'pointer') bad.push({ i, why: 'summary-not-a-control', cursor: cs(sum, 'cursor') })
      // Closed, the box IS the summary; open, it's taller. (Measure the <details>,
      // not the inner <p>: the skin animates ::details-content from block-size 0,
      // which clips the panel while the <p> keeps its own intrinsic height.)
      const grew = el.getBoundingClientRect().height - r.height > 4
      if (grew !== el.open) bad.push({ i, why: 'panel-out-of-sync', open: el.open, grew })
    })

    const scope = "[theme='ship'] "
    const q = (s) => document.querySelector(scope + s)
    // The chevron must turn when open — the affordance that says which way the
    // disclosure goes. The skin hides the UA ::marker and draws its own on
    // summary::after, rotating it 180deg on [open].
    const arrow = (sel) => getComputedStyle(q(sel), '::after').transform
    if (arrow('details.l1-mark summary') === arrow('details[open] summary'))
      bad.push({ why: 'no-open-affordance', shut: arrow('details.l1-mark summary') })
    const mark = (t) => cs(document.querySelector(`[theme='${t}'] .l1-mark summary`), 'color')
    if (mark('ship') === mark('console')) bad.push({ why: 'frozen-token', light: mark('ship') })

    return { count: document.querySelectorAll('details').length, bad: bad.slice(0, 10) }
  },
  behavior: {
    rows() { return [d('id="dt"')] },
    attach: [], // zero JS — the platform owns this one outright
    async run(page) {
      const bad = []
      const isOpen = () => page.evaluate(() => document.getElementById('dt').open)
      if (await isOpen()) bad.push({ why: 'open-at-rest' })
      await page.click('#dt summary')
      await page.waitForTimeout(60)
      if (!(await isOpen())) bad.push({ why: 'click-did-not-open' })
      await page.click('#dt summary')
      await page.waitForTimeout(60)
      if (await isOpen()) bad.push({ why: 'click-did-not-close' })
      // keyboard: the summary is focusable and Enter toggles it
      await page.focus('#dt summary')
      await page.keyboard.press('Enter')
      await page.waitForTimeout(60)
      if (!(await isOpen())) bad.push({ why: 'enter-did-not-open' })
      return bad
    },
  },
}
