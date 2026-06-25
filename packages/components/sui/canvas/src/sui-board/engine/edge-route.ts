// sui-board — pure edge-path geometry. Maps two world-space port anchors to
// an SVG path `d` string. No signals, no DOM — just functions of the two
// endpoints. The view renders the returned `path` into an SVG <path>.

import type { Vec2 } from '../../engine/types'

export type EdgeKind = 'bezier' | 'orthogonal' | 'straight'

const n = (v: number): string => (Number.isInteger(v) ? String(v) : String(Math.round(v * 1000) / 1000))

/** A straight line between the two anchors. */
function straightPath(a: Vec2, b: Vec2): string {
  return `M ${n(a.x)} ${n(a.y)} L ${n(b.x)} ${n(b.y)}`
}

/** A cubic bezier with horizontal control handles pulled toward the midpoint
 *  x — the standard left-to-right connector curve. */
function bezierPath(a: Vec2, b: Vec2): string {
  const dx = Math.max(40, Math.abs(b.x - a.x) * 0.5)
  const c1: Vec2 = { x: a.x + dx, y: a.y }
  const c2: Vec2 = { x: b.x - dx, y: b.y }
  return `M ${n(a.x)} ${n(a.y)} C ${n(c1.x)} ${n(c1.y)} ${n(c2.x)} ${n(c2.y)} ${n(b.x)} ${n(b.y)}`
}

/** An orthogonal (right-angle) connector routed through the x-midpoint. */
function orthogonalPath(a: Vec2, b: Vec2): string {
  const midX = (a.x + b.x) / 2
  return `M ${n(a.x)} ${n(a.y)} L ${n(midX)} ${n(a.y)} L ${n(midX)} ${n(b.y)} L ${n(b.x)} ${n(b.y)}`
}

export function routeEdge(kind: EdgeKind, a: Vec2, b: Vec2): string {
  switch (kind) {
    case 'straight':
      return straightPath(a, b)
    case 'orthogonal':
      return orthogonalPath(a, b)
    case 'bezier':
      return bezierPath(a, b)
  }
}
