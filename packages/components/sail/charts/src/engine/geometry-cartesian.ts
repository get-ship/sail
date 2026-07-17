// geometry-cartesian.ts — line / area / bar projections (Slice 4).
//
// Pure `(normalized series × viewport box) → SVG geometry`. Reuses `linePath`
// from `chart-util`. Null-gap behavior per family:
//   - line/area: a `null` SPLITS the stroke into one sub-path per contiguous
//     run of finite points (break, not interpolate, not zero).
//   - bar: a `null` category is OMITTED (no rect).
//
// Normalized y is `0..1` of the y-domain (0 = bottom, 1 = top); SVG y grows
// downward, so the projection inverts: `pixelY = pad + innerH * (1 - y)`.

import { toneColor } from '../chart-util'
import type { NormalizedSeries, ChartValue } from './types'

export interface Box {
  w: number
  h: number
}

export interface LinePoint {
  id: string
  cx: number
  cy: number
}

export interface SeriesPath {
  name: string
  /** One `d` sub-path per contiguous finite run (joined for a single `path`). */
  strokeD: string
  color: string
  pts: LinePoint[]
}

/**
 * Project normalized series (`y ∈ 0..1`, shared y-domain) to stroke paths,
 * splitting the stroke into a fresh `M…` sub-path at each null gap. Single-point
 * runs center on `w/2` (the existing `linePath` single-point convention).
 */
export function lineGeometry(
  series: NormalizedSeries[],
  box: Box,
  pad = 2,
): SeriesPath[] {
  const out: SeriesPath[] = []
  for (const s of series) {
    const color = toneColor(s.tone)
    const len = s.data.length
    const runs = splitRuns(s.data.map((p) => p.y))
    let d = ''
    const pts: LinePoint[] = []
    for (const run of runs) {
      const segs = run.map((idx) => {
        const single = len === 1
        const cx = single ? box.w / 2 : projectX(idx, len, box.w)
        const cy = projectY(s.data[idx]!.y as number, box.h, pad)
        return { idx, cx, cy }
      })
      d +=
        segs
          .map((p, k) => `${k === 0 ? 'M' : 'L'}${p.cx.toFixed(2)},${p.cy.toFixed(2)}`)
          .join(' ') + ' '
      for (const seg of segs) pts.push({ id: `${s.name}-${seg.idx}`, cx: seg.cx, cy: seg.cy })
    }
    if (!d.trim()) continue
    out.push({ name: s.name, strokeD: d.trim(), color, pts })
  }
  return out
}

export interface AreaPath extends SeriesPath {
  /** Fill `d` (each finite run closed to the baseline). */
  fillD: string
}

/** Line + area-fill geometry; each finite run closes to the baseline. */
export function areaGeometry(series: NormalizedSeries[], box: Box, pad = 2): AreaPath[] {
  const lines = lineGeometry(series, box, pad)
  return lines.map((lp) => {
    // Group the line points back into runs by contiguity in id index order.
    const baseline = box.h - pad
    let fillD = ''
    if (lp.pts.length) {
      const first = lp.pts[0]!
      const last = lp.pts[lp.pts.length - 1]!
      fillD = `${lp.strokeD} L${last.cx.toFixed(2)},${baseline.toFixed(2)} L${first.cx.toFixed(2)},${baseline.toFixed(2)} Z`
    }
    return { ...lp, fillD }
  })
}

export interface Bar {
  id: string
  x: number
  y: number
  w: number
  h: number
  rx: number
  ry: number
  color: string
  title: string
}

export interface BarOptions {
  horizontal?: boolean
  tone?: string
  labels?: string[]
  gap?: number
  radius?: number
}

/**
 * Categorical bars from the first series' normalized points. A `null` category
 * is omitted (no rect). Bar height is `normalized.y` of the plot height, so the
 * tallest finite value reaches the full plot extent under an auto-domain.
 */
export function barGeometry(series: NormalizedSeries[], box: Box, opts: BarOptions = {}): Bar[] {
  const first = series[0]
  if (!first) return []
  const { horizontal = false, gap = 4, radius = 3 } = opts
  const color = toneColor(opts.tone)
  const labels = opts.labels ?? []
  const n = first.data.length
  if (n === 0) return []

  const bars: Bar[] = []
  for (let i = 0; i < n; i++) {
    const pt = first.data[i]!
    if (pt.y == null) continue // null category omitted
    const ratio = clamp01(pt.y)
    let x: number, y: number, barW: number, barH: number
    if (horizontal) {
      barH = (box.h - gap * (n - 1)) / n
      y = i * (barH + gap)
      barW = ratio * box.w
      x = 0
    } else {
      barW = (box.w - gap * (n - 1)) / n
      x = i * (barW + gap)
      barH = ratio * box.h
      y = box.h - barH
    }
    const rawText = pt.raw == null ? '' : String(pt.raw)
    bars.push({
      id: `b${i}`,
      x,
      y,
      w: Math.max(0, barW),
      h: Math.max(0, barH),
      rx: radius,
      ry: radius,
      color,
      title: `${labels[i] ?? rawText}: ${rawText}`,
    })
  }
  return bars
}

/** Contiguous runs of finite indices (gap-aware split). */
function splitRuns(ys: ChartValue[]): number[][] {
  const runs: number[][] = []
  let current: number[] = []
  ys.forEach((y, i) => {
    if (y == null) {
      if (current.length) runs.push(current)
      current = []
    } else {
      current.push(i)
    }
  })
  if (current.length) runs.push(current)
  return runs
}

function projectX(i: number, len: number, w: number): number {
  if (len <= 1) return w / 2
  return (i / (len - 1)) * w
}

/** Normalized `y ∈ 0..1` → pixel y (SVG y grows down; 1 = top, inset by pad). */
function projectY(y: number, h: number, pad: number): number {
  const innerH = h - pad * 2
  return pad + innerH - clamp01(y) * innerH
}

function clamp01(n: number): number {
  return n < 0 ? 0 : n > 1 ? 1 : n
}
