// L1 conformance spec: <select> (mechanism-2, native + skin, zero JS).
// Markup mirrors lowered Style stamp: sail-select + component.field rest atoms.
// size xs|sm|md|lg · invalid · disabled · multiple.

const SEL =
  'sail-select s-displayblock s-minHcontrolhmd s-padY15 s-padS3 s-padE8 s-wfull s-fgfieldforeground s-bgfield s-borderWidth1 s-borderfieldborder s-radiusmd s-shadowxs s-cursorpointer s-fontSizebase s-hover-borderborderstrong s-disabled-cursornotallowed s-disabled-opacity50 s-sizexs-minHcontrolh2xs s-sizexs-padS2 s-sizexs-fontSizexs s-sizesm-minHcontrolhsm s-sizesm-padS25 s-sizesm-fontSizesm s-sizemd-minHcontrolhmd s-sizemd-padS3 s-sizemd-fontSizebase s-sizelg-minHcontrolhlg s-sizelg-padS4 s-sizelg-fontSizemd s-invalidtrue-borderdestructive'

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
  width: 720,
  rows() {
    return [
      // rest-default + disabled + invalid + multiple
      [sel('class="l1-mark"'), sel('disabled'), sel('invalid'), sel('multiple')].join(''),
      // size ladder (xs · sm · md · lg)
      [sel("size='xs'"), sel("size='sm'"), sel("size='md'"), sel("size='lg'")].join(''),
    ]
  },
  probe() {
    const bad = []
    const cs = (el, p) => getComputedStyle(el)[p]
    const clear = (v) => v === 'rgba(0, 0, 0, 0)' || v === 'transparent' || v === ''

    document.querySelectorAll('select').forEach((el, i) => {
      const r = el.getBoundingClientRect()
      if (r.width <= 0 || r.height <= 0) return bad.push({ i, why: 'not-rendered' })
      if (clear(cs(el, 'borderTopColor')) && cs(el, 'boxShadow') === 'none')
        bad.push({ i, why: 'field-not-visible' })
      if (el.options.length < 3) bad.push({ i, why: 'options-missing' })
      if (clear(cs(el, 'backgroundColor'))) bad.push({ i, why: 'no-token-bg' })
      if (clear(cs(el, 'color'))) bad.push({ i, why: 'no-token-fg' })
    })

    const scope = "[theme='ship'] "
    const q = (s) => document.querySelector(scope + s)
    const base = q('select.l1-mark')
    const dis = q('select[disabled]')
    const inv = q('select[invalid]')
    if (
      cs(base, 'opacity') === cs(dis, 'opacity') &&
      cs(base, 'backgroundColor') === cs(dis, 'backgroundColor')
    )
      bad.push({ why: 'disabled-not-distinct' })
    if (cs(base, 'borderTopColor') === cs(inv, 'borderTopColor'))
      bad.push({ why: 'invalid-not-distinct' })
    if (!document.querySelector('select[multiple]')) bad.push({ why: 'missing-multiple' })
    const h = (s) => q(s).getBoundingClientRect().height
    const [xs, sm, md, lg] = [
      h("select[size='xs']"),
      h("select[size='sm']"),
      h("select[size='md']"),
      h("select[size='lg']"),
    ]
    if (!(xs < sm && sm < md && md < lg)) bad.push({ why: 'size-not-monotonic', xs, sm, md, lg })
    // Chevron room: padE(8) on base must survive every size step
    const padE = (s) => parseFloat(cs(q(s), 'paddingInlineEnd')) || 0
    for (const s of [
      "select[size='xs']",
      "select[size='sm']",
      "select[size='md']",
      "select[size='lg']",
      'select.l1-mark',
    ]) {
      const pe = padE(s)
      if (pe < 30) bad.push({ why: 'chevron-padE-clobbered', sel: s, padE: pe })
    }
    const mark = (t) => cs(document.querySelector(`[theme='${t}'] .l1-mark`), 'backgroundColor')
    if (mark('ship') === mark('console')) bad.push({ why: 'frozen-token', light: mark('ship') })

    return { count: document.querySelectorAll('select').length, bad: bad.slice(0, 10) }
  },
}
