import type { Action } from '../action'

let _seq = 0

/**
 * `use:radioGroup` — the L1 `<radio-group>` element's behavior. Native radios
 * that share a `name` already give single selection, arrow-key roving, focus,
 * and form submission for free; this action supplies the wiring CSS can't:
 *  - give every child <radio>'s inner <input> the shared group name + its own
 *    value + disabled (group [disabled] OR the radio's own), and check the one
 *    matching the group's `value`;
 *  - on a radio `change`, reflect the new value to the group's `value` attribute
 *    + a hidden <input> (form) and dispatch sail-change {value};
 *  - aria-label / aria-required from the group.
 * Tone/size are ordinary props: put them on the group (CSS custom properties
 * inherit) and/or on each radio.
 * Re-syncs when the group's attributes or its children change. Written once —
 * no per-instance controller.
 */
export const radioGroup: Action<void> = (node) => {
  const el = node as HTMLElement
  if (!el.hasAttribute('role')) el.setAttribute('role', 'radiogroup')
  const fallbackName = `sail-radio-${++_seq}`

  const hidden = document.createElement('input')
  hidden.type = 'hidden'
  el.appendChild(hidden)

  const radios = (): HTMLElement[] =>
    Array.from(el.querySelectorAll('[data-sail-host="radio"]'))
  const inputOf = (r: HTMLElement): HTMLInputElement | null => r.querySelector('input[type="radio"]')

  const sync = (): void => {
    const val = el.getAttribute('value') ?? ''
    const groupDisabled = el.hasAttribute('disabled')
    const name = el.getAttribute('name') || fallbackName
    for (const r of radios()) {
      const input = inputOf(r)
      if (!input) continue
      const rval = r.getAttribute('value') ?? ''
      input.name = name
      input.value = rval
      input.checked = rval === val
      input.disabled = groupDisabled || r.hasAttribute('disabled')
    }
    hidden.name = el.getAttribute('name') ?? ''
    hidden.value = val
    const label = el.getAttribute('label')
    if (label) el.setAttribute('aria-label', label)
    else el.removeAttribute('aria-label')
    if (el.hasAttribute('required')) el.setAttribute('aria-required', 'true')
    else el.removeAttribute('aria-required')
  }

  const onChange = (e: Event): void => {
    const input = e.target as HTMLInputElement | null
    if (!input || input.type !== 'radio') return
    const val = input.value
    if (el.getAttribute('value') === val) return
    el.setAttribute('value', val)
    sync()
    el.dispatchEvent(new CustomEvent('sail-change', { detail: { value: val }, bubbles: true, composed: true }))
  }
  el.addEventListener('change', onChange)

  sync()
  const obs = new MutationObserver(sync)
  obs.observe(el, {
    attributes: true,
    attributeFilter: ['value', 'disabled', 'name', 'label', 'required'],
    childList: true,
  })

  return {
    destroy: () => {
      el.removeEventListener('change', onChange)
      obs.disconnect()
      hidden.remove()
    },
  }
}
