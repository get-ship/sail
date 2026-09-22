// sail-draw — pen-stroke geometry. Maps freehand world-space sample points to
// a filled SVG path `d` string via the vendored perfect-freehand outline.
//
// Pure geometry: no signals, no DOM. The scene model stores raw sample points
// (world coords + pressure); the view renders strokePath(points) into an SVG
// <path> fill. Heavy outline math is the perfect-freehand npm core; this
// module is the thin adapter from outline polygon → SVG path.

import { getStroke, type StrokeOptions } from 'perfect-freehand'

/** A single freehand sample: world point + optional pen pressure [0..1]. */
export interface StrokeSample {
  x: number
  y: number
  pressure?: number
}

const round = (v: number): number => Math.round(v * 100) / 100

/** Build an SVG path `d` from a perfect-freehand outline polygon. The polygon
 *  is a closed ring of [x, y] points; we emit a quadratic-smoothed loop so the
 *  filled stroke reads as a continuous nib, not a faceted polygon. */
export function outlineToPath(outline: number[][]): string {
  if (outline.length === 0) return ''
  if (outline.length < 3) {
    const a = outline[0]!
    return `M ${round(a[0]!)} ${round(a[1]!)} Z`
  }
  const d: string[] = []
  const first = outline[0]!
  d.push(`M ${round(first[0]!)} ${round(first[1]!)}`)
  for (let i = 0; i < outline.length; i++) {
    const a = outline[i]!
    const b = outline[(i + 1) % outline.length]!
    const mx = (a[0]! + b[0]!) / 2
    const my = (a[1]! + b[1]!) / 2
    d.push(`Q ${round(a[0]!)} ${round(a[1]!)} ${round(mx)} ${round(my)}`)
  }
  d.push('Z')
  return d.join(' ')
}

const DEFAULT_STROKE: StrokeOptions = {
  size: 8,
  thinning: 0.6,
  smoothing: 0.5,
  streamline: 0.5,
  simulatePressure: true,
  last: true,
}

/** Convert raw freehand samples → a filled SVG path via perfect-freehand. */
export function strokePath(samples: StrokeSample[], options?: StrokeOptions): string {
  if (samples.length === 0) return ''
  const input = samples.map((s) => [s.x, s.y, s.pressure ?? 0.5])
  const outline = getStroke(input, { ...DEFAULT_STROKE, ...options })
  return outlineToPath(outline)
}

/** The world-space AABB enclosing a set of stroke samples. */
export function strokeBounds(samples: StrokeSample[]): { x: number; y: number; w: number; h: number } {
  if (samples.length === 0) return { x: 0, y: 0, w: 0, h: 0 }
  let minX = Infinity
  let minY = Infinity
  let maxX = -Infinity
  let maxY = -Infinity
  for (const s of samples) {
    minX = Math.min(minX, s.x)
    minY = Math.min(minY, s.y)
    maxX = Math.max(maxX, s.x)
    maxY = Math.max(maxY, s.y)
  }
  return { x: minX, y: minY, w: maxX - minX, h: maxY - minY }
}
