// detailsToggle — invoke onToggle when native <details> toggles.
//
// Capability mix-in: any host that IS a <details> (or behaves as one) can
// stamp `use:detailsToggle` so bound signals can sync on open flips.
// Stacked with `use:detailsDisabled` on the L1 <accordion> root.

import type { Action } from '../action'
import { invokeCallback } from '../callback'

/**
 * On native `toggle`, invoke `onToggle` with `{ open: el.open }` so hosts
 * can sync a bound signal. The browser owns open/close, keyboard, focus,
 * and ARIA — this only bridges the moment.
 */
export const detailsToggle: Action<void> = (node) => {
  const el = node as HTMLDetailsElement

  const onToggle = (): void => {
    invokeCallback(el, 'onToggle', { open: el.open })
  }

  el.addEventListener('toggle', onToggle)

  return {
    destroy: () => {
      el.removeEventListener('toggle', onToggle)
    },
  }
}
