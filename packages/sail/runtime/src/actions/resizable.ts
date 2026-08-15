import type { Action } from '../action'

const MIN_PANE_PX = 32
const NUDGE = 2
const NUDGE_LARGE = 10

/**
 * Internal: L1 `<resizable>` handle behavior (not a public `use:`).
 * Authors use `<resizable>`, not `use:resizable`.
 *
 * Pointer-down on the handle, move, set flex-basis % on the start pane
 * (previous sibling). Arrow keys nudge the same percentage; Home/End
 * snap to the min/max clamp. Stamps `role=separator` + `aria-orientation`
 * from the host's `orientation` attr (no free ARIA in the stdlib source).
 */
export const resizable: Action<void> = (node) => {
  const handle = node as HTMLElement

  const hostOf = (): HTMLElement | null =>
    handle.closest<HTMLElement>('[data-sail-host="resizable"]') ??
    (handle.parentElement as HTMLElement | null)

  const startOf = (): HTMLElement | null =>
    (handle.previousElementSibling as HTMLElement | null) ?? null

  const orientationOf = (): 'horizontal' | 'vertical' =>
    hostOf()?.getAttribute('orientation') === 'vertical' ? 'vertical' : 'horizontal'

  const isH = (): boolean => orientationOf() === 'horizontal'

  const setAttr = (el: Element, name: string, value: string): void => {
    if (el.getAttribute(name) !== value) el.setAttribute(name, value)
  }

  const stamp = (): void => {
    const orientation = orientationOf()
    // Handle Style match/compound keys on this attr (not the host's).
    setAttr(handle, 'orientation', orientation)
    setAttr(handle, 'role', 'separator')
    setAttr(handle, 'aria-orientation', orientation)
    setAttr(handle, 'aria-valuemin', '0')
    setAttr(handle, 'aria-valuemax', '100')
    if (!handle.hasAttribute('aria-valuenow')) setAttr(handle, 'aria-valuenow', '50')
  }

  const applyPct = (pct: number): void => {
    const pane = startOf()
    const host = hostOf()
    if (!pane) return
    let next = Math.min(100, Math.max(0, pct))
    if (host) {
      const hostRect = host.getBoundingClientRect()
      const handleRect = handle.getBoundingClientRect()
      const total = isH() ? hostRect.width : hostRect.height
      const handleSize = isH() ? handleRect.width : handleRect.height
      if (total > 0) {
        next = (clampPx((next / 100) * total, total, handleSize) / total) * 100
      }
    }
    pane.style.flexGrow = '0'
    pane.style.flexShrink = '0'
    pane.style.flexBasis = `${next}%`
    setAttr(handle, 'aria-valuenow', String(Math.round(next)))
  }

  const clampPx = (raw: number, total: number, handleSize: number): number => {
    const max = Math.max(MIN_PANE_PX, total - handleSize - MIN_PANE_PX)
    return Math.min(max, Math.max(MIN_PANE_PX, raw))
  }

  const currentPct = (): number => {
    const pane = startOf()
    const host = hostOf()
    if (!pane || !host) return 50
    const hostRect = host.getBoundingClientRect()
    const total = isH() ? hostRect.width : hostRect.height
    if (total <= 0) return 50
    const paneRect = pane.getBoundingClientRect()
    const size = isH() ? paneRect.width : paneRect.height
    return (size / total) * 100
  }

  let dragging = false
  let grab = 0

  const onDown = (e: PointerEvent): void => {
    if (e.button !== 0) return
    const host = hostOf()
    if (!host || !startOf()) return
    e.preventDefault()
    dragging = true
    const handleRect = handle.getBoundingClientRect()
    grab = isH() ? e.clientX - handleRect.left : e.clientY - handleRect.top
    if (typeof handle.setPointerCapture === 'function' && e.pointerId != null) {
      try {
        handle.setPointerCapture(e.pointerId)
      } catch {
        /* jsdom / detached */
      }
    }
    window.addEventListener('pointermove', onMove)
    window.addEventListener('pointerup', onUp)
    window.addEventListener('pointercancel', onUp)
  }

  const onMove = (e: PointerEvent): void => {
    if (!dragging) return
    const host = hostOf()
    if (!host) return
    const hostRect = host.getBoundingClientRect()
    const handleRect = handle.getBoundingClientRect()
    const total = isH() ? hostRect.width : hostRect.height
    const handleSize = isH() ? handleRect.width : handleRect.height
    if (total <= 0) return
    const pos = isH() ? e.clientX - hostRect.left : e.clientY - hostRect.top
    const startPx = clampPx(pos - grab, total, handleSize)
    applyPct((startPx / total) * 100)
  }

  const onUp = (e: PointerEvent): void => {
    if (!dragging) return
    dragging = false
    if (typeof handle.releasePointerCapture === 'function' && e.pointerId != null) {
      try {
        handle.releasePointerCapture(e.pointerId)
      } catch {
        /* already released */
      }
    }
    window.removeEventListener('pointermove', onMove)
    window.removeEventListener('pointerup', onUp)
    window.removeEventListener('pointercancel', onUp)
  }

  const onKey = (e: KeyboardEvent): void => {
    const host = hostOf()
    if (!host || !startOf()) return
    const horiz = isH()
    const key = e.key
    let delta = 0
    const step = e.shiftKey ? NUDGE_LARGE : NUDGE
    if (horiz && key === 'ArrowLeft') delta = -step
    else if (horiz && key === 'ArrowRight') delta = step
    else if (!horiz && key === 'ArrowUp') delta = -step
    else if (!horiz && key === 'ArrowDown') delta = step
    else if (key === 'Home') {
      e.preventDefault()
      applyPct(0)
      return
    } else if (key === 'End') {
      e.preventDefault()
      applyPct(100)
      return
    } else {
      return
    }
    e.preventDefault()
    const hostRect = host.getBoundingClientRect()
    const handleRect = handle.getBoundingClientRect()
    const total = horiz ? hostRect.width : hostRect.height
    const handleSize = horiz ? handleRect.width : handleRect.height
    if (total <= 0) {
      applyPct(currentPct() + delta)
      return
    }
    const next = currentPct() + delta
    const startPx = clampPx((next / 100) * total, total, handleSize)
    applyPct((startPx / total) * 100)
  }

  stamp()
  handle.addEventListener('pointerdown', onDown)
  handle.addEventListener('keydown', onKey)

  const host = hostOf()
  const obs = host
    ? new MutationObserver(() => {
        stamp()
      })
    : null
  obs?.observe(host!, { attributes: true, attributeFilter: ['orientation'] })

  return {
    destroy: () => {
      handle.removeEventListener('pointerdown', onDown)
      handle.removeEventListener('keydown', onKey)
      window.removeEventListener('pointermove', onMove)
      window.removeEventListener('pointerup', onUp)
      window.removeEventListener('pointercancel', onUp)
      obs?.disconnect()
    },
  }
}
