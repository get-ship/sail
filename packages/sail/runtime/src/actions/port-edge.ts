import { writeCssVar } from '../bind'
import type { Action } from '../action'

/**
 * `use:portEdge={side}` — anchor a fixed-size port element to the MIDPOINT
 * of one of its parent's four edges.
 *
 * The CSS form this replaces:
 *
 *     .port {
 *       position: absolute;
 *       width: var(--space-3);
 *       height: var(--space-3);
 *     }
 *     .port[data-side='left']  { left: calc(var(--space-3) * -0.5); top: 50%; transform: translateY(-50%); }
 *     .port[data-side='right'] { right: calc(var(--space-3) * -0.5); top: 50%; transform: translateY(-50%); }
 *     .port[data-side='top']   { top: calc(var(--space-3) * -0.5); left: 50%; transform: translateX(-50%); }
 *     .port[data-side='bottom'] { bottom: calc(var(--space-3) * -0.5); left: 50%; transform: translateX(-50%); }
 *
 * The `top: 50%` / `transform: translateY(-50%)` trick only collapses when
 * the PARENT's height is known to CSS at layout time. A port is rendered
 * INSIDE a node whose size is runtime data (the board model reports the
 * rect, not the pixels), so `top` has to derive from the parent's centerline
 * by measuring. A name — not `use:style` — because the algorithm IS the
 * contract (a port is "the parent-edge midpoint minus half-self"), not "an
 * arbitrary transform expression". Addable and deprecatable in the catalogue.
 *
 * Writes six custom properties keyed by side:
 *
 *     --port-top    --port-left    --port-right   --port-bottom
 *
 * the action FOLDS the centering into the absolute offset (e.g. for
 * `left` it sets `top: parent_h/2 - self_h/2`, `left: -self_w/2`). Then sx
 * just reads the matching property:
 *
 *     style="absolute w-3 h-3 rounded-full bg-background border border-primary cursor-crosshair state-left:left-(--port-left) state-left:top-(--port-top) state-right:right-(--port-right) state-right:top-(--port-top) state-top:top-(--port-top) state-top:left-(--port-left) state-bottom:bottom-(--port-bottom) state-bottom:left-(--port-left)"
 *
 * `null`/missing side (or unmount) clears all four properties so the port's
 * natural flow takes over — the same null-vs-empty distinction `rect` uses
 * for "0 vs off".
 *
 * A `ResizeObserver` on the parent catches the rare case where the node
 * reflows; the cleanup disconnects it.
 */
export type PortSide = 'left' | 'right' | 'top' | 'bottom' | null | undefined

const KEYS = ['--port-top', '--port-left', '--port-right', '--port-bottom'] as const

const writeEdge = (
  el: HTMLElement,
  side: PortSide,
  parentW: number,
  parentH: number,
  selfW: number,
  selfH: number,
): void => {
  if (!side) {
    for (const k of KEYS) writeCssVar(el, k, null)
    return
  }
  const halfW = selfW / 2
  const halfH = selfH / 2
  // The center of the port should sit ON the parent's edge midpoint. The
  // absolute top/left offsets are written relative to the parent's content
  // box (the % in CSS resolves there).
  switch (side) {
    case 'left': {
      writeCssVar(el, '--port-top', `${parentH / 2 - halfH}px`)
      writeCssVar(el, '--port-left', `${-halfW}px`)
      writeCssVar(el, '--port-right', null)
      writeCssVar(el, '--port-bottom', null)
      break
    }
    case 'right': {
      writeCssVar(el, '--port-top', `${parentH / 2 - halfH}px`)
      writeCssVar(el, '--port-left', null)
      writeCssVar(el, '--port-right', `-${halfW}px`)
      writeCssVar(el, '--port-bottom', null)
      break
    }
    case 'top': {
      writeCssVar(el, '--port-top', `${-halfH}px`)
      writeCssVar(el, '--port-left', `${parentW / 2 - halfW}px`)
      writeCssVar(el, '--port-right', null)
      writeCssVar(el, '--port-bottom', null)
      break
    }
    case 'bottom': {
      writeCssVar(el, '--port-bottom', `-${halfH}px`)
      writeCssVar(el, '--port-left', `${parentW / 2 - halfW}px`)
      writeCssVar(el, '--port-right', null)
      writeCssVar(el, '--port-top', null)
      break
    }
  }
}

export const portEdge: Action<PortSide> = (node, initial) => {
  const el = node as HTMLElement
  let observer: ResizeObserver | null = null
  let currentSide: PortSide = initial
  const recompute = (): void => {
    if (!currentSide) {
      writeEdge(el, currentSide, 0, 0, 0, 0)
      return
    }
    const parent = el.parentElement
    if (!parent) {
      writeEdge(el, currentSide, 0, 0, 0, 0)
      return
    }
    const pr = parent.getBoundingClientRect()
    const sr = el.getBoundingClientRect()
    writeEdge(el, currentSide, pr.width, pr.height, sr.width, sr.height)
  }
  const startObserve = (): void => {
    if (typeof ResizeObserver === 'undefined') return
    observer = new ResizeObserver(recompute)
    const parent = el.parentElement
    if (parent) observer.observe(parent)
  }
  startObserve()
  recompute()
  return {
    update(next) {
      currentSide = next
      if (next && !observer) startObserve()
      recompute()
    },
    destroy() {
      observer?.disconnect()
      observer = null
      for (const k of KEYS) writeCssVar(el, k, null)
    },
  }
}
