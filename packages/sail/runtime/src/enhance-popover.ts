// Native popover behavior — thin sugar over the platform's Popover API.
//
// The parallel to enhance-dialog: the platform owns the hard parts — the top
// layer, light-dismiss (`popover="auto"` closes on outside-click / ESC), and
// focus return. This helper only wires a reactive open state to
// showPopover()/hidePopover() and syncs it back from the native `toggle` event,
// so author state can't desync from what the platform did. It never
// reimplements the engine.

import { effect, type Signal } from './signal'
import type { Disposer } from './bind'

/** Options for {@link bindPopover}. */
export interface PopoverOptions {
  /**
   * `manual` = no light-dismiss; the popover only closes programmatically.
   * Default (`auto`) closes on an outside click or ESC — the platform default.
   */
  manual?: boolean
}

/**
 * Two-way bind a boolean signal to a native popover's shown state.
 *
 * - signal `true`  → `el.showPopover()` (top layer, light-dismiss, ESC — native)
 * - signal `false` → `el.hidePopover()`
 * - the native `toggle` event (outside-click, ESC, or a `popovertarget`
 *   button) writes the new state back into the signal.
 *
 * The element is made a popover (`popover="auto"|"manual"`) if it isn't one.
 * `showPopover()/hidePopover()` throw in the wrong state, so each call is
 * guarded by `:popover-open`.
 *
 * Compiler target: `<div use:menu>` where `menu = usePopover()`.
 */
export function bindPopover(
  el: HTMLElement,
  sig: Signal<boolean>,
  opts: PopoverOptions = {},
): Disposer {
  if (!el.hasAttribute('popover')) {
    el.setAttribute('popover', opts.manual ? 'manual' : 'auto')
  }

  // `:popover-open` is the authoritative shown-state in modern browsers, but
  // selector engines without that pseudo (older Safari, jsdom) THROW on it.
  // Keep a mirror updated from the native `toggle` event and fall back to it,
  // so the binding degrades instead of throwing out of the effect.
  let shownMirror = false
  const isShown = (): boolean => {
    try {
      return el.matches(':popover-open')
    } catch {
      return shownMirror
    }
  }

  const dispose = effect(() => {
    const want = sig()
    const shown = isShown()
    if (want && !shown) {
      el.showPopover()
      shownMirror = true
    } else if (!want && shown) {
      el.hidePopover()
      shownMirror = false
    }
  })

  // native toggle (light-dismiss / ESC / popovertarget / programmatic) → signal
  const onToggle = (e: Event) => {
    const open = (e as unknown as { newState?: string }).newState === 'open'
    shownMirror = open
    if (sig() !== open) sig.set(open)
  }
  el.addEventListener('toggle', onToggle)

  return () => {
    dispose()
    el.removeEventListener('toggle', onToggle)
  }
}
