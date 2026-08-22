// anchor — position a floating element (a native popover) relative to its
// trigger. `use:anchor` on the popover; the trigger is found via the native
// `popovertarget` link, so no ref is needed.
//
// Positioning is the one place native `<dialog>`/popover leaves to us. This is
// a cross-browser JS baseline (getBoundingClientRect + fixed, with a flip when
// there's no room). CSS anchor-positioning (`position-area`) is a progressive
// enhancement to layer on where supported — it can supersede this with zero JS,
// but it's Chromium-only today, so it can't be the floor for a public library.
//
// This is THE positioning implementation for the whole popover family
// (popover / hover-card / dropdown-menu / context-menu). Its placement
// vocabulary is deliberately the full set those components expose — a bare side
// aligns to the trigger's start edge, `-end` aligns to the far edge — so no
// component needs to hand-roll a competing positioner.

import type { Action } from '../action'
import { writeCssVar } from '../bind'

/**
 * Placement of the floating element relative to its trigger.
 *
 * A bare side (`bottom`) opens on that side, aligned to the trigger's START
 * edge (left for top/bottom, top for left/right). The `-end` variant aligns to
 * the FAR edge (right for top/bottom, bottom for left/right).
 */
export type AnchorPlacement =
  | 'top' | 'top-end'
  | 'bottom' | 'bottom-end'
  | 'left' | 'left-end'
  | 'right' | 'right-end'

const GAP = 4

const clamp = (v: number, lo: number, hi: number): number =>
  Math.max(lo, Math.min(v, hi))

/**
 * Position a native popover next to its trigger. Repositions on open, scroll,
 * and resize. `use:anchor` or `use:anchor={"bottom-end"}`.
 *
 * The trigger is the element whose `popovertarget` names this popover's `id`;
 * if none is found, the action is inert (the popover keeps its default spot).
 */
export const anchor: Action<AnchorPlacement | undefined> = (node, placement) => {
  const el = node as HTMLElement
  // Param wins; otherwise read the element's own `placement` attribute (how the
  // L1 <popover>/<dropdown>/<hover-card> elements express it — their templates
  // are static, so they can't pass a param), else the default side.
  let place: AnchorPlacement =
    placement ?? ((el.getAttribute('placement') as AnchorPlacement) || 'bottom')

  // Find the trigger — the element whose popovertarget names this popover.
  // L1 is light DOM: trigger and panel share the document (or page root).
  // Residual: if a rare foreign-content shadow wrapper hosts the panel, fall
  // through to that host's light children for the trigger marker.
  const trigger = (): HTMLElement | null => {
    if (!el.id) return null
    const sel = `[popovertarget="${el.id}"]`
    const root = el.getRootNode() as ParentNode & { host?: Element }
    const inRoot = root.querySelector?.<HTMLElement>(sel)
    if (inRoot) return inRoot
    return root.host?.querySelector<HTMLElement>(sel) ?? null
  }

  // `:popover-open` throws in selector engines without that pseudo (older
  // Safari, jsdom); mirror the shown-state from the toggle event and fall back.
  let openMirror = false
  const isOpen = (): boolean => {
    try {
      return el.matches(':popover-open')
    } catch {
      return openMirror
    }
  }

  const reposition = () => {
    if (!isOpen()) return
    const t = trigger()
    if (!t) return
    const r = t.getBoundingClientRect()
    const w = el.offsetWidth
    const h = el.offsetHeight
    const vw = window.innerWidth
    const vh = window.innerHeight
    const side = place.split('-')[0] // top | bottom | left | right
    const end = place.endsWith('-end')

    let top: number
    let left: number

    if (side === 'top' || side === 'bottom') {
      // Vertical axis: above/below the trigger; aligned to its left (start) or
      // right (end) edge horizontally.
      top = side === 'bottom' ? r.bottom + GAP : r.top - h - GAP
      // Block flip: overflow the chosen side but room on the other → flip.
      if (side === 'bottom' && top + h > vh && r.top - h - GAP >= 0) {
        top = r.top - h - GAP
      } else if (side === 'top' && top < 0 && r.bottom + h + GAP <= vh) {
        top = r.bottom + GAP
      }
      left = end ? r.right - w : r.left
    } else {
      // Horizontal axis: left/right of the trigger; aligned to its top (start)
      // or bottom (end) edge vertically.
      left = side === 'right' ? r.right + GAP : r.left - w - GAP
      // Inline flip.
      if (side === 'right' && left + w > vw && r.left - w - GAP >= 0) {
        left = r.left - w - GAP
      } else if (side === 'left' && left < 0 && r.right + w + GAP <= vw) {
        left = r.right + GAP
      }
      top = end ? r.bottom - h : r.top
    }

    // Geometry only — skins already declare position:fixed + margin:0.
    // Publish the computed box through the sx/skin custom-property bridge.
    // --anchor-width mirrors the trigger so listboxes can min-w-(--anchor-width)
    // without CSS anchor-size() (combobox match-trigger-width).
    writeCssVar(el, '--anchor-left', `${clamp(left, GAP, Math.max(GAP, vw - w - GAP))}px`)
    writeCssVar(el, '--anchor-top', `${clamp(top, GAP, Math.max(GAP, vh - h - GAP))}px`)
    writeCssVar(el, '--anchor-width', `${Math.max(r.width, 0)}px`)
  }

  /** Double rAF so slotted content (minW / scroll-area) has measured width before place. */
  const scheduleReposition = () => {
    requestAnimationFrame(() => {
      requestAnimationFrame(reposition)
    })
  }

  const onToggle = (e: Event) => {
    const te = e as Event & { newState?: string }
    // Prefer ToggleEvent.newState; fall back to :popover-open (some open paths omit newState).
    const opening =
      te.newState === 'open' ||
      (te.newState !== 'closed' && isOpen())
    openMirror = opening
    if (opening) scheduleReposition()
  }
  el.addEventListener('toggle', onToggle)
  // beforetoggle fires earlier — reposition as soon as the browser commits open.
  el.addEventListener('beforetoggle', onToggle)
  window.addEventListener('resize', reposition)
  window.addEventListener('scroll', reposition, true)

  // If already open when the action attaches (late-mounted menus), place immediately.
  if (isOpen()) scheduleReposition()

  return {
    update: (p) => {
      place = p ?? ((el.getAttribute('placement') as AnchorPlacement) || 'bottom')
      // Param changes re-place synchronously (geometry already measured). Open
      // uses scheduleReposition so slotted content can settle first.
      if (isOpen()) reposition()
    },
    destroy: () => {
      el.removeEventListener('toggle', onToggle)
      el.removeEventListener('beforetoggle', onToggle)
      window.removeEventListener('resize', reposition)
      window.removeEventListener('scroll', reposition, true)
      writeCssVar(el, '--anchor-left', null)
      writeCssVar(el, '--anchor-top', null)
      writeCssVar(el, '--anchor-width', null)
    },
  }
}
