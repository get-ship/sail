import type { Action } from '../action'

/**
 * `use:recoverySurface` — the L1 `<error-state>` element's behavior. Surface, icon
 * tile, and the details/toggle visibility are PURE CSS gated on the reserved
 * `data-state` attribute (`has-details` / `open` tokens — group-state-*: in
 * structure sx). This action supplies what CSS can't:
 *  - fill the code chip from the `code` attribute (re-synced on change);
 *  - detect whether the details region has content → stamp
 *    `data-state="has-details"` so the Show/Hide toggle appears, and wire the
 *    toggle (label, aria-expanded, `open` token on data-state);
 *  - auto-focus the first action so the recovery control is reachable at once.
 * Written once in the toolchain — no per-instance controller, compiles away.
 */
export const recoverySurface: Action<void> = (node) => {
  const el = node as HTMLElement
  const code = el.querySelector('.code') as HTMLElement | null
  const details = el.querySelector('.details') as HTMLElement | null
  const toggle = el.querySelector('.details-toggle') as HTMLButtonElement | null

  const syncCode = (): void => {
    if (code) code.textContent = el.getAttribute('code') ?? ''
  }

  const hasDetails = (): boolean =>
    !!details && (details.textContent ?? '').trim().length > 0

  const isOpen = (): boolean => {
    const s = el.getAttribute('data-state') ?? ''
    return s.split(/\s+/).includes('open')
  }

  /** Stamp `data-state` tokens the structure sx group-state-*: prefixes read. */
  const setState = (has: boolean, open: boolean): void => {
    if (!has) {
      el.removeAttribute('data-state')
      return
    }
    el.setAttribute('data-state', open ? 'has-details open' : 'has-details')
  }

  const syncDetails = (): void => {
    if (hasDetails()) {
      setState(true, isOpen())
    } else {
      setState(false, false)
      if (toggle) {
        toggle.textContent = 'Show details'
        toggle.setAttribute('aria-expanded', 'false')
      }
    }
  }

  const onToggle = (): void => {
    const open = !isOpen()
    setState(hasDetails(), open)
    if (toggle) {
      toggle.textContent = open ? 'Hide details' : 'Show details'
      toggle.setAttribute('aria-expanded', open ? 'true' : 'false')
    }
  }
  toggle?.addEventListener('click', onToggle)

  syncCode()
  syncDetails()

  const obs = new MutationObserver(() => {
    syncCode()
    syncDetails()
  })
  obs.observe(el, { attributes: true, attributeFilter: ['code'], childList: true, subtree: true, characterData: true })

  // Auto-focus the first action control (the retry) so recovery is reachable.
  const firstAction = el.querySelector('.actions button, .actions [tabindex], .actions a') as HTMLElement | null
  if (firstAction) requestAnimationFrame(() => firstAction.focus?.())

  return {
    destroy: () => {
      obs.disconnect()
      toggle?.removeEventListener('click', onToggle)
    },
  }
}
