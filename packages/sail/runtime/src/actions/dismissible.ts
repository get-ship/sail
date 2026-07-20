// dismissible — removable host emits `sail-remove`. `use:dismissible`
//
// One of the three behaviors decomposed from the L1 `<chip>` element
// controller (pressable / pressed / dismissible); stack them with `use:`
// stamps instead of a monolithic controller. Extracted verbatim from
// `element-controllers/chip.ts` — parity is the contract.

import type { Action } from '../action'

/**
 * Emit `sail-remove` (bubbles, composed) from a removable host:
 *
 * - `Backspace` / `Delete` keydown fires it, but only while the host carries
 *   `[removable]` — the attribute is the gate because L1 templates are
 *   static: stamps are unconditional, so the condition lives here. The key
 *   is `preventDefault()`ed so it doesn't navigate history or delete text.
 * - A click on a `.remove` descendant fires it too, with `stopPropagation()`
 *   so the remove gesture isn't also treated as host activation.
 *
 * Both paths are suppressed while the host is `[disabled]`. Consumers listen
 * for `sail-remove` and own the actual removal — the action only signals.
 */
export const dismissible: Action<void> = (node) => {
  const el = node as HTMLElement

  const emitRemove = (): void => {
    el.dispatchEvent(new CustomEvent('sail-remove', { detail: {}, bubbles: true, composed: true }))
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

  const remove = el.querySelector('.remove') as HTMLElement | null
  const onRemoveClick = (e: MouseEvent): void => {
    e.stopPropagation()
    if (el.hasAttribute('disabled')) return
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
