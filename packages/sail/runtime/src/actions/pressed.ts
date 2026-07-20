// pressed — toggle state reflection. `use:pressed`
//
// One of the three behaviors the L1 `<chip>` element stacks with `use:`
// stamps (pressable / pressed / dismissible) instead of a monolithic
// controller. Behavior parity with the retired chip controller is the
// contract.

import type { Action } from '../action'

/**
 * Reflect the host's `[selected]` attribute to `aria-pressed`
 * (`"true"` / `"false"`), kept in sync by a MutationObserver.
 *
 * Pairs with `use:pressable` for toggle-button hosts — filter chips,
 * toggles, segmented-control options: any element whose pressed state is
 * author-driven by a `selected` attribute rather than by the platform.
 */
export const pressed: Action<void> = (node) => {
  const el = node as HTMLElement

  const sync = (): void => {
    el.setAttribute('aria-pressed', el.hasAttribute('selected') ? 'true' : 'false')
  }
  sync()
  const obs = new MutationObserver(sync)
  obs.observe(el, { attributes: true, attributeFilter: ['selected'] })

  return {
    destroy: () => {
      obs.disconnect()
    },
  }
}
