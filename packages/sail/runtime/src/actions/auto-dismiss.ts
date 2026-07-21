// autoDismiss — timed self-removal with hover/focus pause.
//
// Capability mix-in: reads `duration` attr (ms; default 5000; ≤0 = sticky),
// pauses the timer on pointerenter/focusin, resumes on pointerleave/focusout.
// On fire: dispatch `sail-toast-dismiss` then remove. Stacked with
// `use:dismissControl` on the L1 <toast> root. Duration comes from the attr
// only — param is void.

import type { Action } from '../action'
import { dismissSelf, isDismissed } from './dismiss-self'

function readDuration(el: HTMLElement): number {
  const raw = el.getAttribute('duration')
  if (raw == null || raw === '') return 5000
  const n = Number(raw)
  // NaN (unparseable) falls back to default; 0 stays 0 (sticky).
  return Number.isFinite(n) ? n : 5000
}

/**
 * Auto-dismiss after `duration` ms (attr). 0 / negative = sticky (no timer).
 * Pause while hovered or focused so a reader is never cut off mid-read.
 */
export const autoDismiss: Action<void> = (node) => {
  const el = node as HTMLElement
  const duration = readDuration(el)

  let timer: ReturnType<typeof setTimeout> | undefined

  const fire = (): void => {
    if (isDismissed(el)) return
    if (timer) {
      clearTimeout(timer)
      timer = undefined
    }
    dismissSelf(el)
  }

  const start = (): void => {
    if (isDismissed(el) || duration <= 0) return
    if (timer) clearTimeout(timer)
    timer = setTimeout(fire, duration)
  }
  const pause = (): void => {
    if (timer) clearTimeout(timer)
    timer = undefined
  }

  el.addEventListener('pointerenter', pause)
  el.addEventListener('pointerleave', start)
  el.addEventListener('focusin', pause)
  el.addEventListener('focusout', start)
  start()

  return {
    destroy: () => {
      pause()
      el.removeEventListener('pointerenter', pause)
      el.removeEventListener('pointerleave', start)
      el.removeEventListener('focusin', pause)
      el.removeEventListener('focusout', start)
    },
  }
}
