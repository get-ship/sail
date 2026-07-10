import type { Action } from '../action'

/**
 * `use:emptyState` — the L1 `<empty-state>` element's only behavior. The
 * surface, tile, typography, and variant/size/tone are all PURE CSS. The one
 * thing CSS can't express is the `connecting` variant's live region: a
 * connecting placeholder should announce (role=status / aria-live=polite),
 * while an ordinary empty state is inert (role=note / aria-live=off, the static
 * defaults). This action flips those two attributes off the `variant`, and
 * re-evaluates when it changes. Written once — no per-instance controller.
 */
export const emptyState: Action<void> = (node) => {
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
