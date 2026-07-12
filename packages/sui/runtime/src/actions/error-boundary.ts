import type { Action } from '../action'
import { attachBoundary, detachBoundary } from '../error-boundary'

/** The boundary hangs `reset()` on its own host so callers can recover it
 *  programmatically: `document.querySelector('error-boundary').reset()`. */
interface HostWithReset extends HTMLElement {
  reset?: () => void
}

/**
 * `use:errorBoundary` — the L1 `<error-boundary>` element's behavior.
 *
 * The catch machinery already exists in the runtime (`attachBoundary` /
 * `findNearestBoundary` / `runProtected`, called from component mount, `bindIf`,
 * and the `{#each}` row renderer). What was missing was the ELEMENT that
 * registers a handle for the DOM walk to find — without one,
 * `findNearestBoundary` always returns null and every caught error degrades to a
 * `console.error`. This is that element's half of the contract.
 *
 * The branch swap is PURE CSS, gated on `[data-caught]` — no signal, no `{#if}`,
 * no re-render. This action supplies only what CSS cannot:
 *  - register the boundary handle for the runtime's DOM walk;
 *  - mark `[data-caught]` on the first throw, with a re-entrancy guard so a
 *    second inner throw cannot overwrite the original error;
 *  - publish the error out of band (`data-error-message` + a `sui-error-caught`
 *    event) WITHOUT rendering it — an exception string is written for a
 *    developer, not for the person using the app;
 *  - wire Retry, and expose the same path as `host.reset()`.
 */
export const errorBoundary: Action<void> = (node) => {
  const el = node as HostWithReset
  // Present only when the default fallback is in play — an author-supplied
  // `slot="fallback"` replaces it wholesale, and then we touch neither.
  const message = el.querySelector('.message') as HTMLElement | null
  const retry = el.querySelector('.retry') as HTMLButtonElement | null

  const reset = (): void => {
    if (!el.hasAttribute('data-caught')) return // already healthy — idempotent
    el.removeAttribute('data-caught')
    el.removeAttribute('data-error-message')
    el.dispatchEvent(new CustomEvent('sui-error-reset', { bubbles: true, composed: true }))
  }

  attachBoundary(el, {
    catch(err, source) {
      // Re-entrancy guard. Effects inside the protected subtree are owned by the
      // ENCLOSING component, not by this element, so they survive the catch and
      // may throw again. The first error is the true one; later ones are echoes.
      if (el.hasAttribute('data-caught')) return
      const error = err instanceof Error ? err : new Error(String(err))

      el.setAttribute('data-caught', '')
      el.setAttribute('data-error-message', error.message)
      if (message) message.textContent = el.getAttribute('message') || 'Something went wrong'

      el.dispatchEvent(
        new CustomEvent('sui-error-caught', {
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
