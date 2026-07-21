import type { Action } from '../action'

/**
 * `use:hoverSurface` — opens the L1 `<hover-card>` (a native `[popover]`) on hover or
 * focus of its trigger (the element whose `popovertarget` names it), with
 * open/close delays so it doesn't flicker. Positioning is `use:anchor`'s job;
 * top-layer + light-dismiss are the native popover's.
 */
export const hoverSurface: Action<void> = (node) => {
  const root = node as HTMLElement & { showPopover?: () => void; hidePopover?: () => void }
  const openMs = Number(root.getAttribute('open-delay') ?? '') || 200
  const closeMs = Number(root.getAttribute('close-delay') ?? '') || 150

  const trigger = (): HTMLElement | null =>
    root.id ? document.querySelector<HTMLElement>(`[popovertarget="${root.id}"]`) : null

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

  const t = trigger()
  t?.addEventListener('pointerenter', show)
  t?.addEventListener('pointerleave', hide)
  t?.addEventListener('focus', show)
  t?.addEventListener('blur', hide)
  // Keep it open while the pointer is inside the card itself.
  root.addEventListener('pointerenter', clear)
  root.addEventListener('pointerleave', hide)

  return {
    destroy: () => {
      clear()
      t?.removeEventListener('pointerenter', show)
      t?.removeEventListener('pointerleave', hide)
      t?.removeEventListener('focus', show)
      t?.removeEventListener('blur', hide)
      root.removeEventListener('pointerenter', clear)
      root.removeEventListener('pointerleave', hide)
    },
  }
}
