// inView — fire a callback when the host enters the viewport.
// `use:inView={onEnter}`
//
// IntersectionObserver wrapper for reveal-on-scroll, lazy sections, and
// analytics beacons. The callback receives the entry; the observer stays
// attached so re-entry fires again (callers can self-gate if once-only).

import type { Action } from '../action'

export type InViewHandler = (entry: IntersectionObserverEntry) => void

/**
 * Observe the host; invoke `onEnter` whenever it becomes intersecting.
 * Environments without `IntersectionObserver` (jsdom without a stub) are a
 * silent no-op so SSR / tests stay quiet.
 *
 * `use:inView={(entry) => reveal()}`.
 */
export const inView: Action<InViewHandler | undefined> = (node, onEnter) => {
  if (typeof IntersectionObserver === 'undefined') {
    return {
      update: (fn) => {
        onEnter = fn
      },
    }
  }

  const observer = new IntersectionObserver((entries) => {
    for (const entry of entries) {
      if (entry.isIntersecting) onEnter?.(entry)
    }
  })
  observer.observe(node)

  return {
    update: (fn) => {
      onEnter = fn
    },
    destroy: () => {
      observer.disconnect()
    },
  }
}
