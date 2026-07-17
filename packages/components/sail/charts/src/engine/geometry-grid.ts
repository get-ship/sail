// geometry-grid.ts — grid lines + heatmap cells (Slice 4).
//
// Pure `(extent | normalized × viewport box) → geometry`. Grid lines are the
// y-reference lines for cartesian charts; cell geometry is the heatmap grid
// (intensity driven by `fill-opacity` over `var(--<tone>)`, NOT color-mix —
// the token-clean continuous-intensity path; null cell → empty, opacity 0).

import { toneColor } from '../chart-util'
import type { Extent, NormalizedSeries } from './types'
import type { Box } from './geometry-cartesian'

export interface GridLine {
  id: string
  y: number
}

/**
 * Horizontal y-grid lines at the quarter marks of the plot box. Pure function
 * of the extent (count) and the viewport box — re-projects on resize, no
 * mutation. An empty domain yields no lines.
 */
export function gridGeometry(extent: Extent, box: Box, pad = 2): GridLine[] {
  if (extent.count === 0) return []
  const innerH = box.h - pad * 2
  return [0.25, 0.5, 0.75].map((f, i) => ({ id: `g${i}`, y: pad + innerH * f }))
}

export interface Cell {
  id: string
  x: number
  y: number
  w: number
  h: number
  color: string
  /** `0..1` intensity from the normalized value; null → 0 (empty cell). */
  intensity: number
}

export interface CellOptions {
  tone?: string
  gap?: number
}

/**
 * Heatmap grid: one cell per point across the normalized rows. Intensity is the
 * normalized `y` (a null point → 0, an empty cell). `color` is the tone role
 * token; the template binds `fill-opacity={cell.intensity}` over it — the only
 * token-clean continuous-intensity path (no color-mix literal).
 */
export function cellGeometry(rows: NormalizedSeries[], box: Box, opts: CellOptions = {}): Cell[] {
  const { gap = 2 } = opts
  const color = toneColor(opts.tone)
  const nRows = rows.length
  if (nRows === 0) return []
  const nCols = Math.max(...rows.map((r) => r.data.length), 0)
  if (nCols === 0) return []

  const cellW = (box.w - gap * (nCols - 1)) / nCols
  const cellH = (box.h - gap * (nRows - 1)) / nRows
  const cells: Cell[] = []
  for (let r = 0; r < nRows; r++) {
    const row = rows[r]!
    for (let c = 0; c < nCols; c++) {
      const pt = row.data[c]
      const intensity = pt && pt.y != null ? clamp01(pt.y) : 0
      cells.push({
        id: `c${r}-${c}`,
        x: c * (cellW + gap),
        y: r * (cellH + gap),
        w: Math.max(0, cellW),
        h: Math.max(0, cellH),
        color,
        intensity,
      })
    }
  }
  return cells
}

function clamp01(n: number): number {
  return n < 0 ? 0 : n > 1 ? 1 : n
}
