// L1 conformance spec: <input> (mechanism-2, native + freeform Style stamps).
// Text-like fields share component.field.* with select/textarea. type gates skins;
// probe asserts field visibility, invalid border, size monotonicity, type coverage.
// Harness stamps freeform utility classes from element-skins (product CSS).

const FIELD =
  's-displayblock s-minHcontrolhmd s-padY15 s-padX3 s-wfull s-fgfieldforeground s-bgfield s-borderWidth1 s-borderfieldborder s-radiusmd s-shadowxs s-fontSizebase s-hover-borderborderstrong s-disabled-cursornotallowed s-disabled-opacity50 s-sizexs-minHcontrolh2xs s-sizexs-padX2 s-sizexs-fontSizexs s-sizesm-minHcontrolhsm s-sizesm-padX25 s-sizesm-fontSizesm s-sizemd-minHcontrolhmd s-sizemd-padX3 s-sizemd-fontSizebase s-sizelg-minHcontrolhlg s-sizelg-padX4 s-sizelg-fontSizemd s-invalidtrue-borderdestructive'

const CHECK =
  's-displayinlineblock s-m0 s-pad0 s-minH0 s-wiconsizesm s-hiconsizesm s-appearancenone s-alignmiddle s-borderWidth2 s-bordersolid s-borderfieldforeground s-bgfield s-shadowxs s-radiussm s-cursorpointer s-fontSizebase s-invalidtrue-borderdestructive'

const RANGE =
  's-wfull s-h5 s-appearancenone s-bgtransparent s-cursorpointer s-disabled-cursornotallowed s-disabled-opacity50'

/** Types that use the shared field surface (or close enough for paint probe). */
const FIELD_TYPES = [
  'text',
  'email',
  'password',
  'number',
  'search',
  'tel',
  'url',
  'hidden',
]

const ALL_TYPES = [
  ...FIELD_TYPES,
]

const fieldInp = (attrs = '', val = '') => {
  const m = attrs.match(/class="([^"]*)"/)
  const rest = attrs.replace(/class="[^"]*"\s*/, '').trim()
  const cls = m ? `${FIELD} ${m[1]}` : FIELD
  const v = val !== '' ? ` value="${val}"` : ''
  return `<input class="${cls}" ${rest}${v}>`
}

const typeCell = (type) => {
  if (type === 'hidden') {
    return `<input type="hidden" value="token" data-type-cov="hidden">`
  }
  return fieldInp(`type="${type}"`, type === 'email' ? 'you@example.com' : type)
}

export default {
  tag: 'input',
  width: 900,
  rows() {
    return [
      // rest-default + invalid + disabled + email sample
      [
        fieldInp('class="l1-mark" placeholder="default"', 'default'),
        fieldInp('invalid', 'invalid'),
        fieldInp('disabled', 'disabled'),
        fieldInp('type="email"', 'you@example.com'),
      ].join(''),
      // size ladder (xs · sm · md · lg)
      [
        fieldInp('size="xs"', 'xs'),
        fieldInp('size="sm"', 'sm'),
        fieldInp('size="md"', 'md'),
        fieldInp('size="lg"', 'lg'),
      ].join(''),
      // type matrix — every registry type= value for visual.cross
      ALL_TYPES.map((t) => typeCell(t)).join(''),
    ]
  },
  probe() {
    // probe runs in-page — no module closure; keep constants local
    const types = [
      'text',
      'email',
      'password',
      'number',
      'search',
      'tel',
      'url',
      'hidden',
    ]
    const bad = []
    const cs = (el, p) => getComputedStyle(el)[p]
    const clear = (v) => v === 'rgba(0, 0, 0, 0)' || v === 'transparent' || v === ''

    document.querySelectorAll('input').forEach((el) => {
      if (el.type === 'hidden') return
      if (el.type === 'range') {
        const r = el.getBoundingClientRect()
        if (r.width <= 0) bad.push({ why: 'range-not-rendered' })
        return
      }
      // checkbox/radio may use type-gated skins; still require a box
      if (el.type === 'checkbox' || el.type === 'radio') {
        const r = el.getBoundingClientRect()
        if (r.width <= 0 || r.height <= 0) bad.push({ why: 'check-not-rendered', type: el.type })
        return
      }
      const border = cs(el, 'borderTopColor')
      const shadow = cs(el, 'boxShadow')
      if (clear(border) && (!shadow || shadow === 'none'))
        bad.push({ why: 'field-not-visible', type: el.type, val: el.value, border, shadow })
    })

    const scope = "[theme='ship'] "
    const q = (sel) => document.querySelector(scope + sel)
    // invalid border must be distinct from the default field border
    const defBorder = cs(q('input.l1-mark'), 'borderTopColor')
    const invBorder = cs(q('input[invalid]'), 'borderTopColor')
    if (defBorder === invBorder) bad.push({ why: 'invalid-border-not-distinct', defBorder, invBorder })
    // sizes monotonic (xs < sm < md < lg)
    const h = (sel) => q(sel).getBoundingClientRect().height
    const [xs, sm, md, lg] = [
      h('input[size="xs"]'),
      h('input[size="sm"]'),
      h('input[size="md"]'),
      h('input[size="lg"]'),
    ]
    if (!(xs < sm && sm < md && md < lg)) bad.push({ why: 'size-not-monotonic', xs, sm, md, lg })
    // type coverage present
    for (const t of types) {
      if (!document.querySelector(`input[type="${t}"]`)) bad.push({ why: 'missing-type', type: t })
    }
    // frozen-token guard
    const mark = (t) => cs(document.querySelector(`[theme='${t}'] .l1-mark`), 'borderTopColor')
    if (mark('ship') === mark('console'))
      bad.push({ why: 'frozen-token', light: mark('ship'), dark: mark('console') })
    return { count: document.querySelectorAll('input').length, bad: bad.slice(0, 12) }
  },
}
