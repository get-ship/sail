// detailsToggle — re-emit native <details> toggle as composed sail-toggle.
//
// Capability mix-in: any host that IS a <details> (or behaves as one) can
// stamp `use:detailsToggle` so bound signals can sync on open flips.
// Stacked with `use:detailsDisabled` on the L1 <collapsible> root.

import type { Action } from '../action'

/**
 * Re-emit the native `toggle` event as a composed `sail-toggle` with
 * `{ open: el.open }` so hosts can sync a bound signal. The browser owns
 * open/close, keyboard, focus, and ARIA — this only bridges the event.
 */
export const detailsToggle: Action<void> = (node) => {
  const el = node as HTMLDetailsElement

  const onToggle = (): void => {
    el.dispatchEvent(
      new CustomEvent('sail-toggle', { detail: { open: el.open }, bubbles: true, composed: true }),
    )
  }

  el.addEventListener('toggle', onToggle)

  return {
    destroy: () => {
      el.removeEventListener('toggle', onToggle)
    },
  }
}
