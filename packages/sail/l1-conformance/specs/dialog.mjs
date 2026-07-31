// L1 conformance spec: <dialog> + the sheet variant (mechanism-2, native + skin).
// ZERO JS: the platform's invoker commands (command="show-modal" / "close") open
// and close it, Escape and the backdrop are native, top-layer is native. A SHEET
// is not a second element — it's <dialog side="…">, the same primitive docked to
// an edge. RENDER: the surface + header/section/footer + sizes, forced in-flow
// (a real modal is top-layer, so it can't sit in a 7-up grid).
// BEHAVIOR: the real thing — invoker opens it modally, invoker + Escape close it.

const DL =
  'sail-dialog sx-positionfixed sx-inset0 sx-mauto sx-pad0 sx-wfull sx-maxWpanelmaxwmd sx-overflowhidden sx-borderWidth1 sx-borderborder sx-radiusxl sx-bgsurface1 sx-fgcardforeground sx-shadowxl sx-fontFamilybody sx-open-displayflex sx-open-flexDircolumn sx-sizesm-maxWpanelmaxwsm sx-sizemd-maxWpanelmaxwmd sx-sizelg-maxWpanelmaxwlg sx-sizefull-maxWfull sx-sizefull-maxHfull sx-sidecenter-mauto sx-sidestart-inset0 sx-sidestart-maxWpanelmaxwsm sx-sidestart-radiusnone sx-sidestart-m0 sx-sideend-inset0 sx-sideend-maxWpanelmaxwsm sx-sideend-radiusnone sx-sideend-m0 sx-sidetop-inset0 sx-sidetop-wfull sx-sidetop-maxWnone sx-sidetop-radiusnone sx-sidetop-m0 sx-sidebottom-inset0 sx-sidebottom-wfull sx-sidebottom-maxWnone sx-sidebottom-radiusnone sx-sidebottom-m0'

const shell = (attrs = '') => {
  const m = attrs.match(/class="([^"]*)"/)
  const rest = attrs.replace(/class="[^"]*"\s*/, '').trim()
  const cls = m ? `${DL} ${m[1]}` : DL
  return (
    `<dialog open class="${cls}" ${rest} style="position:static;margin:0;max-height:none">` +
    `<header>Delete project</header>` +
    `<section>This cannot be undone.</section>` +
    `<footer><button variant="ghost">Cancel</button><button variant="destructive">Delete</button></footer>` +
    `</dialog>`
  )
}

export default {
  tag: 'dialog',
  width: 760,
  rows() {
    return [
      shell('class="l1-mark"'),
      shell("size='sm'"),
      shell("side='end'"),
    ]
  },
  probe() {
    const bad = []
    const cs = (el, p) => getComputedStyle(el)[p]
    document.querySelectorAll('dialog').forEach((el, i) => {
      const r = el.getBoundingClientRect()
      if (r.width <= 0 || r.height <= 0) return bad.push({ i, why: 'not-rendered' })
      // an open dialog is a flex column: header / scrollable body / footer
      if (cs(el, 'display') !== 'flex') bad.push({ i, why: 'open-not-flex', display: cs(el, 'display') })
      if (cs(el, 'boxShadow') === 'none') bad.push({ i, why: 'no-elevation' })
      const head = el.querySelector('header')
      const body = el.querySelector('section')
      const foot = el.querySelector('footer')
      if (!head || !body || !foot) return bad.push({ i, why: 'missing-slot' })
      // the header/footer must be visually separated from the body
      if (cs(head, 'borderBottomWidth') === '0px') bad.push({ i, why: 'header-not-divided' })
      if (cs(foot, 'borderTopWidth') === '0px') bad.push({ i, why: 'footer-not-divided' })
      // the body is the ONLY part that scrolls
      if (cs(body, 'overflowY') !== 'auto') bad.push({ i, why: 'body-not-scrollable' })
    })
    const scope = "[data-theme-preview='ship'] "
    const q = (s) => document.querySelector(scope + s)
    // the size channel actually narrows
    const w = (s) => q(s).getBoundingClientRect().width
    if (!(w("dialog[size='sm']") < w('dialog.l1-mark')))
      bad.push({ why: 'size-not-monotonic', sm: w("dialog[size='sm']"), md: w('dialog.l1-mark') })
    // a sheet squares off the edge it's docked to — it is NOT the same box
    if (cs(q("dialog[side='end']"), 'borderTopRightRadius') !== '0px')
      bad.push({ why: 'sheet-not-squared' })
    // frozen-token guard
    const mark = (t) => cs(document.querySelector(`[data-theme-preview='${t}'] .l1-mark`), 'backgroundColor')
    if (mark('ship') === mark('console')) bad.push({ why: 'frozen-token', light: mark('ship') })
    return { count: document.querySelectorAll('dialog').length, bad: bad.slice(0, 10) }
  },
  behavior: {
    // No actions to attach — the dialog is pure platform. That IS the assertion.
    rows() {
      return [
        `<button id="open" command="show-modal" commandfor="d1">Open</button>` +
        `<dialog id="d1" class="${DL}"><header>Hi</header><section>Body</section>` +
        `<footer><button id="x" command="close" commandfor="d1">Close</button></footer></dialog>`,
      ]
    },
    attach: [],
    async run(page) {
      const bad = []
      const isOpen = () => page.evaluate(() => document.getElementById('d1').open)
      const isModal = () => page.evaluate(() => document.getElementById('d1').matches(':modal'))

      if (await isOpen()) bad.push({ why: 'open-before-invoke' })
      await page.click('#open')
      await page.waitForTimeout(60)
      if (!(await isOpen())) bad.push({ why: 'invoker-did-not-open' })
      // show-modal must be MODAL (top layer + inert page), not just visible
      if (!(await isModal())) bad.push({ why: 'not-modal' })

      // the close invoker closes it
      await page.click('#x')
      await page.waitForTimeout(60)
      if (await isOpen()) bad.push({ why: 'close-invoker-did-not-close' })

      // Escape closes it — free, from the platform
      await page.click('#open')
      await page.waitForTimeout(60)
      await page.keyboard.press('Escape')
      await page.waitForTimeout(60)
      if (await isOpen()) bad.push({ why: 'escape-did-not-close' })
      return bad
    },
  },
}
