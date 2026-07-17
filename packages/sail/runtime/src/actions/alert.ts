import type { Action } from '../action'

/**
 * `use:alert` — the L1 `<alert>` element's behavior. The surface, the leading
 * tone stripe, the icon chip, and the dismiss control's visibility are all
 * PURE CSS (the skin, gated on the `tone` / `dismissible` attributes). This
 * action supplies only what CSS can't:
 *  - role / aria-live upgrade to an ASSERTIVE alert for warning/destructive
 *    tones (the static defaults — role=status, aria-live=polite — already
 *    announce correctly, so the banner is accessible even before this hydrates;
 *    the action only sharpens urgent tones), re-evaluated when `tone` changes;
 *  - the dismiss button fires `sail-dismiss` (bubbles, composed) and removes the
 *    banner — but only while the element is [dismissible].
 * Written once in the toolchain — no per-instance controller, compiles away.
 */
export const alert: Action<void> = (node) => {
  const el = node as HTMLElement

  const syncLive = (): void => {
    const urgent = el.getAttribute('tone') === 'warning' || el.getAttribute('tone') === 'destructive'
    el.setAttribute('role', urgent ? 'alert' : 'status')
    el.setAttribute('aria-live', urgent ? 'assertive' : 'polite')
  }
  syncLive()
  const obs = new MutationObserver(syncLive)
  obs.observe(el, { attributes: true, attributeFilter: ['tone'] })

  const close = el.querySelector('.close') as HTMLElement | null
  const onClose = (): void => {
    if (!el.hasAttribute('dismissible')) return
    el.dispatchEvent(new CustomEvent('sail-dismiss', { bubbles: true, composed: true }))
    el.remove()
  }
  close?.addEventListener('click', onClose)

  return {
    destroy: () => {
      obs.disconnect()
      close?.removeEventListener('click', onClose)
    },
  }
}
