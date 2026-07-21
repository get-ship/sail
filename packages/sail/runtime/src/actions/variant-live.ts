// variantLive — variant-gated live region for connecting placeholders.
//
// Capability mix-in: any host that exposes a `variant` attribute can stamp
// `use:variantLive` so `variant="connecting"` upgrades to a polite live
// region (role=status / aria-live=polite), while every other variant stays
// inert (role=note / aria-live=off). Stacked on the L1 <empty-state> root.
// Deliberately separate from any tone-based alert live-region capability.

import type { Action } from '../action'

/**
 * Observe `variant` and flip role/aria-live for connecting placeholders.
 * Static defaults on the template (`role=note`, `aria-live=off`) keep the
 * host inert before hydrate; this action only upgrades `connecting` and
 * re-evaluates when the attribute changes.
 */
export const variantLive: Action<void> = (node) => {
  const el = node as HTMLElement

  const sync = (): void => {
    const connecting = el.getAttribute('variant') === 'connecting'
    el.setAttribute('role', connecting ? 'status' : 'note')
    el.setAttribute('aria-live', connecting ? 'polite' : 'off')
  }
  sync()
  const obs = new MutationObserver(sync)
  obs.observe(el, { attributes: true, attributeFilter: ['variant'] })

  return { destroy: () => obs.disconnect() }
}
