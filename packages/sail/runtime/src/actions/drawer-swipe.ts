// drawerSwipe — non-modal <drawer> dismiss: swipe toward the docked edge,
// Escape, and light-dismiss. Internal L1 stamp (not a public `use:`).
//
// Native <dialog> via command="show" is non-modal: no top layer, no
// ::backdrop, no platform ESC. This action is the dismissible counterpart
// of bindOpen's modal opts. Swipe yields to an inner scroller that can
// still move in the close direction.

import type { Action } from '../action'
import { writeCssVar } from '../bind'
import { invokeCallback } from '@ship/sail-runtime/internal'

type Axis = 'x' | 'y'

const LOCK_PX = 10
const CLOSE_PX = 80
const CLOSE_RATIO = 0.28

function isDismissible(el: HTMLElement): boolean {
  // Prefer the bound JS prop (bool host attr) when present.
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const prop = (el as any).dismissible
  if (typeof prop === 'boolean') return prop
  const raw = el.getAttribute('dismissible')
  if (raw == null) return true
  return raw !== 'false'
}

function isOpen(el: HTMLElement): boolean {
  if (el instanceof HTMLDialogElement) return el.open
  return el.hasAttribute('open')
}

function closeVector(el: HTMLElement): { axis: Axis; sign: 1 | -1 } {
  const side = el.getAttribute('side') || 'end'
  const rtl = getComputedStyle(el).direction === 'rtl'
  switch (side) {
    case 'start':
      return { axis: 'x', sign: rtl ? 1 : -1 }
    case 'top':
      return { axis: 'y', sign: -1 }
    case 'bottom':
      return { axis: 'y', sign: 1 }
    case 'end':
    default:
      return { axis: 'x', sign: rtl ? -1 : 1 }
  }
}

function canScrollInDir(
  host: HTMLElement,
  target: EventTarget | null,
  axis: Axis,
  sign: 1 | -1,
): boolean {
  let n: Element | null = target instanceof Element ? target : null
  while (n && n !== host) {
    const style = getComputedStyle(n)
    const overflow = axis === 'x' ? style.overflowX : style.overflowY
    if (overflow === 'auto' || overflow === 'scroll') {
      if (axis === 'y') {
        if (sign < 0 && n.scrollTop > 0) return true
        if (sign > 0 && n.scrollTop + n.clientHeight < n.scrollHeight - 1) return true
      } else {
        if (sign < 0 && n.scrollLeft > 0) return true
        if (sign > 0 && n.scrollLeft + n.clientWidth < n.scrollWidth - 1) return true
      }
    }
    n = n.parentElement
  }
  return false
}

function closeDrawer(el: HTMLElement): void {
  if (el instanceof HTMLDialogElement) {
    if (el.open) el.close()
    return
  }
  if (!el.hasAttribute('open')) return
  el.removeAttribute('open')
  el.dispatchEvent(new Event('close'))
}

/**
 * Swipe toward the docked edge closes a dismissible open drawer.
 * Escape and an outside pointerdown do the same — non-modal dialog
 * has no platform dismiss of its own.
 */
export const drawerSwipe: Action<void> = (node) => {
  const el = node as HTMLElement
  let startX = 0
  let startY = 0
  let tracking = false
  let locked: 'swipe' | 'ignore' | null = null
  let pid: number | null = null

  const resetDrag = (): void => {
    tracking = false
    locked = null
    pid = null
    writeCssVar(el, 'transition', '')
    writeCssVar(el, 'transform', '')
  }

  const onPointerDown = (e: PointerEvent): void => {
    if (!isOpen(el) || !isDismissible(el) || e.button !== 0) return
    tracking = true
    locked = null
    startX = e.clientX
    startY = e.clientY
    pid = e.pointerId
  }

  const onPointerMove = (e: PointerEvent): void => {
    if (!tracking || e.pointerId !== pid) return
    const dx = e.clientX - startX
    const dy = e.clientY - startY
    const { axis, sign } = closeVector(el)
    const primary = axis === 'x' ? dx : dy
    const cross = axis === 'x' ? dy : dx
    const toward = primary * sign

    if (!locked) {
      if (Math.abs(primary) < LOCK_PX && Math.abs(cross) < LOCK_PX) return
      if (toward > LOCK_PX && toward >= Math.abs(cross)) {
        if (canScrollInDir(el, e.target, axis, sign)) {
          locked = 'ignore'
          return
        }
        locked = 'swipe'
        try {
          el.setPointerCapture(e.pointerId)
        } catch {
          /* capture not available */
        }
        writeCssVar(el, 'transition', 'none')
      } else {
        locked = 'ignore'
        return
      }
    }
    if (locked !== 'swipe') return
    if (e.cancelable) e.preventDefault()
    const travel = Math.max(0, toward)
    writeCssVar(
      el,
      'transform',
      axis === 'x' ? `translateX(${sign * travel}px)` : `translateY(${sign * travel}px)`,
    )
  }

  const onPointerUp = (e: PointerEvent): void => {
    if (!tracking || e.pointerId !== pid) return
    const wasSwipe = locked === 'swipe'
    const dx = e.clientX - startX
    const dy = e.clientY - startY
    resetDrag()
    if (!wasSwipe || !isDismissible(el)) return
    const { axis, sign } = closeVector(el)
    const toward = (axis === 'x' ? dx : dy) * sign
    const box = el.getBoundingClientRect()
    const size = axis === 'x' ? box.width : box.height
    const threshold = Math.min(CLOSE_PX, size * CLOSE_RATIO)
    if (toward >= threshold) closeDrawer(el)
  }

  const onKeydown = (e: KeyboardEvent): void => {
    if (e.key !== 'Escape' || !isOpen(el) || !isDismissible(el)) return
    e.preventDefault()
    closeDrawer(el)
  }

  const onDocPointerDown = (e: PointerEvent): void => {
    if (!isOpen(el) || !isDismissible(el)) return
    const t = e.target as Node | null
    if (t && !el.contains(t)) closeDrawer(el)
  }

  const onClose = (): void => {
    invokeCallback(el, 'onClose')
    invokeCallback(el, 'onDismiss')
  }

  // LANGUAGE GAP: native invoker commands have show-modal, not show.
  // command="show" is the non-modal open we document; the action is the
  // polyfill. Prefer the CommandEvent, fall back to the click invoker.
  const openDrawer = (): void => {
    if (el instanceof HTMLDialogElement) {
      if (!el.open) el.show()
      return
    }
    el.setAttribute('open', '')
  }

  const onCommand = (e: Event): void => {
    const cmd = (e as Event & { command?: string }).command
    if (cmd === 'show' || cmd === '--show') openDrawer()
  }

  const onInvokerClick = (e: Event): void => {
    const t = e.target
    if (!(t instanceof Element)) return
    const btn = t.closest('[command][commandfor]')
    if (!btn) return
    if (btn.getAttribute('commandfor') !== el.id) return
    if (btn.getAttribute('command') !== 'show') return
    openDrawer()
  }

  el.addEventListener('pointerdown', onPointerDown)
  el.addEventListener('pointermove', onPointerMove)
  el.addEventListener('pointerup', onPointerUp)
  el.addEventListener('pointercancel', resetDrag)
  el.addEventListener('close', onClose)
  el.addEventListener('command', onCommand)
  document.addEventListener('click', onInvokerClick)
  document.addEventListener('keydown', onKeydown)
  // Arm outside-dismiss after this frame so the opening click cannot close.
  const raf = requestAnimationFrame(() => {
    document.addEventListener('pointerdown', onDocPointerDown, true)
  })

  return {
    destroy: () => {
      cancelAnimationFrame(raf)
      resetDrag()
      el.removeEventListener('pointerdown', onPointerDown)
      el.removeEventListener('pointermove', onPointerMove)
      el.removeEventListener('pointerup', onPointerUp)
      el.removeEventListener('pointercancel', resetDrag)
      el.removeEventListener('close', onClose)
      el.removeEventListener('command', onCommand)
      document.removeEventListener('click', onInvokerClick)
      document.removeEventListener('keydown', onKeydown)
      document.removeEventListener('pointerdown', onDocPointerDown, true)
    },
  }
}
