import type { Action } from '../action'

/**
 * `use:hoverSurface` — opens the L1 `<hover-card>` (a native `[popover]`) on hover or
 * focus of its trigger (the element whose `popovertarget` names it), with
 * open/close delays so it doesn't flicker. Positioning is `use:anchor`'s job;
 * top-layer + light-dismiss are the native popover's.
 *
 * Trigger resolution is **live**: catalog tiles and remounts often attach this
 * action before the invoker is in the document (or replace the invoker later).
 * One-shot capture at mount left hover dead and only the native click flash
 * remaining — re-query + rebind whenever the DOM / id / popovertarget changes.
 */
export const hoverSurface: Action<void> = (node) => {
  const root = node as HTMLElement & { showPopover?: () => void; hidePopover?: () => void }
  const openMs = Number(root.getAttribute('open-delay') ?? '') || 200
  const closeMs = Number(root.getAttribute('close-delay') ?? '') || 150

  const resolveTrigger = (): HTMLElement | null => {
    if (!root.id) return null
    // CSS.escape keeps ids with dots/colons queryable; fall back if missing.
    const id =
      typeof CSS !== 'undefined' && typeof CSS.escape === 'function'
        ? CSS.escape(root.id)
        : root.id
    const sel = `[popovertarget="${id}"]`
    const rn = root.getRootNode() as ParentNode & { host?: Element }
    const inRoot = rn.querySelector?.<HTMLElement>(sel)
    if (inRoot) return inRoot
    return (
      rn.host?.querySelector<HTMLElement>(sel) ??
      document.querySelector<HTMLElement>(sel)
    )
  }

  let openT: ReturnType<typeof setTimeout> | undefined
  let closeT: ReturnType<typeof setTimeout> | undefined
  const clear = (): void => {
    if (openT) clearTimeout(openT)
    if (closeT) clearTimeout(closeT)
    openT = closeT = undefined
  }
  const show = (): void => {
    clear()
    openT = setTimeout(() => {
      try {
        root.showPopover?.()
      } catch {
        /* no popover support */
      }
    }, openMs)
  }
  const hide = (): void => {
    clear()
    closeT = setTimeout(() => {
      try {
        root.hidePopover?.()
      } catch {
        /* no popover support */
      }
    }, closeMs)
  }

  let bound: HTMLElement | null = null
  const detach = (): void => {
    if (!bound) return
    bound.removeEventListener('pointerenter', show)
    bound.removeEventListener('pointerleave', hide)
    bound.removeEventListener('focus', show)
    bound.removeEventListener('blur', hide)
    bound = null
  }
  const rebind = (): void => {
    const next = resolveTrigger()
    if (next === bound) return
    detach()
    bound = next
    if (!bound) return
    bound.addEventListener('pointerenter', show)
    bound.addEventListener('pointerleave', hide)
    bound.addEventListener('focus', show)
    bound.addEventListener('blur', hide)
  }

  rebind()

  // Late-mounted or remounted invokers (catalog tiles, HMR, sibling order).
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
    attributeFilter: ['popovertarget', 'id'],
  })
  // Our own id may be stamped after the action attaches.
  mo.observe(root, { attributes: true, attributeFilter: ['id'] })

  // Keep it open while the pointer is inside the card itself.
  root.addEventListener('pointerenter', clear)
  root.addEventListener('pointerleave', hide)

  return {
    destroy: () => {
      clear()
      if (raf && typeof cancelAnimationFrame === 'function') cancelAnimationFrame(raf)
      mo.disconnect()
      detach()
      root.removeEventListener('pointerenter', clear)
      root.removeEventListener('pointerleave', hide)
    },
  }
}
