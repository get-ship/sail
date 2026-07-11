import type { Action } from '../action'

let _seq = 0

/**
 * `use:field` — the L1 `<field>` element's behavior. The vertical stack and the
 * hint↔error swap are pure CSS; this action wires accessibility: give the hint
 * and error regions stable ids, then point the first slotted control's
 * aria-describedby at the active region (hint when valid, error when [invalid])
 * and set aria-invalid to match. Re-wires when [invalid] toggles or the control
 * changes. Written once — no per-instance controller, compiles away.
 */
export const field: Action<void> = (node) => {
  const el = node as HTMLElement
  const n = ++_seq
  const hint = el.querySelector('.hint') as HTMLElement | null
  const error = el.querySelector('.error') as HTMLElement | null
  const control = el.querySelector('.control') as HTMLElement | null
  const hintId = `sui-field-hint-${n}`
  const errorId = `sui-field-error-${n}`
  if (hint) hint.id = hintId
  if (error) error.id = errorId

  const CONTROL_SEL = 'input, textarea, select, [contenteditable], [role="textbox"], [role="combobox"], [role="spinbutton"], .sui-switch'

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
