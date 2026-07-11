import type { Action } from '../action'
import type { Signal } from '../signal'
import { bindOpen } from '../enhance-dialog'

let _seq = 0

/**
 * `use:dialog` — the L1 `<dialog>` element's behavior, on the native modal. The
 * platform owns the hard parts (top layer, focus trap, ESC, focus-return,
 * background inertness) and DIALOG_SKIN_CSS owns the look (panel, ::backdrop,
 * size, header/section/footer, scroll-lock via :has); this action adds only the
 * author-facing wiring:
 *  - two-way open ↔ showModal()/close() via the bound signal (bindOpen), with
 *    dismissible ([dismissible='false'] blocks ESC + backdrop);
 *  - an injected × close button (unless [no-close-button]) that closes;
 *  - aria-labelledby / aria-describedby from the first <header> / <section>;
 *  - composed sui-open / sui-close events off the native open/close.
 * Usage: <dialog use:dialog={openSignal}>…</dialog>. Written once — no
 * per-instance controller; the dialog is a bare native element.
 */
export const dialog: Action<Signal<boolean>> = (node, sig) => {
  const el = node as HTMLDialogElement
  const n = ++_seq
  const dismissible = el.getAttribute('dismissible') !== 'false'

  // aria wiring from the semantic structure the skin styles.
  const header = el.querySelector(':scope > header') as HTMLElement | null
  const body = el.querySelector(':scope > section') as HTMLElement | null
  if (header) {
    if (!header.id) header.id = `sui-dialog-header-${n}`
    el.setAttribute('aria-labelledby', header.id)
  }
  if (body) {
    if (!body.id) body.id = `sui-dialog-body-${n}`
    el.setAttribute('aria-describedby', body.id)
  }

  // injected close button (convenience; opt out with [no-close-button]).
  let closeBtn: HTMLButtonElement | null = null
  const onCloseClick = (): void => el.close()
  if (!el.hasAttribute('no-close-button')) {
    closeBtn = document.createElement('button')
    closeBtn.type = 'button'
    closeBtn.className = 'sui-dialog-close'
    closeBtn.setAttribute('aria-label', 'Close')
    closeBtn.innerHTML =
      '<svg viewBox="0 0 10 10" aria-hidden="true"><path d="M2 2 L8 8 M8 2 L2 8" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" fill="none"/></svg>'
    closeBtn.addEventListener('click', onCloseClick)
    el.prepend(closeBtn)
  }

  // sui-open / sui-close off the native open state.
  const emit = (name: string): void =>
    void el.dispatchEvent(new CustomEvent(name, { bubbles: true, composed: true }))
  const obs = new MutationObserver(() => {
    if (el.open) emit('sui-open')
  })
  obs.observe(el, { attributes: true, attributeFilter: ['open'] })
  const onClose = (): void => emit('sui-close')
  el.addEventListener('close', onClose)

  const stopBind = sig ? bindOpen(el, sig, { dismissible }) : () => {}

  return {
    destroy: () => {
      stopBind()
      obs.disconnect()
      el.removeEventListener('close', onClose)
      closeBtn?.removeEventListener('click', onCloseClick)
      closeBtn?.remove()
    },
  }
}
