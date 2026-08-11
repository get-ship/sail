// L1 conformance spec: <textarea> (mechanism-2, native + freeform Style stamps).
// Same field contract as <input> — border/elevation, invalid, disabled, size —
// plus rows and use:autosize behavior.

const TA =
  's-displayblock s-minHcontrolhmd s-padY15 s-padX3 s-wfull s-fgfieldforeground s-bgfield s-borderWidth1 s-borderfieldborder s-radiusmd s-shadowxs s-fontSizebase s-hover-borderborderstrong s-disabled-cursornotallowed s-disabled-opacity50 s-sizexs-minHcontrolh2xs s-sizexs-padX2 s-sizexs-fontSizexs s-sizesm-minHcontrolhsm s-sizesm-padX25 s-sizesm-fontSizesm s-sizemd-minHcontrolhmd s-sizemd-padX3 s-sizemd-fontSizebase s-sizelg-minHcontrolhlg s-sizelg-padX4 s-sizelg-fontSizemd s-invalidtrue-borderdestructive'

const ta = (attrs = '', val = '') => {
  const m = attrs.match(/class="([^"]*)"/)
  const rest = attrs.replace(/class="[^"]*"\s*/, '').trim()
  const cls = m ? `${TA} ${m[1]}` : TA
  return `<textarea class="${cls}" ${rest}>${val}</textarea>`
}

export default {
  tag: 'textarea',
  width: 720,
  rows() {
    return [
      // rest-default + invalid + disabled
      [ta('class="l1-mark"', 'default'), ta('invalid', 'invalid'), ta('disabled', 'disabled')].join(
        '',
      ),
      // size ladder
      [
        ta('size="xs" rows="2"', 'xs'),
        ta('size="sm" rows="2"', 'sm'),
        ta('size="md" rows="2"', 'md'),
        ta('size="lg" rows="2"', 'lg'),
      ].join(''),
      // rows must size height (border-box)
      ta('rows="4" style="width:100%"', 'four rows'),
    ]
  },
  probe() {
    const bad = []
    const cs = (el, p) => getComputedStyle(el)[p]
    const clear = (v) => v === 'rgba(0, 0, 0, 0)' || v === 'transparent' || v === ''

    document.querySelectorAll('textarea').forEach((el, i) => {
      const r = el.getBoundingClientRect()
      if (r.width <= 0 || r.height <= 0) return bad.push({ i, why: 'not-rendered' })
      if (clear(cs(el, 'borderTopColor')) && cs(el, 'boxShadow') === 'none')
        bad.push({ i, why: 'field-not-visible' })
      // design font — normalize quotes/whitespace
      const norm = (v) => v.replace(/["']/g, '').replace(/\s+/g, '').toLowerCase()
      const want = getComputedStyle(el).getPropertyValue('--font-body')
      if (want && norm(cs(el, 'fontFamily')) !== norm(want))
        bad.push({ i, why: 'not-design-font', got: cs(el, 'fontFamily'), want })
    })

    const scope = "[theme='ship'] "
    const q = (s) => document.querySelector(scope + s)
    if (cs(q('textarea.l1-mark'), 'borderTopColor') === cs(q('textarea[invalid]'), 'borderTopColor'))
      bad.push({ why: 'invalid-not-distinct' })
    // size monotonic
    const h = (s) => q(s).getBoundingClientRect().height
    const [xs, sm, md, lg] = [
      h('textarea[size="xs"]'),
      h('textarea[size="sm"]'),
      h('textarea[size="md"]'),
      h('textarea[size="lg"]'),
    ]
    if (!(xs <= sm && sm <= md && md <= lg)) bad.push({ why: 'size-not-monotonic', xs, sm, md, lg })
    // rows must actually size it
    const base = q('textarea.l1-mark').getBoundingClientRect().height
    const four = q('textarea[rows="4"]').getBoundingClientRect().height
    if (!(base < four)) bad.push({ why: 'rows-not-honored', base, four })
    if (cs(q('textarea.l1-mark'), 'boxSizing') !== 'border-box')
      bad.push({ why: 'not-border-box' })
    const mark = (t) => cs(document.querySelector(`[theme='${t}'] .l1-mark`), 'backgroundColor')
    if (mark('ship') === mark('console')) bad.push({ why: 'frozen-token', light: mark('ship') })

    return { count: document.querySelectorAll('textarea').length, bad: bad.slice(0, 10) }
  },
  behavior: {
    rows() {
      return [`<textarea id="ta" rows="2" style="width:400px"></textarea>`]
    },
    attach: [['#ta', 'autosize']],
    async run(page) {
      const bad = []
      const h = () => page.evaluate(() => document.getElementById('ta').getBoundingClientRect().height)
      const start = await h()
      await page.click('#ta')
      await page.keyboard.type('one\ntwo\nthree\nfour\nfive\nsix')
      await page.waitForTimeout(80)
      const grown = await h()
      if (!(grown > start)) bad.push({ why: 'autosize-did-not-grow', start, grown })
      const scrolls = await page.evaluate(() => {
        const el = document.getElementById('ta')
        return el.scrollHeight - el.clientHeight > 2
      })
      if (scrolls) bad.push({ why: 'autosize-still-scrolls' })
      await page.evaluate(() => {
        const el = document.getElementById('ta')
        el.value = ''
        el.dispatchEvent(new Event('input', { bubbles: true }))
      })
      await page.waitForTimeout(80)
      const shrunk = await h()
      if (!(shrunk < grown)) bad.push({ why: 'autosize-did-not-shrink', grown, shrunk })
      return bad
    },
  },
}
