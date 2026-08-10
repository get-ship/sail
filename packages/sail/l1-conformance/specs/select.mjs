// L1 conformance spec: <select> (mechanism-2, native + skin, zero JS). The native
// picker IS the L1 select — it gets the platform's keyboard, typeahead, mobile
// sheet and form participation for free. A styled listbox with arbitrary item
// content is a <combobox>, a different primitive; this is not it.
// Markup mirrors lowered Style stamp: sail-select + component.field rest atoms
// (field / field-foreground / field-border). Hover spends free border-strong
// (SPEC-D1 rest-only; field-border-hover is storage residual, not PE rest).

const SEL =
  'sail-select s-displayblock s-minHcontrolhmd s-padY15 s-padS3 s-padE8 s-wfull s-fgfieldforeground s-bgfield s-borderWidth1 s-borderfieldborder s-radiusmd s-shadowxs s-cursorpointer s-fontSizebase s-hover-borderfieldborderhover s-disabled-cursornotallowed s-disabled-opacity50 s-sizexs-minHcontrolh2xs s-sizexs-padS2 s-sizexs-fontSizexs s-sizesm-minHcontrolhsm s-sizesm-padS25 s-sizesm-fontSizesm s-sizemd-minHcontrolhmd s-sizemd-padS3 s-sizemd-fontSizebase s-sizelg-minHcontrolhlg s-sizelg-padS4 s-sizelg-fontSizemd s-invalidtrue-borderdestructive'

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
      // Token-backed field surface (field / field-foreground / field-border).
      if (clear(cs(el, 'backgroundColor'))) bad.push({ i, why: 'no-token-bg' })
      if (clear(cs(el, 'color'))) bad.push({ i, why: 'no-token-fg' })
    })

    const scope = "[theme='ship'] "
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
    // Chevron room: padE(8) on base must survive every size step (not clobbered by padX).
    const padE = (s) => parseFloat(cs(q(s), 'paddingInlineEnd')) || 0
    for (const s of ["select[size='sm']", "select[size='md']", "select[size='lg']", 'select.l1-mark']) {
      const pe = padE(s)
      // --space-8 is 32px in the token sheet; allow minor UA rounding.
      if (pe < 30) bad.push({ why: 'chevron-padE-clobbered', sel: s, padE: pe })
    }
    // frozen-token guard
    const mark = (t) => cs(document.querySelector(`[theme='${t}'] .l1-mark`), 'backgroundColor')
    if (mark('ship') === mark('console')) bad.push({ why: 'frozen-token', light: mark('ship') })

    return { count: document.querySelectorAll('select').length, bad: bad.slice(0, 10) }
  },
}
