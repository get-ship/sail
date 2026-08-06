import type { Action } from '../action'

/**
 * `use:rect={box}` — a positioned element's box (left/top/width/height).
 *
 * Concrete uses:
 * - <sail-graph-canvas> nodes: a graph layout placed each node at (left, top)
 *   with a measured or author-supplied (width, height). Numbers in px; not
 *   tokens (a tier-3 graph layout returns `x: 217.4` because the algorithm
 *   says so, not because of design).
 * - <sail-annotation> rubber-band drag and region rectangles: same shape.
 * - <sail-annotation> popover placement: (left, top) only — width/height
 *   ride the natural box.
 *
 * Reader atoms in sx:
 *     style="top-(--box-top) left-(--box-left) w-(--box-w) h-(--box-h)"
 *
 * The action distinguishes "null = off" vs "0 = present-but-empty" the same
 * way the css-token rule would: null removes the property so the element's
 * natural positioning takes over.
 */
export interface BoxRect {
  top?: number | string | null
  left?: number | string | null
  width?: number | string | null
  height?: number | string | null
}

const toValue = (v: number | string | null | undefined): string | null => {
  if (v == null) return null
  if (typeof v === 'number') return Number.isFinite(v) ? `${v}px` : null
  return v === '' ? null : v
}

export const rect: Action<BoxRect | null | undefined> = (node, initial) => {
  const el = node as HTMLElement
  const apply = (box: BoxRect | null | undefined): void => {
    if (!box) {
      el.style.removeProperty('--box-top')
      el.style.removeProperty('--box-left')
      el.style.removeProperty('--box-w')
      el.style.removeProperty('--box-h')
      return
    }
    const set = (name: string, v: number | string | null | undefined): void => {
      const val = toValue(v)
      if (val == null) el.style.removeProperty(name)
      else el.style.setProperty(name, val)
    }
    set('--box-top', box.top)
    set('--box-left', box.left)
    set('--box-w', box.width)
    set('--box-h', box.height)
  }
  apply(initial)
  return {
    update: (next) => apply(next),
  }
}

/**
 * `use:pin={point}` — a 0×0 positioned element (an annotation pin). Distinct
 * from `rect` only because the values are unambiguously a POINT (no width or
 * height) — the naming is the intent the lint recovers in review.
 */
export interface PinPoint {
  x: number
  y: number
}

export const pin: Action<PinPoint | null | undefined> = (node, initial) => {
  const el = node as HTMLElement
  const apply = (p: PinPoint | null | undefined): void => {
    if (!p) {
      el.style.removeProperty('--pin-x')
      el.style.removeProperty('--pin-y')
      return
    }
    el.style.setProperty('--pin-x', Number.isFinite(p.x) ? `${p.x}px` : '0')
    el.style.setProperty('--pin-y', Number.isFinite(p.y) ? `${p.y}px` : '0')
  }
  apply(initial)
  return {
    update: (next) => apply(next),
  }
}
