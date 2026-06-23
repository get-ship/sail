import { controller, signal, attachBoundary, detachBoundary } from '@ship/sui-runtime'
import type { BoundarySource, ViewScope } from '@ship/sui-runtime'
import type { Scope } from './sui-error-boundary.sui'

interface HostWithReset extends HTMLElement {
  reset?: () => void
}

export default controller({
  scope: (_ctx: ViewScope<Scope>, host?: Element) => {
    const caughtError = signal<Error | null>(null)
    const hasFallbackSlot = signal<boolean>(false)
    const reset = (): void => {
      caughtError.set(null)
      if (host instanceof HTMLElement) host.removeAttribute('data-error-message')
    }
    return { caughtError, hasFallbackSlot, reset }
  },

  mount(state, host) {
    // Authors commonly wrap their custom fallback in <template slot="fallback">
    // so the markup isn't visible while the boundary is healthy. Templates
    // don't render their contents, so we clone the inert template into a
    // live wrapper element bearing the same slot name — the assignment
    // becomes visible the moment the boundary swaps to its fallback branch.
    const templates = host.querySelectorAll(':scope > template[slot="fallback"]')
    for (const tpl of Array.from(templates)) {
      const wrap = document.createElement('div')
      wrap.setAttribute('slot', 'fallback')
      wrap.appendChild((tpl as HTMLTemplateElement).content.cloneNode(true))
      tpl.replaceWith(wrap)
    }
    state.hasFallbackSlot = !!host.querySelector(':scope > [slot="fallback"]')

    attachBoundary(host, {
      catch(err, source) {
        // Re-entrancy guard: once the fallback is rendered, a second
        // inner throw shouldn't overwrite the original error.
        if (state.caughtError !== null) return
        const error = err instanceof Error ? err : new Error(String(err))
        host.setAttribute('data-error-message', error.message)
        state.caughtError = error
        host.dispatchEvent(
          new CustomEvent('sui-error-boundary:caught', {
            bubbles: true,
            composed: true,
            detail: { error, source: source as BoundarySource },
          }),
        )
      },
      reset: state.reset,
    })

    ;(host as HostWithReset).reset = state.reset

    return () => {
      detachBoundary(host)
      delete (host as HostWithReset).reset
    }
  },
})
