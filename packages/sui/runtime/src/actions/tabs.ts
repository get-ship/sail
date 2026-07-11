import type { Action } from '../action'

/**
 * `use:tabs` — the L1 `<tabs>` element's behavior, in light DOM. The current
 * `<sui-tabs>` shadow component queried `assignedElements()`; here we own the
 * whole subtree, so we query descendant `.sui-tab` / `.sui-tab-panel` directly.
 *
 * Wires the ARIA tablist contract (role, aria-selected, aria-controls /
 * aria-labelledby), roving tabindex, keyboard nav (Arrow per orientation,
 * Home/End, Enter/Space), panel show/hide, and emits `sui-tab-change`. The
 * active tab is the element's `active` attribute (the name of the active tab);
 * unset defaults to the first tab.
 */
export const tabs: Action<void> = (node) => {
  const root = node as HTMLElement
  const nameOf = (el: Element): string => el.getAttribute('name') ?? ''
  const tabsOf = (): HTMLElement[] =>
    Array.from(root.querySelectorAll<HTMLElement>('.sui-tab'))
  const panelsOf = (): HTMLElement[] =>
    Array.from(root.querySelectorAll<HTMLElement>('.sui-tab-panel'))
  const activeName = (): string => root.getAttribute('active') ?? ''

  const sync = (): void => {
    const tabs = tabsOf()
    let active = activeName()
    if (!active && tabs.length) {
      active = nameOf(tabs[0]!)
      if (active) root.setAttribute('active', active)
    }
    for (const t of tabs) {
      const name = nameOf(t)
      const on = name === active
      t.setAttribute('role', 'tab')
      t.setAttribute('aria-selected', on ? 'true' : 'false')
      t.setAttribute('tabindex', on ? '0' : '-1')
      t.toggleAttribute('active', on)
      if (name) {
        if (!t.id) t.id = `sui-tab-${name}`
        t.setAttribute('aria-controls', `sui-tabpanel-${name}`)
      }
    }
    for (const p of panelsOf()) {
      const name = nameOf(p)
      const on = name === active
      p.setAttribute('role', 'tabpanel')
      p.hidden = !on
      p.setAttribute('tabindex', on ? '0' : '-1')
      if (name) {
        if (!p.id) p.id = `sui-tabpanel-${name}`
        p.setAttribute('aria-labelledby', `sui-tab-${name}`)
      }
    }
  }

  const focusTab = (name: string): void => {
    tabsOf().find((t) => nameOf(t) === name)?.focus()
  }

  const activate = (name: string): void => {
    if (!name) return
    if (name !== activeName()) {
      root.setAttribute('active', name)
      sync()
      root.dispatchEvent(
        new CustomEvent('sui-tab-change', {
          detail: { name },
          bubbles: true,
          composed: true,
        }),
      )
    }
    focusTab(name)
  }

  const tabFrom = (e: Event): HTMLElement | null => {
    const t = (e.target as Element | null)?.closest<HTMLElement>('.sui-tab') ?? null
    return t && root.contains(t) ? t : null
  }

  const onClick = (e: MouseEvent): void => {
    const t = tabFrom(e)
    if (t && !t.hasAttribute('disabled')) activate(nameOf(t))
  }

  const onKeydown = (e: KeyboardEvent): void => {
    const t = tabFrom(e)
    if (!t) return
    const enabled = tabsOf().filter((x) => !x.hasAttribute('disabled'))
    const i = enabled.indexOf(t)
    if (i < 0) return
    const vertical = root.getAttribute('orientation') === 'vertical'
    const [next, prev] = vertical ? ['ArrowDown', 'ArrowUp'] : ['ArrowRight', 'ArrowLeft']
    let j = -1
    if (e.key === next) j = (i + 1) % enabled.length
    else if (e.key === prev) j = (i - 1 + enabled.length) % enabled.length
    else if (e.key === 'Home') j = 0
    else if (e.key === 'End') j = enabled.length - 1
    else if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault()
      if (!t.hasAttribute('disabled')) activate(nameOf(t))
      return
    }
    if (j >= 0) {
      e.preventDefault()
      activate(nameOf(enabled[j]!))
    }
  }

  root.addEventListener('click', onClick)
  root.addEventListener('keydown', onKeydown)
  const mo = new MutationObserver(() => sync())
  mo.observe(root, { childList: true, subtree: true })
  sync()

  return {
    destroy: () => {
      root.removeEventListener('click', onClick)
      root.removeEventListener('keydown', onKeydown)
      mo.disconnect()
    },
  }
}
