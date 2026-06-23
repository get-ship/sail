// geometry-radial.ts — donut segments + gauge arc (Slice 4).
//
// Pure `(series | scalar × viewport box) → SVG arc geometry`. Reuses `arcPath`
// from `chart-util`. Null-gap behavior:
//   - donut: a `null` (or non-positive) value SKIPS that slice — it contributes
//     nothing to the total and renders no arc.
//   - gauge: a single scalar; below-min clamps to 0 (no fill), above-max clamps
//     to the full 270° sweep.
//
// Radial geometry is proportion-of-total (donut) or value-in-[min,max] (gauge),
// NOT the shared y-extent normalization — so these helpers read raw values off
// the canonical series rather than the engine's `normalized` 0..1 mapping.

import { arcPath, toneColor } from '../chart-util'
import type { ChartSeries, ChartValue, ChartPoint } from './types'

const TWO_PI = 2 * Math.PI
// Cap a full-circle sweep just short of 2π so a lone 100% segment's start and
// end points don't coincide (a coincident-endpoint arc renders nothing).
const FULL_SWEEP = TWO_PI - 0.001
const START_TOP = -Math.PI / 2

export interface DonutOptions {
  /** Outer box (square). */
  size: number
  /** Stroke width of the ring. */
  thickness: number
  /** Fallback tone when a slice carries none. */
  tone?: string
}

export interface DonutSegment {
  id: string
  d: string
  color: string
  title: string
}

/** Read the y of a flat value or an explicit point. */
function yOf(entry: ChartValue | ChartPoint): ChartValue {
  return typeof entry === 'object' && entry !== null ? entry.y : entry
}

/** Read the label of a point entry (falls back to the index). */
function labelOf(entry: ChartValue | ChartPoint, index: number): string {
  if (typeof entry === 'object' && entry !== null && typeof entry.x === 'string') {
    return entry.x
  }
  return String(index)
}

/**
 * Donut segments from the first series' points. Each finite positive value is
 * one stroked arc whose sweep is its share of the positive total; a `null` or
 * non-positive value is skipped (no arc). An all-empty series yields `[]`.
 */
export function arcGeometry(series: ChartSeries[], opts: DonutOptions): DonutSegment[] {
  const first = series[0]
  if (!first) return []
  const entries = first.data as Array<ChartValue | ChartPoint>

  const total = entries.reduce<number>((acc, e) => {
    const v = yOf(e)
    return acc + (v != null && v > 0 ? v : 0)
  }, 0)
  if (total <= 0) return []

  const { size, thickness } = opts
  const r = (size - thickness) / 2
  const cx = size / 2
  const cy = size / 2
  const seriesTone = first.tone

  const segments: DonutSegment[] = []
  let angle = START_TOP
  entries.forEach((entry, i) => {
    const v = yOf(entry)
    if (v == null || v <= 0) return // null / non-positive slice skipped
    const rawSweep = (v / total) * TWO_PI
    const sweep = rawSweep >= FULL_SWEEP ? FULL_SWEEP : rawSweep
    const d = arcPath(cx, cy, r, angle, angle + sweep)
    angle += rawSweep
    const tone =
      typeof entry === 'object' && entry !== null && 'tone' in entry
        ? (entry as { tone?: string }).tone
        : seriesTone
    segments.push({
      id: `s${i}`,
      d,
      color: toneColor(tone ?? opts.tone),
      title: `${labelOf(entry, i)}: ${v}`,
    })
  })
  return segments
}

export interface GaugeOptions {
  size: number
  thickness: number
  min: number
  max: number
  /** Explicit tone — empty resolves from the thresholds. */
  tone?: string
  /** `0` = unset. */
  lowThreshold: number
  /** `0` = unset. */
  highThreshold: number
}

export interface GaugeArc {
  trackD: string
  /** Empty when the clamped ratio is 0 (no fill path). */
  fillD: string
  color: string
}

const DEG = Math.PI / 180
// 270° arc: 135° → 405° (= 45° past a full turn, clockwise past the bottom gap).
const GAUGE_START = 135 * DEG
const GAUGE_END = 405 * DEG

/**
 * Single-value gauge: a full 270° track plus a value arc from the start to the
 * clamped `(value-min)/(max-min)` fraction. The fill color resolves from the
 * explicit tone, else the threshold bands (below-low → destructive, below-high
 * → warning, else success when any threshold is set, else primary).
 */
export function gaugeGeometry(value: number, opts: GaugeOptions): GaugeArc {
  const { size, thickness, min, max, lowThreshold: low, highThreshold: high } = opts
  const r = (size - thickness) / 2 - 2
  const cx = size / 2
  const cy = size / 2

  const trackD = arcPath(cx, cy, r, GAUGE_START, GAUGE_END)
  const ratio = Math.max(0, Math.min(1, (value - min) / (max - min || 1)))
  const endAngle = GAUGE_START + (GAUGE_END - GAUGE_START) * ratio
  const fillD = ratio > 0 ? arcPath(cx, cy, r, GAUGE_START, endAngle) : ''

  let resolved = opts.tone ?? ''
  if (!resolved) {
    if (low > 0 && value < low) resolved = 'destructive'
    else if (high > 0 && value < high) resolved = 'warning'
    else if (low > 0 || high > 0) resolved = 'success'
    else resolved = 'primary'
  }

  return { trackD, fillD, color: toneColor(resolved) }
}
