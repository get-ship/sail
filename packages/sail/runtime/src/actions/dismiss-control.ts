// dismissControl — click on a close control dismisses the host.
//
// Capability mix-in: a click on a descendant matching `[data-toast-close]`
// (or `[data-dismiss]`) runs the same dismiss path as autoDismiss —
// `sail-toast-dismiss` + remove. Stacked with `use:autoDismiss` on the L1
// <toast> root. Idempotent with the timer path via dismiss-self.

import type { Action } from '../action'
import { dismissSelf, isDismissed } from './dismiss-self'

const CLOSE_SEL = '[data-toast-close], [data-dismiss]'

/**
 * Click on a descendant close control (`[data-toast-close]` or
 * `[data-dismiss]`) dismisses the host: composed `sail-toast-dismiss`, then
 * `el.remove()`.
 */
export const dismissControl: Action<void> = (node) => {
  const el = node as HTMLElement

  const onClick = (e: MouseEvent): void => {
    if (isDismissed(el)) return
    const close = (e.target as Element | null)?.closest(CLOSE_SEL)
    if (close && el.contains(close)) dismissSelf(el)
  }

  el.addEventListener('click', onClick)

  return {
    destroy: () => {
      el.removeEventListener('click', onClick)
    },
  }
}
