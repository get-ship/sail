import type { Action } from '../action'

/**
 * `use:toast` — the L1 `<toast>` behavior. Auto-dismisses after `duration` ms
 * (0 = sticky), wires the close control (`[data-toast-close]`), pauses the timer
 * while hovered/focused so a reader is never cut off, and emits
 * `sui-toast-dismiss` when it goes. Removal is the element's own job — it
 * removes itself from the region on dismiss.
 */
export const toast: Action<void> = (node) => {
  const el = node as HTMLElement
  const duration = Number(el.getAttribute('duration') ?? '') || 5000

  let timer: ReturnType<typeof setTimeout> | undefined
  let done = false

  const dismiss = (): void => {
    if (done) return
    done = true
    if (timer) clearTimeout(timer)
    el.dispatchEvent(
      new CustomEvent('sui-toast-dismiss', { bubbles: true, composed: true }),
    )
    el.remove()
  }

  const start = (): void => {
    if (done || duration <= 0) return
    if (timer) clearTimeout(timer)
    timer = setTimeout(dismiss, duration)
  }
  const pause = (): void => {
    if (timer) clearTimeout(timer)
    timer = undefined
  }

  const onClick = (e: MouseEvent): void => {
    const close = (e.target as Element | null)?.closest('[data-toast-close]')
    if (close && el.contains(close)) dismiss()
  }

  if (!el.hasAttribute('role')) el.setAttribute('role', 'status')
  if (!el.hasAttribute('aria-live')) el.setAttribute('aria-live', 'polite')

  el.addEventListener('click', onClick)
  el.addEventListener('pointerenter', pause)
  el.addEventListener('pointerleave', start)
  el.addEventListener('focusin', pause)
  el.addEventListener('focusout', start)
  start()

  return {
    destroy: () => {
      pause()
      el.removeEventListener('click', onClick)
      el.removeEventListener('pointerenter', pause)
      el.removeEventListener('pointerleave', start)
      el.removeEventListener('focusin', pause)
      el.removeEventListener('focusout', start)
    },
  }
}
