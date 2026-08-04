// Native <dialog> behavior — thin sugar over the platform's modal engine.
//
// The platform owns the hard, easy-to-get-wrong parts and we do NOT touch
// them: top-layer stacking, the focus trap, ESC, focus-return to the trigger,
// background inertness. This helper adds ONLY what the platform leaves to the
// author: a reactive open<->showModal/close binding, close-event sync so state
// can't desync, and backdrop-click dismissal. Scroll-lock is pure CSS
// (`html:has(dialog[open])`) in the skin — no JS. If this file ever grows a
// focus trap, it has crossed the line from "extend" to "compete".

import { effect, type Signal } from './signal'
import type { Disposer } from './bind'

/** Options for {@link bindOpen}. */
export interface DialogOptions {
  /**
   * Can ESC or a backdrop click close the dialog? Default `true`. A
   * destructive confirm sets this `false` so the user must pick an explicit
   * action (Cancel / Delete) rather than dismissing by accident.
   */
  dismissible?: boolean
}

/**
 * Two-way bind a boolean signal to a native `<dialog>`'s MODAL state.
 *
 * - signal `true`  → `el.showModal()` — top layer, `::backdrop`, focus trap,
 *   ESC and focus-return, all from the platform.
 * - signal `false` → `el.close()`.
 * - the native `close` event (ESC, backdrop, a close button, `<form
 *   method="dialog">`, or `.close()` anywhere) writes `false` back into the
 *   signal — so author state can never desync from what the platform did.
 *
 * Note the deliberate choice of `showModal()` over the `open` attribute: the
 * attribute shows the dialog NON-modally (no backdrop, no top layer), so a
 * plain attribute reflection would be the wrong behavior.
 *
 * Compiler target: `<dialog bind:open={state}>` (optionally with
 * `dismissible={false}`).
 *
 * @param el The native dialog element.
 * @param sig Boolean signal that owns the open state.
 * @param opts {@link DialogOptions}.
 * @returns Disposer that stops the binding and removes every listener.
 */
export function bindOpen(
  el: HTMLDialogElement,
  sig: Signal<boolean>,
  opts: DialogOptions = {},
): Disposer {
  const dismissible = opts.dismissible !== false
  let disposed = false
  let pendingConnect: (() => void) | null = null

  /**
   * showModal() throws InvalidStateError when the dialog is not connected.
   * Compiler emit for `<if {open()}><dialog open={open()}></if>` runs bindOpen
   * (and this effect) while the node is still in a DocumentFragment — insert
   * happens only after the branch render returns. Defer until connected so
   * dirty-leave modals actually enter the top layer.
   */
  const showWhenReady = () => {
    if (disposed || el.open || !sig()) return
    if (el.isConnected) {
      try {
        el.showModal()
      } catch {
        // Already open / racing close — next signal tick re-applies.
      }
      return
    }
    if (pendingConnect) return
    const retry = () => {
      pendingConnect = null
      if (disposed || el.open || !sig()) return
      if (el.isConnected) {
        try {
          el.showModal()
        } catch {
          /* ignore */
        }
        return
      }
      // Still detached (slow parent) — one more frame.
      pendingConnect = retry
      requestAnimationFrame(retry)
    }
    pendingConnect = retry
    queueMicrotask(retry)
  }

  // signal → modal state. Guard on el.open so a re-run never double-calls
  // showModal()/close() (both throw / no-op in the wrong state).
  const dispose = effect(() => {
    const want = sig()
    if (want && !el.open) showWhenReady()
    else if (!want && el.open) el.close()
  })

  // native close (ESC / backdrop / a close button / form method=dialog /
  // .close()) → signal. Guarded so the programmatic close above doesn't loop.
  // Also emit sail-close / sail-dismiss so author templates that listened to the
  // retired CE events still sync parent state (claim capture, mission create).
  const onClose = () => {
    if (sig()) sig.set(false)
    el.dispatchEvent(new CustomEvent('sail-close', { bubbles: true, composed: true }))
    el.dispatchEvent(new CustomEvent('sail-dismiss', { bubbles: true, composed: true }))
  }
  el.addEventListener('close', onClose)

  // ESC fires `cancel` then `close`. When not dismissible, cancel the cancel.
  const onCancel = (e: Event) => {
    if (!dismissible) e.preventDefault()
  }
  el.addEventListener('cancel', onCancel)

  // Backdrop click → close (native doesn't do this). A click on the backdrop
  // targets the dialog element itself; hit-test against the panel rect so a
  // click on the panel's own padding still counts as inside.
  const onClick = (e: MouseEvent) => {
    if (!dismissible || e.target !== el) return
    const r = el.getBoundingClientRect()
    const inside =
      e.clientX >= r.left &&
      e.clientX <= r.right &&
      e.clientY >= r.top &&
      e.clientY <= r.bottom
    if (!inside) el.close()
  }
  el.addEventListener('click', onClick)

  return () => {
    disposed = true
    pendingConnect = null
    dispose()
    el.removeEventListener('close', onClose)
    el.removeEventListener('cancel', onCancel)
    el.removeEventListener('click', onClick)
  }
}
