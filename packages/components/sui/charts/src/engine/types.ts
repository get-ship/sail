// Chart-data engine — canonical typed shapes (Backbone #1).
//
// One series shape (`ChartSeries[]`) is the floor every chart in the pack
// derives geometry from. `normalizeSeries` collapses the five on-disk input
// shapes (comma-string, JSON `[{name,data}]`, 1D/2D array, donut
// `[{label,value}]`) into this; `createChartData` derives reactive
// scales/extent/normalization from it.
//
// Pure types — no DOM, no ship.*, no runtime import.

import type { ChartTone } from '../chart-util'

export type { ChartTone }

/**
 * A single point value. `null` marks a GAP / missing point — it is NOT
 * `0` and NOT interpolated.
 *
 * GAP CONTRACT (per chart family, v1):
 *  - line/area/sparkline: a `null` SPLITS the stroke into sub-paths at the
 *    gap (break, not interpolate, not zero).
 *  - bar: a `null` category is OMITTED (no rect).
 *  - donut/gauge: a `null` value SKIPS that slice (contributes nothing).
 *  - heatmap: a `null` cell renders EMPTY (fill-opacity 0).
 *
 * Extent and `count` ignore nulls — they derive from finite points only.
 */
export type ChartValue = number | null

/** An explicit point: `x` is an index, category label, or time-ms. */
export interface ChartPoint {
  x: number | string
  y: ChartValue
}

/**
 * THE one canonical series shape every chart derives from. `data` entries are
 * each either a flat value or an explicit `{x,y}` point — the engine reads `x`
 * and `y` polymorphically (see `scales.ts`), so a series may mix bare numbers
 * with points without a cast.
 */
export interface ChartSeries {
  name: string
  data: Array<ChartValue | ChartPoint>
  tone?: ChartTone
}

/** A series whose points are normalized to the y-domain (`y ∈ 0..1`). */
export interface NormalizedPoint {
  /** Original x (index for flat series, category/label for point series). */
  x: number | string
  /** Normalized value in `0..1`, or `null` for a gap. */
  y: ChartValue
  /** The raw finite value before normalization (null for a gap). */
  raw: ChartValue
}

export interface NormalizedSeries {
  name: string
  tone?: ChartTone
  data: NormalizedPoint[]
}

/** Resolved y-domain plus the finite-point count. */
export interface Extent {
  yMin: number
  yMax: number
  /** Number of finite (non-null) points across all series. */
  count: number
}

/** One cell of the accessible described-data table. */
export interface AriaCell {
  text: string
  header?: boolean
}
