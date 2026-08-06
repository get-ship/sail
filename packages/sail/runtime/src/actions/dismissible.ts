// dismissible — removable host invokes `onRemove`. `use:dismissible`
//
// One of the three behaviors the L1 `<chip>` element stacks with `use:`
// stamps (pressable / pressed / dismissible) instead of a monolithic
// controller. Behavior parity with the retired chip controller is the
// contract.

import type { Action } from '../action'
import { invokeCallback } from '../callback'

/**
 * Invoke `onRemove` from a removable host:
 *
 * - `Backspace` / `Delete` keydown fires it, but only while the host carries
 *   `[removable]` — the attribute is the gate because L1 templates are
 *   static: stamps are unconditional, so the condition lives here. The key
 *   is `preventDefault()`ed so it doesn't navigate history or delete text.
 * - A click on the remove control (`button[aria-label="Remove"]`) fires it
 *   too, with `stopPropagation()` so the remove gesture isn't also treated as
 *   host activation.
 *
 * Both paths are suppressed while the host is `[disabled]`. Consumers set
 * `on-remove={fn}` and own the actual removal — the action only signals.
 */
export const dismissible: Action<void> = (node) => {
  const el = node as HTMLElement

  const emitRemove = (): void => {
    invokeCallback(el, 'onRemove')
  }

  const onKeydown = (e: KeyboardEvent): void => {
    if (el.hasAttribute('disabled')) return
    if (!el.hasAttribute('removable')) return
    if (e.key === 'Backspace' || e.key === 'Delete') {
      e.preventDefault()
      emitRemove()
    }
  }
  el.addEventListener('keydown', onKeydown)

  const remove = el.querySelector('button[aria-label="Remove"]') as HTMLElement | null
  const onRemoveClick = (e: MouseEvent): void => {
    e.stopPropagation()
    if (el.hasAttribute('disabled')) return
    if (!el.hasAttribute('removable')) return
    emitRemove()
  }
  remove?.addEventListener('click', onRemoveClick)

  return {
    destroy: () => {
      el.removeEventListener('keydown', onKeydown)
      remove?.removeEventListener('click', onRemoveClick)
    },
  }
}
