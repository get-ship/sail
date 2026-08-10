import type { Action } from '../action'

/**
 * `use:recoverySurface` — the L1 `<error-state>` element's behavior. Surface, icon
 * tile, and the details/toggle visibility are PURE CSS gated on the reserved
 * `data-state` attribute (`has-details` / `open` tokens — group-state-*: in
 * structure sx). This action supplies what CSS can't:
 *  - stamp role=alert + aria-live=assertive (recovery is urgent; no tone axis);
 *  - fill the code chip from the `code` attribute (re-synced on change);
 *  - detect whether the details region has content → stamp
 *    `data-state="has-details"` so the Show/Hide toggle appears, and wire the
 *    toggle (label, aria-expanded, `open` token on data-state);
 *  - auto-focus the first action so the recovery control is reachable at once.
 *
 * Anatomy is host-relative structure (not class names):
 *   children[0] icon, [1] copy column, [2] actions row
 *   copy: title, desc, code, details-toggle button, details
 *
 * MutationObserver writes MUST be idempotent. The observer watches subtree
 * childList + characterData (light-DOM slot projection + dynamic default-slot
 * text). Unconditional `textContent = …` replaces the text node every time,
 * re-queues the observer, and wedges the event loop (G6 / LANG-1 item 1).
 */
export const recoverySurface: Action<void> = (node) => {
  const el = node as HTMLElement
  const copy = el.children[1] as HTMLElement | undefined
  // title, desc, code, button, details
  const code = (copy?.children[2] as HTMLElement | undefined) ?? null
  const toggle = (copy?.querySelector('button[type="button"]') as HTMLButtonElement | null) ?? null
  const details = (toggle?.nextElementSibling as HTMLElement | null) ?? null
  const actions = el.children[2] as HTMLElement | undefined

  const writeText = (target: HTMLElement, next: string): void => {
    if (target.textContent !== next) target.textContent = next
  }

  const writeAttr = (target: Element, name: string, next: string | null): void => {
    if (next === null) {
      if (target.hasAttribute(name)) target.removeAttribute(name)
      return
    }
    if (target.getAttribute(name) !== next) target.setAttribute(name, next)
  }

  // Recovery surfaces announce immediately — always alert/assertive (no tone ladder).
  writeAttr(el, 'role', 'alert')
  writeAttr(el, 'aria-live', 'assertive')

  const syncCode = (): void => {
    if (code) writeText(code, el.getAttribute('code') ?? '')
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
      writeAttr(el, 'data-state', null)
      return
    }
    writeAttr(el, 'data-state', open ? 'has-details open' : 'has-details')
  }

  const syncDetails = (): void => {
    if (hasDetails()) {
      setState(true, isOpen())
    } else {
      setState(false, false)
      if (toggle) {
        writeText(toggle, 'Show details')
        writeAttr(toggle, 'aria-expanded', 'false')
      }
    }
  }

  const onToggle = (): void => {
    const open = !isOpen()
    setState(hasDetails(), open)
    if (toggle) {
      writeText(toggle, open ? 'Hide details' : 'Show details')
      writeAttr(toggle, 'aria-expanded', open ? 'true' : 'false')
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
  const firstAction = actions?.querySelector(
    'button, [tabindex], a',
  ) as HTMLElement | null
  if (firstAction) requestAnimationFrame(() => firstAction.focus?.())

  return {
    destroy: () => {
      obs.disconnect()
      toggle?.removeEventListener('click', onToggle)
    },
  }
}
