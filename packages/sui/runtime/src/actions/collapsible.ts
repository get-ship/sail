import type { Action } from '../action'

/**
 * `use:collapsible` — the L1 `<collapsible>` element's behavior. The native
 * <details>/<summary> owns open/close, keyboard, focus, and ARIA; this action
 * supplies only the two things native can't:
 *  - re-emit the native `toggle` as a composed `sui-toggle` {open} so hosts can
 *    sync a bound signal;
 *  - suppress activation (click + Enter/Space) while [disabled] — native
 *    <details> has no disabled state.
 * Written once in the toolchain — no per-instance controller, compiles away.
 */
export const collapsible: Action<void> = (node) => {
  const el = node as HTMLDetailsElement

  const onToggle = (): void => {
    el.dispatchEvent(
      new CustomEvent('sui-toggle', { detail: { open: el.open }, bubbles: true, composed: true }),
    )
  }
  const onClickCapture = (e: MouseEvent): void => {
    if (el.hasAttribute('disabled')) e.preventDefault()
  }
  const onKeydownCapture = (e: KeyboardEvent): void => {
    if (el.hasAttribute('disabled') && (e.key === 'Enter' || e.key === ' ')) e.preventDefault()
  }

  el.addEventListener('toggle', onToggle)
  el.addEventListener('click', onClickCapture, true)
  el.addEventListener('keydown', onKeydownCapture, true)

  return {
    destroy: () => {
      el.removeEventListener('toggle', onToggle)
      el.removeEventListener('click', onClickCapture, true)
      el.removeEventListener('keydown', onKeydownCapture, true)
    },
  }
}
