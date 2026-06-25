// sui-draw — vector shape geometry. Maps a world-space rect (the drag AABB of
// a rectangle/ellipse/line/arrow) to an SVG path `d` string. Pure functions of
// the two drag endpoints — no signals, no DOM.
//
// Clean-vector output by default; the hand-drawn ("sketchy") aesthetic is the
// rough-adapter's job (rough-adapter.ts), the SOLE roughjs importer.

import type { Vec2 } from '../../engine/types'

export type ShapeKind = 'rect' | 'ellipse' | 'line' | 'arrow'

const n = (v: number): string => (Number.isInteger(v) ? String(v) : String(Math.round(v * 100) / 100))

/** Normalize two drag endpoints into a positive-extent rect. */
export function rectFromPoints(a: Vec2, b: Vec2): { x: number; y: number; w: number; h: number } {
  const x = Math.min(a.x, b.x)
  const y = Math.min(a.y, b.y)
  return { x, y, w: Math.abs(b.x - a.x), h: Math.abs(b.y - a.y) }
}

function rectPath(a: Vec2, b: Vec2): string {
  const r = rectFromPoints(a, b)
  return `M ${n(r.x)} ${n(r.y)} h ${n(r.w)} v ${n(r.h)} h ${n(-r.w)} Z`
}

/** A four-arc ellipse inscribed in the drag AABB. */
function ellipsePath(a: Vec2, b: Vec2): string {
  const r = rectFromPoints(a, b)
  const rx = r.w / 2
  const ry = r.h / 2
  const cx = r.x + rx
  const cy = r.y + ry
  return (
    `M ${n(cx - rx)} ${n(cy)} ` +
    `a ${n(rx)} ${n(ry)} 0 1 0 ${n(rx * 2)} 0 ` +
    `a ${n(rx)} ${n(ry)} 0 1 0 ${n(-rx * 2)} 0 Z`
  )
}

function linePath(a: Vec2, b: Vec2): string {
  return `M ${n(a.x)} ${n(a.y)} L ${n(b.x)} ${n(b.y)}`
}

/** A line with a simple V arrowhead at the `b` end. */
function arrowPath(a: Vec2, b: Vec2): string {
  const dx = b.x - a.x
  const dy = b.y - a.y
  const len = Math.hypot(dx, dy) || 1
  const ux = dx / len
  const uy = dy / len
  const head = Math.min(16, len * 0.3)
  // two barbs rotated ±150° from the shaft direction
  const ang = Math.atan2(uy, ux)
  const a1 = ang + Math.PI - Math.PI / 6
  const a2 = ang + Math.PI + Math.PI / 6
  const h1: Vec2 = { x: b.x + Math.cos(a1) * head, y: b.y + Math.sin(a1) * head }
  const h2: Vec2 = { x: b.x + Math.cos(a2) * head, y: b.y + Math.sin(a2) * head }
  return (
    `M ${n(a.x)} ${n(a.y)} L ${n(b.x)} ${n(b.y)} ` +
    `M ${n(h1.x)} ${n(h1.y)} L ${n(b.x)} ${n(b.y)} L ${n(h2.x)} ${n(h2.y)}`
  )
}

export function shapePath(kind: ShapeKind, a: Vec2, b: Vec2): string {
  switch (kind) {
    case 'rect':
      return rectPath(a, b)
    case 'ellipse':
      return ellipsePath(a, b)
    case 'line':
      return linePath(a, b)
    case 'arrow':
      return arrowPath(a, b)
  }
}
