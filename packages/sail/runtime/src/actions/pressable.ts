// pressable — button semantics on a non-button host. `use:pressable`
//
// One of the three behaviors decomposed from the L1 `<chip>` element
// controller (pressable / pressed / dismissible); stack them with `use:`
// stamps instead of a monolithic controller. Extracted verbatim from
// `element-controllers/chip.ts` — parity is the contract.

import type { Action } from '../action'

/**
 * Make a non-button host behave like a button:
 *
 * - `role="button"` when the host has no role of its own;
 * - `[disabled]` reflected to `aria-disabled="true"` + `tabindex="-1"`
 *   (removed / restored to `"0"` when re-enabled) via a MutationObserver;
 * - `Enter` / `Space` keydown activates the host (`preventDefault()` +
 *   `el.click()`), suppressed while disabled.
 *
 * A real `<button>` host needs nothing here — the platform already does all
 * of this. This action is for non-button hosts (spans, divs, custom
 * elements) that must be focusable and keyboard-activatable. Pairs with
 * `use:pressed` for toggle-button hosts.
 */
export const pressable: Action<void> = (node) => {
  const el = node as HTMLElement

  if (!el.hasAttribute('role')) el.setAttribute('role', 'button')

  const sync = (): void => {
    if (el.hasAttribute('disabled')) {
      el.setAttribute('aria-disabled', 'true')
      el.setAttribute('tabindex', '-1')
    } else {
      el.removeAttribute('aria-disabled')
      if (el.getAttribute('tabindex') == null || el.getAttribute('tabindex') === '-1') {
        el.setAttribute('tabindex', '0')
      }
    }
  }
  sync()
  const obs = new MutationObserver(sync)
  obs.observe(el, { attributes: true, attributeFilter: ['disabled'] })

  const onKeydown = (e: KeyboardEvent): void => {
    if (el.hasAttribute('disabled')) return
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault()
      el.click()
    }
  }
  el.addEventListener('keydown', onKeydown)

  return {
    destroy: () => {
      obs.disconnect()
      el.removeEventListener('keydown', onKeydown)
    },
  }
}
