import type { Action } from '../action'

/**
 * `use:recoverySurface` — the L1 `<error-state>` element's behavior. Surface, icon
 * tile, and the details/toggle visibility are PURE CSS (gated on the
 * `data-has-details` / `data-details-open` attributes). This action supplies
 * what CSS can't:
 *  - fill the code chip from the `code` attribute (re-synced on change);
 *  - detect whether the details region has content → mark [data-has-details]
 *    so the Show/Hide toggle appears, and wire the toggle (label,
 *    aria-expanded, [data-details-open]);
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

  const syncDetails = (): void => {
    if (hasDetails()) {
      el.setAttribute('data-has-details', '')
    } else {
      el.removeAttribute('data-has-details')
      el.removeAttribute('data-details-open')
      if (toggle) {
        toggle.textContent = 'Show details'
        toggle.setAttribute('aria-expanded', 'false')
      }
    }
  }

  const onToggle = (): void => {
    const open = !el.hasAttribute('data-details-open')
    el.toggleAttribute('data-details-open', open)
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
