// clickOutside — fire a callback when a pointer press lands outside the element.
// `use:clickOutside={() => menu.close()}`
//
// The parameterized shape of the `use:` API: an action WITH a reactive param.
// The compiler passes the callback via the param thunk; `update` swaps it when
// the expression's dependencies change. The blessed way to do light-dismiss for
// menus, popovers, and dropdowns without a hand-wired document listener.

import type { Action } from '../action'

/**
 * Call `onOutside` when a `pointerdown` occurs outside `node` (capture phase,
 * so it beats inner handlers). The listener is added on the next microtask so
 * the same press that opened the surface doesn't immediately dismiss it.
 *
 * `use:clickOutside={() => close()}`.
 */
export const clickOutside: Action<(() => void) | undefined> = (node, onOutside) => {
  const handler = (e: Event) => {
    const target = e.target as Node | null
    if (target && !node.contains(target)) onOutside?.()
  }
  let armed = false
  const arm = () => {
    armed = true
    document.addEventListener('pointerdown', handler, true)
  }
  const raf = requestAnimationFrame(arm)
  return {
    update: (fn) => {
      onOutside = fn
    },
    destroy: () => {
      cancelAnimationFrame(raf)
      if (armed) document.removeEventListener('pointerdown', handler, true)
    },
  }
}
