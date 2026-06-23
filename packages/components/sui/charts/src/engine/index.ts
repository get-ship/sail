// Chart-data engine barrel — Backbone #1 + #2 (pure @ship/sui-runtime).
//
// One typed series shape in (`normalizeSeries`), reactive scales/extent/
// geometry out (`createChartData` + the geometry helpers), responsive box from
// one `ResizeObserver` (`createChartViewport`). No DOM, no ship.*, no fetch.

export type {
  ChartTone,
  ChartValue,
  ChartPoint,
  ChartSeries,
  NormalizedPoint,
  NormalizedSeries,
  Extent,
  AriaCell,
} from './types'

export { normalizeSeries } from './normalize'
export type { NormalizableInput } from './normalize'

export { createChartData } from './create-chart-data'
export type { ChartData, ChartDataOptions } from './create-chart-data'

export { createChartViewport } from './create-chart-viewport'
export type { ChartViewport, ChartViewportOptions } from './create-chart-viewport'

export { lineGeometry, areaGeometry, barGeometry } from './geometry-cartesian'
export type { Box, SeriesPath, AreaPath, Bar, BarOptions, LinePoint } from './geometry-cartesian'

export { gridGeometry, cellGeometry } from './geometry-grid'
export type { GridLine, Cell, CellOptions } from './geometry-grid'

export { arcGeometry, gaugeGeometry } from './geometry-radial'
export type {
  DonutOptions,
  DonutSegment,
  GaugeOptions,
  GaugeArc,
} from './geometry-radial'
