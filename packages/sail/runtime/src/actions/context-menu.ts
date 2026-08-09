import type { Action } from '../action'

/**
 * `use:contextMenu` — opens the L1 `<context-menu>` (a native `[popover]`) at the
 * pointer on right-click. The menu's `for` attribute names the target element (or
 * it defaults to the menu's parent); we suppress the native context menu, place
 * the popover at the pointer, and show it. Item semantics/keyboard come from
 * `use:menu`; light-dismiss + top-layer come from the native popover.
 *
 * Target resolution is **live**: `for` may be set after mount, and the target
 * node may appear later (catalog tile mount order). Resolve on every
 * `contextmenu` and rebind the listener when `for` / the target changes.
 */
export const contextMenu: Action<void> = (node) => {
  const root = node as HTMLElement & { showPopover?: () => void; hidePopover?: () => void }

  const resolveTarget = (): HTMLElement | null => {
    const id = root.getAttribute('for')
    if (id) {
      const byId = document.getElementById(id)
      if (byId) return byId
      // Same light-DOM / residual shadow fallback as use:anchor.
      const safe =
        typeof CSS !== 'undefined' && typeof CSS.escape === 'function' ? CSS.escape(id) : id
      const rn = root.getRootNode() as ParentNode & { host?: Element }
      return (
        rn.querySelector?.<HTMLElement>(`#${safe}`) ??
        rn.host?.querySelector<HTMLElement>(`#${safe}`) ??
        null
      )
    }
    return root.parentElement
  }

  let pending: ReturnType<typeof setTimeout> | undefined

  const onContext = (e: MouseEvent): void => {
    e.preventDefault()
    const px = e.clientX
    const py = e.clientY

    // `contextmenu` fires BETWEEN pointerdown and pointerup. The native
    // popover's light-dismiss pairs that already-fired outside pointerdown with
    // the pointerup still to come — so showing synchronously here means the
    // browser dismisses us the instant the button is released. Defer past the
    // pointer sequence so the menu survives the click that summoned it.
    if (pending) clearTimeout(pending)
    pending = setTimeout(() => {
      pending = undefined
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

  let bound: HTMLElement | null = null
  const detach = (): void => {
    if (!bound) return
    bound.removeEventListener('contextmenu', onContext)
    bound = null
  }
  const rebind = (): void => {
    const next = resolveTarget()
    if (next === bound) return
    detach()
    bound = next
    bound?.addEventListener('contextmenu', onContext)
  }

  rebind()

  let raf = 0
  const scheduleRebind = (): void => {
    if (raf) return
    raf =
      typeof requestAnimationFrame === 'function'
        ? requestAnimationFrame(() => {
            raf = 0
            rebind()
          })
        : 0
    if (!raf) rebind()
  }
  const mo = new MutationObserver(scheduleRebind)
  const scope = root.getRootNode()
  const observeRoot =
    scope instanceof Document ? scope.documentElement : (scope as Node)
  mo.observe(observeRoot, {
    childList: true,
    subtree: true,
    attributes: true,
    attributeFilter: ['id'],
  })
  mo.observe(root, { attributes: true, attributeFilter: ['for'] })

  return {
    destroy: () => {
      if (pending) clearTimeout(pending)
      pending = undefined
      if (raf && typeof cancelAnimationFrame === 'function') cancelAnimationFrame(raf)
      mo.disconnect()
      detach()
    },
  }
}
