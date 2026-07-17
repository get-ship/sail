// Pure transform + coordinate math for the substrate. No signals, no DOM —
// just functions of a `Transform`. Every face does hit-testing, snapping,
// and rendering through `screenToWorld`/`worldToScreen`; there is no second
// coordinate transform.
//
// Convention: a screen point maps to world via `world = (screen - pan) / scale`,
// i.e. `worldToScreen(t, w) = w * scale + pan`. Pan is in screen px.

import type { Transform, Vec2, Rect } from './types'

/** Clamp a zoom factor to [min, max]. */
export function clampScale(scale: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, scale))
}

/** World point → screen point under `t`. */
export function worldToScreen(t: Transform, p: Vec2): Vec2 {
  return { x: p.x * t.scale + t.x, y: p.y * t.scale + t.y }
}

/** Screen point → world point under `t`. */
export function screenToWorld(t: Transform, p: Vec2): Vec2 {
  return { x: (p.x - t.x) / t.scale, y: (p.y - t.y) / t.scale }
}

/** Zoom to `nextScale` about a fixed screen anchor, clamped. The world
 *  point under `anchor` stays under `anchor` after the zoom. */
export function composeZoomAt(
  t: Transform,
  nextScale: number,
  anchor: Vec2,
  min: number,
  max: number,
): Transform {
  const scale = clampScale(nextScale, min, max)
  // Solve pan so worldUnder * scale + pan' = anchor, where
  // worldUnder = (anchor - pan) / t.scale.
  const worldUnder = screenToWorld(t, anchor)
  return {
    scale,
    x: anchor.x - worldUnder.x * scale,
    y: anchor.y - worldUnder.y * scale,
  }
}

/** The world-space AABB currently visible in a viewport of `size`. */
export function visibleWorldRect(t: Transform, size: { w: number; h: number }): Rect {
  const topLeft = screenToWorld(t, { x: 0, y: 0 })
  return {
    x: topLeft.x,
    y: topLeft.y,
    w: size.w / t.scale,
    h: size.h / t.scale,
  }
}

/** The transform that frames a world rect in a viewport, centered, with
 *  padding (screen px) and a max-scale clamp. */
export function fitTransform(
  rect: Rect,
  size: { w: number; h: number },
  min: number,
  max: number,
  padding = 0,
): Transform {
  const availW = Math.max(1, size.w - padding * 2)
  const availH = Math.max(1, size.h - padding * 2)
  const rawScale = rect.w > 0 && rect.h > 0 ? Math.min(availW / rect.w, availH / rect.h) : 1
  const scale = clampScale(rawScale, min, max)
  const cx = rect.x + rect.w / 2
  const cy = rect.y + rect.h / 2
  return {
    scale,
    x: size.w / 2 - cx * scale,
    y: size.h / 2 - cy * scale,
  }
}
