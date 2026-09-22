// `normalizeSeries` — the single adapter that subsumes every on-disk chart
// input shape into the canonical `ChartSeries[]` (Backbone #1, Slice 2).
//
// Accepted inputs (branch on `typeof` / shape):
//   - `ChartSeries[]`                  → identity (prop:series={…} live ref)
//   - comma-number string  "1,2,3"     → one series (bar / sparkline)
//   - JSON `[{name,data,tone?}]` string → typed multi-series (line / area, SSR)
//   - `number[]`                       → one series (1D heatmap / sparkline)
//   - `number[][]`                     → one series per row (2D heatmap)
//   - `[{label,value,tone?}]`          → one series of points (donut)
//
// Empty string, malformed JSON, and empty arrays all return `[]` (no throw).
//
// Pure: imports only `chart-util` parse helpers + `types`. No DOM, no ship.*.

import { parseJson } from '../chart-util'
import type { ChartSeries, ChartTone, ChartValue, ChartPoint } from './types'

interface DonutSlice {
  label: string
  value: number | null
  tone?: ChartTone
}

/** The union `normalizeSeries` accepts — one scope key handles every path. */
export type NormalizableInput =
  | ChartSeries[]
  | string
  | number[]
  | number[][]
  | DonutSlice[]
  | null
  | undefined

const isObject = (v: unknown): v is Record<string, unknown> =>
  typeof v === 'object' && v !== null

const isChartSeries = (v: unknown): v is ChartSeries =>
  isObject(v) && typeof v.name === 'string' && Array.isArray(v.data)

const isDonutSlice = (v: unknown): v is DonutSlice =>
  isObject(v) &&
  typeof v.label === 'string' &&
  (typeof v.value === 'number' || v.value === null)

/** A `null`/non-finite value stays a gap; everything else coerces to number. */
function toChartValue(v: unknown): ChartValue {
  if (v === null) return null
  const n = typeof v === 'number' ? v : Number(v)
  return Number.isFinite(n) ? n : null
}

/**
 * Parse a comma-number string into `ChartValue[]`, PRESERVING each slot: an
 * empty / non-numeric slot becomes a `null` gap (not dropped). This differs
 * from `chart-util.parseNums` (which filters gaps out) because the canonical
 * series shape carries gaps positionally — `"1,2,,4"` is four points with a
 * gap at index 2, per the null-gap contract.
 */
function parseGappyNums(s: string): ChartValue[] {
  return s.split(',').map((p) => {
    const t = p.trim()
    if (t === '') return null
    const n = Number.parseFloat(t)
    return Number.isFinite(n) ? n : null
  })
}

function fromArray(input: unknown[]): ChartSeries[] {
  if (input.length === 0) return []

  // [{name,data}] — already-typed series.
  if (input.every(isChartSeries)) {
    return input.map((s) => ({
      name: s.name,
      data: (s.data as unknown[]).map(normalizePointOrValue),
      ...(s.tone ? { tone: s.tone } : {}),
    }))
  }

  // [{label,value}] — donut slices → one series of points.
  if (input.every(isDonutSlice)) {
    const tone = input.find((s) => s.tone)?.tone
    const data: ChartPoint[] = input.map((s) => ({ x: s.label, y: toChartValue(s.value) }))
    return [{ name: 'series', data, ...(tone ? { tone } : {}) }]
  }

  // number[][] — 2D grid → one series per row.
  if (input.every((row) => Array.isArray(row))) {
    return (input as unknown[][]).map((row, i) => ({
      name: `row-${i}`,
      data: row.map(toChartValue),
    }))
  }

  // number[] (with possible nulls) — 1D → one series.
  return [{ name: 'series', data: input.map(toChartValue) }]
}

/** A point entry is either a bare value or an explicit `{x,y}` point. */
function normalizePointOrValue(v: unknown): ChartValue | ChartPoint {
  if (isObject(v) && 'x' in v && 'y' in v) {
    return { x: v.x as number | string, y: toChartValue(v.y) }
  }
  return toChartValue(v)
}

export function normalizeSeries(input: NormalizableInput): ChartSeries[] {
  if (input == null) return []

  if (typeof input === 'string') {
    const trimmed = input.trim()
    if (trimmed === '') return []
    // JSON shapes start with `[` or `{`; otherwise it's a comma-number string.
    if (trimmed[0] === '[' || trimmed[0] === '{') {
      const parsed = parseJson<unknown>(trimmed, null)
      if (parsed == null) return []
      if (Array.isArray(parsed)) return fromArray(parsed)
      return []
    }
    const nums = parseGappyNums(trimmed)
    // All-gap (or no finite point) → empty; otherwise one series preserving gaps.
    return nums.some((n) => n != null) ? [{ name: 'series', data: nums }] : []
  }

  if (Array.isArray(input)) return fromArray(input)

  return []
}
