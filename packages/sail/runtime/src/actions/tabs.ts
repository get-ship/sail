import type { Action } from '../action'

/**
 * `use:tabs` — the L1 `<tabs>` element's behavior, in light DOM. Owns the
 * whole subtree and queries descendant `.sail-tab` / `.sail-tab-panel` directly.
 *
 * Wires the ARIA tablist contract (role, aria-selected, aria-controls /
 * aria-labelledby), roving tabindex, keyboard nav (Arrow per orientation,
 * Home/End, Enter/Space), panel show/hide, and emits `sail-tab-change`. The
 * active tab is the element's `active` attribute (the name of the active tab);
 * unset defaults to the first tab.
 *
 * Orientation is structural (list/panel layout CVA); it is stamped onto
 * `.sail-tab-list` / `.sail-tab-panel` so child orientation axes resolve.
 * Size/variant paint is authored on children (or host CVA).
 */
export const tabs: Action<void> = (node) => {
  const root = node as HTMLElement
  // A tab without a name pairs with its panel BY POSITION — `<tab>Profile</tab>`
  // is the common authoring shape. All-empty names used to compare equal to the
  // (empty) active name, so every tab was selected and no panel ever hid.
  const keyOf = (el: Element, i: number): string => el.getAttribute('name') || String(i)
  const tabsOf = (): HTMLElement[] =>
    Array.from(root.querySelectorAll<HTMLElement>('.sail-tab'))
  const panelsOf = (): HTMLElement[] =>
    Array.from(root.querySelectorAll<HTMLElement>('.sail-tab-panel'))
  const listOf = (): HTMLElement | null =>
    root.querySelector<HTMLElement>('.sail-tab-list')
  const activeName = (): string => root.getAttribute('active') ?? ''
  const orientationOf = (): 'horizontal' | 'vertical' =>
    root.getAttribute('orientation') === 'vertical' ? 'vertical' : 'horizontal'

  // Structural layout only — not paint chrome (size/variant/tone).
  const stampOrientation = (): void => {
    const orientation = orientationOf()
    const list = listOf()
    if (list) list.setAttribute('orientation', orientation)
    for (const p of panelsOf()) {
      p.setAttribute('orientation', orientation)
    }
  }

  const sync = (): void => {
    stampOrientation()
    const tabs = tabsOf()
    let active = activeName()
    if (!active && tabs.length) {
      active = keyOf(tabs[0]!, 0)
      root.setAttribute('active', active)
    }
    for (const [i, t] of tabs.entries()) {
      const name = keyOf(t, i)
      const on = name === active
      t.setAttribute('role', 'tab')
      t.setAttribute('aria-selected', on ? 'true' : 'false')
      t.setAttribute('tabindex', on ? '0' : '-1')
      t.toggleAttribute('active', on)
      if (name) {
        if (!t.id) t.id = `sail-tab-${name}`
        t.setAttribute('aria-controls', `sail-tabpanel-${name}`)
      }
    }
    for (const [i, p] of panelsOf().entries()) {
      const name = keyOf(p, i)
      const on = name === active
      p.setAttribute('role', 'tabpanel')
      p.hidden = !on
      p.setAttribute('tabindex', on ? '0' : '-1')
      if (name) {
        if (!p.id) p.id = `sail-tabpanel-${name}`
        p.setAttribute('aria-labelledby', `sail-tab-${name}`)
      }
    }
  }

  const focusTab = (name: string): void => {
    tabsOf().find((t, i) => keyOf(t, i) === name)?.focus()
  }

  const activate = (name: string): void => {
    if (!name) return
    if (name !== activeName()) {
      root.setAttribute('active', name)
      sync()
      root.dispatchEvent(
        new CustomEvent('sail-tab-change', {
          detail: { name },
          bubbles: true,
          composed: true,
        }),
      )
    }
    focusTab(name)
  }

  const tabFrom = (e: Event): HTMLElement | null => {
    const t = (e.target as Element | null)?.closest<HTMLElement>('.sail-tab') ?? null
    return t && root.contains(t) ? t : null
  }

  const onClick = (e: MouseEvent): void => {
    const t = tabFrom(e)
    if (t && !t.hasAttribute('disabled')) activate(keyOf(t, tabsOf().indexOf(t)))
  }

  const onKeydown = (e: KeyboardEvent): void => {
    const t = tabFrom(e)
    if (!t) return
    const enabled = tabsOf().filter((x) => !x.hasAttribute('disabled'))
    const i = enabled.indexOf(t)
    if (i < 0) return
    const vertical = orientationOf() === 'vertical'
    const [next, prev] = vertical ? ['ArrowDown', 'ArrowUp'] : ['ArrowRight', 'ArrowLeft']
    let j = -1
    if (e.key === next) j = (i + 1) % enabled.length
    else if (e.key === prev) j = (i - 1 + enabled.length) % enabled.length
    else if (e.key === 'Home') j = 0
    else if (e.key === 'End') j = enabled.length - 1
    else if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault()
      if (!t.hasAttribute('disabled')) activate(keyOf(t, tabsOf().indexOf(t)))
      return
    }
    if (j >= 0) {
      e.preventDefault()
      activate(keyOf(enabled[j]!, tabsOf().indexOf(enabled[j]!)))
    }
  }

  root.addEventListener('click', onClick)
  root.addEventListener('keydown', onKeydown)
  const mo = new MutationObserver(() => sync())
  mo.observe(root, {
    childList: true,
    subtree: true,
    attributes: true,
    attributeFilter: ['orientation', 'active'],
  })
  sync()

  return {
    destroy: () => {
      root.removeEventListener('click', onClick)
      root.removeEventListener('keydown', onKeydown)
      mo.disconnect()
    },
  }
}
