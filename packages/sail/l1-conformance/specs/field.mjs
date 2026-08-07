// L1 conformance spec: <field> (mechanism-1, use:describedControl). Probe runs
// the field essentials (id wiring, aria-describedby → hint|error, aria-invalid)
// and asserts the valid/invalid gate: valid → hint shown/error hidden; invalid →
// error shown/hint hidden. Paint classes mirror lowered Style: field-hint /
// field-error (not free muted-foreground / destructive).

// Control surface stamps component.field (shared with input/select/textarea).
const INPUT =
  'sx-displayblock sx-minHcontrolhmd sx-padY15 sx-padX3 sx-wfull sx-fgfieldforeground sx-bgfield sx-borderWidth1 sx-borderfieldborder sx-radiusmd sx-shadowxs sx-fontSizebase sx-hover-borderfieldborderhover sx-disabled-cursornotallowed sx-disabled-opacity50'
const INPUT_INV = `${INPUT} sx-invalidtrue-borderdestructive`
const HINT = 'sx-mt1 sx-fgfieldhint sx-fontSizexs sx-leadingsnug'
const ERROR = 'sx-mt1 sx-fgfielderror sx-fontSizexs sx-leadingsnug sx-fontWeightmedium'
const LBL = 'sx-fontFamilybody sx-fontWeightmedium sx-fontSizesm'

// Structure mirrors field.sail: label, control, hint, error-gate → [role=alert]
const field = (id, invalid = false) =>
  `<div class="sail-field sail-grp sx-fontFamilybody"${invalid ? ' invalid' : ''} style="width:300px">` +
  `<div><label for="${id}" class="${LBL}">Email address</label></div>` +
  `<div><input id="${id}" class="${invalid ? INPUT_INV : INPUT}" type="email" placeholder="you@example.com" value="${invalid ? 'not-an-email' : ''}"${invalid ? ' invalid' : ''}/></div>` +
  `<div class="${HINT}"${invalid ? ' style="display:none"' : ''}>We'll never share your email.</div>` +
  `<div${invalid ? '' : ' style="display:none"'}><div role="alert" aria-live="polite" class="${ERROR}">` +
  `<svg viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" width="16" height="16"><path d="M10.3 3.9 1.8 18a2 2 0 0 0 1.7 3h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0 -3.4 0z"/><path d="M12 9v4M12 17h.01"/></svg>` +
  `<span>Enter a valid email address.</span></div></div></div>`

export default {
  tag: 'field',
  width: 760,
  rows() {
    return [field('f-valid') + field('f-invalid', true)]
  },
  probe() {
    let n = 0
    const bad = []
    const cs = (el, p) => getComputedStyle(el)[p]
    const clear = (v) => v === 'rgba(0, 0, 0, 0)' || v === 'transparent' || v === ''
    document.querySelectorAll('.sail-field').forEach((el) => {
      const m = ++n
      const control = el.children[1]
      const hint = el.children[2]
      const error = el.querySelector('[role="alert"]')
      const hintId = `sail-field-hint-${m}`
      const errorId = `sail-field-error-${m}`
      if (hint) hint.id = hintId
      if (error) error.id = errorId
      const first = control?.querySelector('input')
      if (!first || !hint || !error) {
        bad.push({ missing: true })
        return
      }
      const inv = el.hasAttribute('invalid')
      first.setAttribute('aria-describedby', inv ? errorId : hintId)
      first.setAttribute('aria-invalid', inv ? 'true' : 'false')
      const hintShown = getComputedStyle(hint).display !== 'none'
      const errShown = getComputedStyle(error).display !== 'none' && getComputedStyle(error.parentElement).display !== 'none'
      const describedbyOk = first.getAttribute('aria-describedby') === (inv ? errorId : hintId)
      const invalidOk = first.getAttribute('aria-invalid') === (inv ? 'true' : 'false')
      if (hintShown !== !inv || errShown !== inv || !describedbyOk || !invalidOk)
        bad.push({ inv, hintShown, errShown, describedbyOk, invalidOk })
      // Token-backed control surface + hint/error ink.
      if (clear(cs(first, 'backgroundColor'))) bad.push({ inv, why: 'no-token-field-bg' })
      if (clear(cs(first, 'color'))) bad.push({ inv, why: 'no-token-field-fg' })
      if (!inv && hintShown && clear(cs(hint, 'color'))) bad.push({ inv, why: 'no-token-hint-fg' })
      if (inv && errShown && clear(cs(error, 'color'))) bad.push({ inv, why: 'no-token-error-fg' })
    })
    // Invalid control border distinct from resting field border.
    const scope = "[theme='ship'] "
    const q = (s) => document.querySelector(scope + s)
    const base = q('.sail-field:not([invalid]) input')
    const invEl = q('.sail-field[invalid] input')
    if (base && invEl && cs(base, 'borderTopColor') === cs(invEl, 'borderTopColor'))
      bad.push({ why: 'invalid-border-not-distinct' })
    return { count: document.querySelectorAll('.sail-field').length, bad }
  },
}
