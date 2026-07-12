import { applyTheme, persistTheme, THEMES, isValidTheme, type ThemeId } from '@ship/tokens/themes'
import type { Action } from '../action'

/**
 * `use:themePicker` — the `<theme-picker>` behavior: a trigger that opens a
 * panel of theme cards.
 *
 * Selection goes through `applyTheme`, the framework's own function: it sets
 * `data-theme`, toggles the `.dark` class, and announces `ship:theme:change` so
 * chrome can repaint. Setting `data-theme` by hand — as this action used to —
 * does the first and skips the other two, which leaves the page half-switched.
 *
 * The panel is a native `[popover]`: top layer, Escape and outside-click close
 * for free, no positioning code.
 */
export const themePicker: Action<void> = (node) => {
  const root = node as HTMLElement
  const trigger = root.querySelector<HTMLButtonElement>('.sui-theme-trigger')
  const panel = root.querySelector<HTMLElement>('.sui-theme-panel')

  const options = (): HTMLElement[] =>
    Array.from(root.querySelectorAll<HTMLElement>('.sui-theme-option'))
  const valueOf = (el: Element): string => el.getAttribute('value') ?? ''

  const current = (): ThemeId => {
    const attr = root.getAttribute('value') || document.documentElement.dataset.theme
    return isValidTheme(attr) ? attr : 'ship-light'
  }

  const sync = (): void => {
    const cur = current()
    // The trigger is an icon button; the CURRENT theme rides its accessible
    // name (and the hover tooltip), not a visible label.
    const name = THEMES.find((t) => t.id === cur)?.label ?? cur
    if (trigger) {
      trigger.setAttribute('aria-label', `Theme: ${name}`)
      trigger.title = `Theme: ${name}`
    }
    for (const o of options()) {
      const on = valueOf(o) === cur
      o.setAttribute('aria-selected', on ? 'true' : 'false')
      o.setAttribute('tabindex', on ? '0' : '-1')
    }
  }

  const select = (v: string): void => {
    if (!isValidTheme(v)) return
    root.setAttribute('value', v)
    applyTheme(v)
    persistTheme(v) // a pick that resets on the next navigation is a bug, not a choice
    sync()
    root.dispatchEvent(
      new CustomEvent('sui-theme-change', { detail: { theme: v }, bubbles: true, composed: true }),
    )
    panel?.hidePopover?.()
    trigger?.focus()
  }

  const optionFrom = (e: Event): HTMLElement | null => {
    const o = (e.target as Element | null)?.closest<HTMLElement>('.sui-theme-option') ?? null
    return o && root.contains(o) ? o : null
  }

  const onClick = (e: MouseEvent): void => {
    const o = optionFrom(e)
    if (o) select(valueOf(o))
  }

  const onKeydown = (e: KeyboardEvent): void => {
    const o = optionFrom(e)
    if (!o) return
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault()
      select(valueOf(o))
      return
    }
    const opts = options()
    const i = opts.indexOf(o)
    let j = -1
    if (e.key === 'ArrowDown' || e.key === 'ArrowRight') j = (i + 1) % opts.length
    else if (e.key === 'ArrowUp' || e.key === 'ArrowLeft') j = (i - 1 + opts.length) % opts.length
    else if (e.key === 'Home') j = 0
    else if (e.key === 'End') j = opts.length - 1
    if (j >= 0) {
      e.preventDefault()
      opts[j]!.focus()
    }
  }

  // Land the keyboard on the active card when the panel opens.
  const onToggle = (e: Event): void => {
    if ((e as ToggleEvent).newState !== 'open') return
    sync()
    const cur = current()
    ;(options().find((o) => valueOf(o) === cur) ?? options()[0])?.focus()
  }

  root.addEventListener('click', onClick)
  root.addEventListener('keydown', onKeydown)
  panel?.addEventListener('toggle', onToggle)
  sync()

  return {
    destroy: () => {
      root.removeEventListener('click', onClick)
      root.removeEventListener('keydown', onKeydown)
      panel?.removeEventListener('toggle', onToggle)
    },
  }
}
