import type { Action } from '../action'

const host = (name: string) => `[data-sail-host="${name}"]`

const isOpen = (el: HTMLElement): boolean => {
  // `:popover-open` throws in engines without popover support — never let a
  // feature probe take the page down.
  try {
    return el.matches(':popover-open')
  } catch {
    return false
  }
}

/**
 * Internal: L1 `<dropdown>` / `<context-menu>` closed behavior (not a public `use:`).
 * Authors compose `<dropdown>` / `<context-menu>` + `<menu-item>`, not `use:menu`.
 *
 * Wires role=menu / menuitem, roving focus (ArrowUp/Down, Home/End), Escape to
 * close, focuses the first item on open, and closes after activating an item.
 * Positioning is `use:anchor`'s job; open/close and light-dismiss are the
 * native popover's.
 */
export const menu: Action<void> = (node) => {
  const root = node as HTMLElement
  const itemSel = host('menu-item')
  const itemsOf = (): HTMLElement[] =>
    Array.from(root.querySelectorAll<HTMLElement>(itemSel)).filter(
      (i) => !i.hasAttribute('disabled'),
    )

  if (!root.hasAttribute('role')) root.setAttribute('role', 'menu')

  const sync = (): void => {
    for (const i of root.querySelectorAll<HTMLElement>(itemSel)) {
      i.setAttribute('role', 'menuitem')
      if (!i.hasAttribute('tabindex')) i.setAttribute('tabindex', '-1')
      if (i.hasAttribute('disabled')) i.setAttribute('aria-disabled', 'true')
    }
  }

  const close = (): void => {
    try {
      ;(root as HTMLElement & { hidePopover?: () => void }).hidePopover?.()
    } catch {
      /* not a popover — nothing to close */
    }
  }

  const onKeydown = (e: KeyboardEvent): void => {
    const list = itemsOf()
    if (!list.length) return
    const cur = list.indexOf(document.activeElement as HTMLElement)
    let j = -1
    if (e.key === 'ArrowDown') j = cur < 0 ? 0 : (cur + 1) % list.length
    else if (e.key === 'ArrowUp') j = cur < 0 ? list.length - 1 : (cur - 1 + list.length) % list.length
    else if (e.key === 'Home') j = 0
    else if (e.key === 'End') j = list.length - 1
    else if (e.key === 'Escape') {
      close()
      return
    }
    if (j >= 0) {
      e.preventDefault()
      list[j]!.focus()
    }
  }

  const onClick = (e: MouseEvent): void => {
    const item = (e.target as Element | null)?.closest<HTMLElement>(itemSel)
    if (!item || !root.contains(item)) return
    if (item.hasAttribute('disabled')) {
      e.preventDefault()
      e.stopPropagation()
      return
    }
    root.dispatchEvent(
      new CustomEvent('sail-menu-select', {
        detail: { value: item.getAttribute('value') ?? item.textContent?.trim() ?? '' },
        bubbles: true,
        composed: true,
      }),
    )
    close()
  }

  const onToggle = (): void => {
    if (isOpen(root)) itemsOf()[0]?.focus()
  }

  root.addEventListener('keydown', onKeydown)
  root.addEventListener('click', onClick)
  root.addEventListener('toggle', onToggle)
  const mo = new MutationObserver(() => sync())
  mo.observe(root, { childList: true, subtree: true })
  sync()

  return {
    destroy: () => {
      root.removeEventListener('keydown', onKeydown)
      root.removeEventListener('click', onClick)
      root.removeEventListener('toggle', onToggle)
      mo.disconnect()
    },
  }
}
