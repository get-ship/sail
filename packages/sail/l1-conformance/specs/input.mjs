// L1 conformance spec: <input> (mechanism-2, native + INPUT_SKIN_CSS). Bare
// <input> is the design-system field; [invalid]/[aria-invalid] → destructive
// border, size via the channel. Probe: every field is bordered, invalid is a
// DISTINCT border, sizes are monotonic, and the frozen-token guard on the base
// border (light ≠ dark).

const inp = (attrs = '', val = '') => `<input ${attrs} value="${val}">`

export default {
  tag: 'input',
  width: 640,
  rows() {
    return [
      [inp('placeholder="default"', 'default'), inp('invalid', 'invalid'), inp('disabled', 'disabled'), inp('type="email"', 'you@example.com')].join(''),
      [inp('size="sm"', 'sm'), inp('size="md"', 'md'), inp('size="lg"', 'lg')].join(''),
      inp('class="l1-mark"', 'neutral'),
    ]
  },
  probe() {
    const bad = []
    const cs = (el, p) => getComputedStyle(el)[p]
    const clear = (v) => v === 'rgba(0, 0, 0, 0)' || v === 'transparent' || v === ''
    // A field must be visually distinguishable from the page — via a border OR
    // a shadow. (Some themes, e.g. halo/glass, are deliberately borderless and
    // separate fields with --shadow-xs; requiring a hard border there is wrong.)
    document.querySelectorAll('input').forEach((el) => {
      const border = cs(el, 'borderTopColor')
      const shadow = cs(el, 'boxShadow')
      if (clear(border) && (!shadow || shadow === 'none'))
        bad.push({ why: 'field-not-visible', val: el.value, border, shadow })
    })
    const scope = "[theme='ship'] "
    const q = (sel) => document.querySelector(scope + sel)
    // invalid border must be distinct from the default field border
    const defBorder = cs(q('input:not([invalid]):not([disabled])'), 'borderTopColor')
    const invBorder = cs(q('input[invalid]'), 'borderTopColor')
    if (defBorder === invBorder) bad.push({ why: 'invalid-border-not-distinct', defBorder, invBorder })
    // sizes monotonic
    const h = (sel) => q(sel).getBoundingClientRect().height
    const [sm, md, lg] = [h("input[size='sm']"), h("input[size='md']"), h("input[size='lg']")]
    if (!(sm < md && md < lg)) bad.push({ why: 'size-not-monotonic', sm, md, lg })
    // frozen-token guard
    const mark = (t) => cs(document.querySelector(`[theme='${t}'] .l1-mark`), 'borderTopColor')
    if (mark('ship') === mark('console')) bad.push({ why: 'frozen-token', light: mark('ship'), dark: mark('console') })
    return { count: document.querySelectorAll('input').length, bad: bad.slice(0, 10) }
  },
}
