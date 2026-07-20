import type { Action } from '../action'

/**
 * `use:contextMenu` — opens the L1 `<context-menu>` (a native `[popover]`) at the
 * pointer on right-click. The menu's `for` attribute names the target element (or
 * it defaults to the menu's parent); we suppress the native context menu, place
 * the popover at the pointer, and show it. Item semantics/keyboard come from
 * `use:menu`; light-dismiss + top-layer come from the native popover.
 */
export const contextMenu: Action<void> = (node) => {
  const root = node as HTMLElement & { showPopover?: () => void; hidePopover?: () => void }

  const target = (): HTMLElement | null => {
    const id = root.getAttribute('for')
    if (id) return document.getElementById(id)
    return root.parentElement
  }

  const onContext = (e: MouseEvent): void => {
    e.preventDefault()
    const px = e.clientX
    const py = e.clientY

    // `contextmenu` fires BETWEEN pointerdown and pointerup. The native
    // popover's light-dismiss pairs that already-fired outside pointerdown with
    // the pointerup still to come — so showing synchronously here means the
    // browser dismisses us the instant the button is released. Defer past the
    // pointer sequence so the menu survives the click that summoned it.
    setTimeout(() => {
      // `hidePopover()` THROWS when not currently showing — never share its try
      // with the show, or a first right-click would never open.
      try {
        root.hidePopover?.()
      } catch {
        /* wasn't open — fine */
      }
      try {
        root.showPopover?.()
      } catch {
        return // no popover support
      }
      // Place at the pointer, clamped into the viewport. Skins own
      // position:fixed + margin:0; the action only publishes geometry vars.
      const r = root.getBoundingClientRect()
      const x = Math.min(px, Math.max(0, window.innerWidth - r.width - 8))
      const y = Math.min(py, Math.max(0, window.innerHeight - r.height - 8))
      root.style.setProperty('--anchor-left', `${Math.max(8, x)}px`)
      root.style.setProperty('--anchor-top', `${Math.max(8, y)}px`)
    }, 0)
  }

  const t = target()
  t?.addEventListener('contextmenu', onContext)

  return {
    destroy: () => {
      t?.removeEventListener('contextmenu', onContext)
    },
  }
}
