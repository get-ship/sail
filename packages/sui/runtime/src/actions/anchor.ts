// anchor — position a floating element (a native popover) relative to its
// trigger. `use:anchor` on the popover; the trigger is found via the native
// `popovertarget` link, so no ref is needed.
//
// Positioning is the one place native `<dialog>`/popover leaves to us. This is
// a cross-browser JS baseline (getBoundingClientRect + fixed, with a block
// flip when there's no room). CSS anchor-positioning (`position-area`) is a
// progressive enhancement to layer on where supported — it can supersede this
// with zero JS, but it's Chromium-only today, so it can't be the floor for a
// public library.

import type { Action } from '../action'

/** Placement of the floating element relative to its trigger. */
export type AnchorPlacement = 'bottom-start' | 'bottom-end' | 'top-start' | 'top-end'

const GAP = 4

/**
 * Position a native popover under (or over) its trigger. Repositions on open,
 * scroll, and resize. `use:anchor` or `use:anchor={"bottom-end"}`.
 *
 * The trigger is the element whose `popovertarget` names this popover's `id`;
 * if none is found, the action is inert (the popover keeps its default spot).
 */
export const anchor: Action<AnchorPlacement | undefined> = (node, placement) => {
  const el = node as HTMLElement
  let place: AnchorPlacement = placement ?? 'bottom-start'

  const trigger = (): HTMLElement | null =>
    el.id ? document.querySelector<HTMLElement>(`[popovertarget="${el.id}"]`) : null

  const reposition = () => {
    if (!el.matches(':popover-open')) return
    const t = trigger()
    if (!t) return
    const r = t.getBoundingClientRect()
    const w = el.offsetWidth
    const h = el.offsetHeight
    const below = place.startsWith('bottom')
    const end = place.endsWith('end')

    let top = below ? r.bottom + GAP : r.top - h - GAP
    // Block flip: if it overflows the chosen side but fits the other, flip.
    if (below && top + h > window.innerHeight && r.top - h - GAP >= 0) top = r.top - h - GAP
    else if (!below && top < 0 && r.bottom + h + GAP <= window.innerHeight) top = r.bottom + GAP

    const left = end ? r.right - w : r.left
    el.style.position = 'fixed'
    el.style.margin = '0'
    el.style.left = `${Math.max(GAP, Math.min(left, window.innerWidth - w - GAP))}px`
    el.style.top = `${Math.max(GAP, top)}px`
  }

  const onToggle = (e: Event) => {
    if ((e as unknown as { newState?: string }).newState === 'open') {
      requestAnimationFrame(reposition)
    }
  }
  el.addEventListener('toggle', onToggle)
  window.addEventListener('resize', reposition)
  window.addEventListener('scroll', reposition, true)

  return {
    update: (p) => {
      place = p ?? 'bottom-start'
      reposition()
    },
    destroy: () => {
      el.removeEventListener('toggle', onToggle)
      window.removeEventListener('resize', reposition)
      window.removeEventListener('scroll', reposition, true)
    },
  }
}
