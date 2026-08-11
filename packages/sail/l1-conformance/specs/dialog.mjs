// L1 conformance spec: <dialog> + sheet sides (mechanism-2, native + skin).
// visual.cross: size · backdrop · side · dismissible.

const DL =
  'sail-dialog s-positionfixed s-inset0 s-mauto s-pad0 s-wfull s-maxWpanelmaxwmd s-minWpanelmaxwsm s-overflowhidden s-borderWidth1 s-bordersolid s-borderdialogborder s-radiusxl s-bgdialog s-fgdialogforeground s-shadowxl s-fontFamilybody s-open-displayflex s-open-flexDircolumn s-open-hfit s-open-maxHpanelvhmd s-sizesm-maxWpanelmaxwsm s-sizemd-maxWpanelmaxwmd s-sizelg-maxWpanelmaxwlg s-sizefull-maxWfull s-sizefull-maxHfull s-sidecenter-mauto s-sidestart-inset0 s-sidestart-maxWpanelmaxwsm s-sidestart-radiusnone s-sidestart-m0 s-sideend-inset0 s-sideend-maxWpanelmaxwsm s-sideend-radiusnone s-sideend-m0 s-sidetop-inset0 s-sidetop-wfull s-sidetop-maxWnone s-sidetop-radiusnone s-sidetop-m0 s-sidebottom-inset0 s-sidebottom-wfull s-sidebottom-maxWnone s-sidebottom-radiusnone s-sidebottom-m0'

const shell = (attrs = '') => {
  const m = attrs.match(/class="([^"]*)"/)
  const rest = attrs.replace(/class="[^"]*"\s*/, '').trim()
  const cls = m ? `${DL} ${m[1]}` : DL
  return (
    `<dialog open class="${cls}" data-sail-host="dialog" ${rest} style="position:static;margin:0;max-height:none">` +
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
      shell('class="l1-mark" size="md" backdrop="default" side="center"'),
      shell("size='sm' backdrop='default'") +
        shell("size='lg' backdrop='blur'") +
        shell("size='full' backdrop='solid'"),
      shell("side='start'") +
        shell("side='end'") +
        shell("side='top'") +
        shell("side='bottom'"),
      shell('dismissible="false" size="md"'),
    ]
  },
  probe() {
    const bad = []
    const cs = (el, p) => getComputedStyle(el)[p]
    const clear = (v) => v === 'rgba(0, 0, 0, 0)' || v === 'transparent' || v === ''
    document.querySelectorAll('dialog').forEach((el, i) => {
      const r = el.getBoundingClientRect()
      if (r.width <= 0 || r.height <= 0) return bad.push({ i, why: 'not-rendered' })
      if (cs(el, 'display') !== 'flex') bad.push({ i, why: 'open-not-flex', display: cs(el, 'display') })
      // shadow may be none on glass/flat themes — require paint surface instead
      if (clear(cs(el, 'backgroundColor'))) bad.push({ i, why: 'no-token-bg' })
      if (clear(cs(el, 'color'))) bad.push({ i, why: 'no-token-fg' })
      const head = el.querySelector('header')
      const body = el.querySelector('section')
      const foot = el.querySelector('footer')
      if (!head || !body || !foot) return bad.push({ i, why: 'missing-slot' })
      if (cs(head, 'borderBottomWidth') === '0px') bad.push({ i, why: 'header-not-divided' })
      if (cs(foot, 'borderTopWidth') === '0px') bad.push({ i, why: 'footer-not-divided' })
      if (clear(cs(body, 'color'))) bad.push({ i, why: 'body-no-token-fg' })
      if (cs(body, 'overflowY') !== 'auto') bad.push({ i, why: 'body-not-scrollable' })
    })
    const scope = "[theme='ship'] "
    const q = (s) => document.querySelector(scope + s)
    const w = (s) => q(s)?.getBoundingClientRect().width ?? 0
    if (q("dialog[size='sm']") && q('dialog.l1-mark') && !(w("dialog[size='sm']") < w('dialog.l1-mark')))
      bad.push({ why: 'size-not-monotonic', sm: w("dialog[size='sm']"), md: w('dialog.l1-mark') })
    if (q("dialog[side='end']") && cs(q("dialog[side='end']"), 'borderTopRightRadius') !== '0px')
      bad.push({ why: 'sheet-not-squared' })
    for (const side of ['start', 'end', 'top', 'bottom']) {
      if (!document.querySelector(`dialog[side='${side}']`)) bad.push({ why: `side-${side}-missing` })
    }
    for (const bd of ['blur', 'solid']) {
      if (!document.querySelector(`dialog[backdrop='${bd}']`)) bad.push({ why: `backdrop-${bd}-missing` })
    }
    const locked = [...document.querySelectorAll('dialog')].some(
      (d) => d.getAttribute('dismissible') === 'false',
    )
    if (!locked) bad.push({ why: 'dismissible-false-missing' })
    // Theme attribute rebinding (component.dialog.* often aliases globals that
    // some themes leave at :root — assert the section theme rebinds sources).
    const bg = (t) =>
      getComputedStyle(document.querySelector(`[theme='${t}']`)).getPropertyValue('--background').trim()
    if (!bg('ship') || bg('ship') === bg('console')) {
      bad.push({ why: 'frozen-token', light: bg('ship'), dark: bg('console') })
    }
    return { count: document.querySelectorAll('dialog').length, bad: bad.slice(0, 12) }
  },
  behavior: {
    rows() {
      return [
        `<button id="open" command="show-modal" commandfor="d1">Open</button>` +
        `<dialog id="d1" class="${DL}" data-sail-host="dialog"><header>Hi</header><section>Body</section>` +
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
      if (!(await isModal())) bad.push({ why: 'not-modal' })

      await page.click('#x')
      await page.waitForTimeout(60)
      if (await isOpen()) bad.push({ why: 'close-invoker-did-not-close' })

      await page.click('#open')
      await page.waitForTimeout(60)
      await page.keyboard.press('Escape')
      await page.waitForTimeout(60)
      if (await isOpen()) bad.push({ why: 'escape-did-not-close' })
      return bad
    },
  },
}
