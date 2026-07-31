// L1 conformance spec: <textarea> (mechanism-2, native + skin). Same field
// contract as <input> — border/elevation, invalid, disabled — plus the one thing
// that's its own: it grows. `use:autosize` is the L1 behavior that makes it grow
// to fit its content instead of scrolling, and that's what the behavior block drives.

const ta = (attrs = '', val = '') => `<textarea ${attrs}>${val}</textarea>`

export default {
  tag: 'textarea',
  width: 640,
  rows() {
    return [
      [ta('class="l1-mark"', 'default'), ta('invalid', 'invalid'), ta('disabled', 'disabled')].join(''),
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
      // it takes the DESIGN font — i.e. exactly --font-body, whatever the theme
      // says that is. (Not "isn't mono": the console theme's body font IS mono,
      // and a textarea there SHOULD be mono.)
      // (normalize: computed font-family re-quotes the list differently than the
      // raw custom-property text — "SFMono-Regular" vs SFMono-Regular.)
      const norm = (v) => v.replace(/["']/g, '').replace(/\s+/g, '').toLowerCase()
      const want = getComputedStyle(el).getPropertyValue('--font-body')
      if (want && norm(cs(el, 'fontFamily')) !== norm(want))
        bad.push({ i, why: 'not-design-font', got: cs(el, 'fontFamily'), want })
    })

    const scope = "[data-theme-preview='ship'] "
    const q = (s) => document.querySelector(scope + s)
    if (cs(q('textarea.l1-mark'), 'borderTopColor') === cs(q('textarea[invalid]'), 'borderTopColor'))
      bad.push({ why: 'invalid-not-distinct' })
    // rows must actually size it. This is the border-box assertion in disguise:
    // under content-box the min-height floor is 68px of CONTENT, which swallows
    // four rows whole and makes `rows` a no-op.
    const base = q('textarea.l1-mark').getBoundingClientRect().height
    const four = q('textarea[rows]').getBoundingClientRect().height
    if (!(base < four)) bad.push({ why: 'rows-not-honored', base, four })
    if (cs(q('textarea.l1-mark'), 'boxSizing') !== 'border-box')
      bad.push({ why: 'not-border-box' })
    const mark = (t) => cs(document.querySelector(`[data-theme-preview='${t}'] .l1-mark`), 'backgroundColor')
    if (mark('ship') === mark('console')) bad.push({ why: 'frozen-token', light: mark('ship') })

    return { count: document.querySelectorAll('textarea').length, bad: bad.slice(0, 10) }
  },
  behavior: {
    rows() { return [`<textarea id="ta" rows="2" style="width:400px"></textarea>`] },
    attach: [['#ta', 'autosize']],
    async run(page) {
      const bad = []
      const h = () => page.evaluate(() => document.getElementById('ta').getBoundingClientRect().height)
      const start = await h()
      // typing past the visible rows must GROW the box, not scroll it
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
      // and SHRINK back when the content goes away
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
