import type { Action } from '../action'

let _seq = 0

/**
 * `use:describedControl` — form control ↔ hint/error association for the L1
 * `<field>` host anatomy (structure under host, not class names):
 *   children[0] label, [1] control, [2] hint, [3] error-gate → [data-field-error]
 *
 * The hint↔error visibility swap is pure CSS; this action wires accessibility:
 * stamps role=alert + aria-live=polite on the error part (closed anatomy),
 * gives the hint and error regions stable ids, then points the first slotted
 * control's aria-describedby at the active region (hint when valid, error when
 * [invalid]) and set aria-invalid to match. Re-wires when [invalid] toggles or
 * the control changes. Written once — no per-instance controller, compiles away.
 */
export const describedControl: Action<void> = (node) => {
  const el = node as HTMLElement
  const n = ++_seq
  // Structure order matches field.sail; error is the alert row inside the gate.
  const control = (el.children[1] as HTMLElement | undefined) ?? null
  const hint = (el.children[2] as HTMLElement | undefined) ?? null
  const error =
    (el.querySelector('[data-field-error]') as HTMLElement | null) ??
    (el.querySelector('[role="alert"]') as HTMLElement | null)
  if (error) {
    if (!error.hasAttribute('role')) error.setAttribute('role', 'alert')
    if (!error.hasAttribute('aria-live')) error.setAttribute('aria-live', 'polite')
  }
  const hintId = `sail-field-hint-${n}`
  const errorId = `sail-field-error-${n}`
  if (hint) hint.id = hintId
  if (error) error.id = errorId

  // switch host stamp covers non-native control shells if present.
  const CONTROL_SEL =
    'input, textarea, select, [contenteditable], [role="textbox"], [role="combobox"], [role="spinbutton"], [data-sail-host="switch"]'

  const wire = (): void => {
    const first = control?.querySelector(CONTROL_SEL) as HTMLElement | null
    if (!first) return
    const inv = el.hasAttribute('invalid')
    const targetId = inv ? errorId : hintId
    const prior = (first.getAttribute('aria-describedby') || '')
      .split(/\s+/)
      .filter((id) => id && id !== hintId && id !== errorId)
    const next = [...prior, targetId].join(' ').trim()
    if (next) first.setAttribute('aria-describedby', next)
    first.setAttribute('aria-invalid', inv ? 'true' : 'false')
  }
  wire()

  const obs = new MutationObserver(wire)
  obs.observe(el, { attributes: true, attributeFilter: ['invalid'] })
  if (control) obs.observe(control, { childList: true, subtree: true })

  return { destroy: () => obs.disconnect() }
}
