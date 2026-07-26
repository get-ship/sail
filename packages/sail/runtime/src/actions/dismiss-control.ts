// dismissControl — click on a close control dismisses the host.
//
// Capability mix-in: a click on a descendant matching `[data-toast-close]`,
// `[data-dismiss]`, or `button[aria-label="Dismiss"]` runs the shared
// dismiss-self path (event + remove). Stacked with `use:autoDismiss` on L1
// <toast> and with `use:liveRegion` on L1 <alert>. Idempotent with the timer
// path via dismiss-self.
//
// Event name: toast hosts (`duration` attr or compiler host stamp
// `.sail-toast`) emit `sail-toast-dismiss`; everything else emits
// `sail-dismiss`. Gate: non-toast hosts require `[dismissible]` (alert);
// toast close always works.

import type { Action } from '../action'
import { dismissSelf, isDismissed } from './dismiss-self'

const CLOSE_SEL = '[data-toast-close], [data-dismiss], button[aria-label="Dismiss"]'

function isToastHost(el: HTMLElement): boolean {
  // duration attr is the toast prop; .sail-toast is the compiler host stamp.
  return el.hasAttribute('duration') || el.classList.contains('sail-toast')
}

/**
 * Click on a descendant close control dismisses the host: composed dismiss
 * event, then `el.remove()`. Toast → `sail-toast-dismiss` (no dismissible
 * gate). Alert / other → `sail-dismiss`, only while `[dismissible]`.
 */
export const dismissControl: Action<void> = (node) => {
  const el = node as HTMLElement

  const onClick = (e: MouseEvent): void => {
    if (isDismissed(el)) return
    const close = (e.target as Element | null)?.closest(CLOSE_SEL)
    if (!close || !el.contains(close)) return

    const toast = isToastHost(el)
    if (!toast && !el.hasAttribute('dismissible')) return

    dismissSelf(el, toast ? 'sail-toast-dismiss' : 'sail-dismiss')
  }

  el.addEventListener('click', onClick)

  return {
    destroy: () => {
      el.removeEventListener('click', onClick)
    },
  }
}
