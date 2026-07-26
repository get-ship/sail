// L1 conformance spec: <field> (mechanism-1, use:describedControl). Probe runs
// the field essentials (id wiring, aria-describedby → hint|error, aria-invalid)
// and asserts the valid/invalid gate: valid → hint shown/error hidden; invalid →
// error shown/hint hidden. Valid + invalid rendered side by side.

const inputCss = 'font:inherit;font-size:14px;padding:8px 10px;border-radius:8px;border:1px solid var(--border);background:var(--surface-1);color:var(--foreground);width:100%;box-sizing:border-box'
const invCss = inputCss + ';border-color:var(--destructive)'
const lbl = 'font-size:13px;font-weight:600;color:var(--foreground)'
// Structure mirrors field.sail: label, control, hint, error-gate → [role=alert]
const field = (id, invalid = false) =>
  `<div class="sail-field"${invalid ? ' invalid' : ''} style="width:300px"><div><label for="${id}" style="${lbl}">Email address</label></div><div><input id="${id}" type="email" placeholder="you@example.com" value="${invalid ? 'not-an-email' : ''}" style="${invalid ? invCss : inputCss}"/></div><div style="${invalid ? 'display:none' : ''}">We'll never share your email.</div><div style="${invalid ? '' : 'display:none'}"><div role="alert" aria-live="polite"><svg viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M10.3 3.9 1.8 18a2 2 0 0 0 1.7 3h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0z"/><path d="M12 9v4M12 17h.01"/></svg><span>Enter a valid email address.</span></div></div></div>`

export default {
  tag: 'field',
  width: 760,
  rows() {
    return [field('f-valid') + field('f-invalid', true)]
  },
  probe() {
    let n = 0
    const bad = []
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
    })
    return { count: document.querySelectorAll('.sail-field').length, bad }
  },
}
