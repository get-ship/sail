import type { Action } from '../action'
import { attachBoundary, detachBoundary } from '../error-boundary'

/** The boundary hangs `reset()` on its own host so callers can recover it
 *  programmatically: `document.querySelector('error-boundary').reset()`. */
interface HostWithReset extends HTMLElement {
  reset?: () => void
}

/**
 * `use:catchRender` — the `<error-boundary>` element's behavior.
 *
 * `runProtected` routes a throw by walking up the DOM for an element carrying a
 * boundary handle; this registers that handle. Without it `findNearestBoundary`
 * finds nothing and a caught error degrades to a `console.error`.
 *
 * The branch swap is pure CSS, gated on `[data-caught]` — no signal, no `{#if}`.
 * This supplies what CSS cannot: the handle, the caught mark, the error published
 * out of band, and Retry.
 *
 * Default fallback anatomy (structure under [data-error-alert], not classes):
 * message div immediately before the Retry button. Author `slot="fallback"`
 * replaces the default and then we touch neither. Stamps role=alert (closed
 * anatomy — not free author markup).
 */
export const catchRender: Action<void> = (node) => {
  const el = node as HostWithReset
  // Present only when the default fallback is in play — an author-supplied
  // `slot="fallback"` replaces it wholesale, and then we touch neither.
  const alert =
    (el.querySelector('[data-error-alert]') as HTMLElement | null) ??
    (el.querySelector('[role="alert"]') as HTMLElement | null)
  if (alert && !alert.hasAttribute('role')) alert.setAttribute('role', 'alert')
  const retry = alert?.querySelector('button[type="button"]') as HTMLButtonElement | null
  const message = (retry?.previousElementSibling as HTMLElement | null) ?? null

  const reset = (): void => {
    if (!el.hasAttribute('data-caught')) return
    el.removeAttribute('data-caught')
    el.removeAttribute('data-error-message')
    el.dispatchEvent(new CustomEvent('sail-error-reset', { bubbles: true, composed: true }))
  }

  attachBoundary(el, {
    catch(err, source) {
      // Effects in the protected subtree are owned by the enclosing component, so
      // they survive the catch and may throw again. The first error is the real
      // one; later ones are echoes of it.
      if (el.hasAttribute('data-caught')) return
      const error = err instanceof Error ? err : new Error(String(err))

      el.setAttribute('data-caught', '')
      el.setAttribute('data-error-message', error.message)
      // The message shown is the prop, never the exception text: that string is
      // written for a developer, not for the person using the app.
      if (message) message.textContent = el.getAttribute('message') || 'Something went wrong'

      el.dispatchEvent(
        new CustomEvent('sail-error-caught', {
          bubbles: true,
          composed: true,
          detail: { error, source },
        }),
      )
    },
    reset,
  })

  retry?.addEventListener('click', reset)
  el.reset = reset

  return {
    destroy: () => {
      detachBoundary(el)
      retry?.removeEventListener('click', reset)
      delete el.reset
    },
  }
}
