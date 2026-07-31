// L1 conformance spec: <select> (mechanism-2, native + skin, zero JS). The native
// picker IS the L1 select — it gets the platform's keyboard, typeahead, mobile
// sheet and form participation for free. A styled listbox with arbitrary item
// content is a <combobox>, a different primitive; this is not it.
// Markup mirrors freeform host stamp: sail-select + Style field classes.

const SEL =
  'sail-select sx-displayblock sx-minHcontrolhmd sx-padY15 sx-padX3 sx-fontSizebase sx-fgforeground sx-bgbackground sx-borderWidth1 sx-borderborder sx-radiusmd sx-shadowxs sx-wfull sx-cursorpointer sx-hover-borderborderstrong sx-disabled-cursornotallowed sx-disabled-opacity50 sx-sizexs-minHcontrolh2xs sx-sizexs-padX2 sx-sizexs-fontSizexs sx-sizesm-minHcontrolhsm sx-sizesm-padX25 sx-sizesm-fontSizesm sx-sizemd-minHcontrolhmd sx-sizemd-padX3 sx-sizemd-fontSizebase sx-sizelg-minHcontrolhlg sx-sizelg-padX4 sx-sizelg-fontSizemd sx-invalidtrue-borderdestructive'

const sel = (attrs = '') => {
  const m = attrs.match(/class="([^"]*)"/)
  const rest = attrs.replace(/class="[^"]*"\s*/, '').trim()
  const cls = m ? `${SEL} ${m[1]}` : SEL
  return (
    `<select class="${cls}" ${rest}><optgroup label="Team"><option>Ada</option><option>Grace</option></optgroup>` +
    `<option>Alan</option></select>`
  )
}

export default {
  tag: 'select',
  width: 640,
  rows() {
    return [
      [sel('class="l1-mark"'), sel('disabled'), sel('invalid')].join(''),
      [sel("size='sm'"), sel("size='md'"), sel("size='lg'")].join(''),
    ]
  },
  probe() {
    const bad = []
    const cs = (el, p) => getComputedStyle(el)[p]
    const clear = (v) => v === 'rgba(0, 0, 0, 0)' || v === 'transparent' || v === ''

    document.querySelectorAll('select').forEach((el, i) => {
      const r = el.getBoundingClientRect()
      if (r.width <= 0 || r.height <= 0) return bad.push({ i, why: 'not-rendered' })
      // same rule as <input>: a field is visible via a border OR an elevation
      // (halo/glass are deliberately borderless)
      if (clear(cs(el, 'borderTopColor')) && cs(el, 'boxShadow') === 'none')
        bad.push({ i, why: 'field-not-visible' })
      if (el.options.length < 3) bad.push({ i, why: 'options-missing' })
    })

    const scope = "[data-theme-preview='ship'] "
    const q = (s) => document.querySelector(scope + s)
    // disabled + invalid are each distinct from the resting field
    const base = q('select.l1-mark')
    const dis = q('select[disabled]')
    const inv = q('select[invalid]')
    if (cs(base, 'opacity') === cs(dis, 'opacity') && cs(base, 'backgroundColor') === cs(dis, 'backgroundColor'))
      bad.push({ why: 'disabled-not-distinct' })
    if (cs(base, 'borderTopColor') === cs(inv, 'borderTopColor'))
      bad.push({ why: 'invalid-not-distinct' })
    // the size channel is the SAME channel <input> uses — monotonic
    const h = (s) => q(s).getBoundingClientRect().height
    const [sm, md, lg] = [h("select[size='sm']"), h("select[size='md']"), h("select[size='lg']")]
    if (!(sm < md && md < lg)) bad.push({ why: 'size-not-monotonic', sm, md, lg })
    // frozen-token guard
    const mark = (t) => cs(document.querySelector(`[data-theme-preview='${t}'] .l1-mark`), 'backgroundColor')
    if (mark('ship') === mark('console')) bad.push({ why: 'frozen-token', light: mark('ship') })

    return { count: document.querySelectorAll('select').length, bad: bad.slice(0, 10) }
  },
}
