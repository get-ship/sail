// scales.ts — the pure extent → linear-scale resolution that `createChartData`
// wires into the reactive `derived` chain (Backbone #1, Slice 3).
//
// Pure functions over `ChartSeries[]`: flatten y-values, resolve the y-domain
// (honoring explicit yMin/yMax overrides, null = auto), and project each finite
// y to `0..1` (nulls preserved as gaps). The scalar math lives in `chart-util`
// (`extent`/`scaleLinear`) — this file consolidates it into series-shaped ops.

import { extent, scaleLinear } from '../chart-util'
import type {
  ChartSeries,
  ChartValue,
  ChartPoint,
  Extent,
  NormalizedSeries,
  NormalizedPoint,
} from './types'

/** Read the y of a flat value or an explicit point. */
function yOf(entry: ChartValue | ChartPoint): ChartValue {
  return typeof entry === 'object' && entry !== null ? entry.y : entry
}

/** Read the x of an entry: explicit point x, else the array index. */
function xOf(entry: ChartValue | ChartPoint, index: number): number | string {
  return typeof entry === 'object' && entry !== null ? entry.x : index
}

/** Every y across every series, in order (gaps included as `null`). */
export function flatValues(series: ChartSeries[]): ChartValue[] {
  const out: ChartValue[] = []
  for (const s of series) {
    for (const entry of s.data as Array<ChartValue | ChartPoint>) out.push(yOf(entry))
  }
  return out
}

/**
 * Resolve the y-domain + finite-point count. Explicit `yMin`/`yMax` override
 * the auto extent; `null`/`undefined` means auto (preserves bar `max=0`→auto).
 */
export function resolveExtent(
  series: ChartSeries[],
  yMin?: number | null,
  yMax?: number | null,
): Extent {
  const e = extent(flatValues(series))
  return {
    yMin: yMin == null ? e.min : yMin,
    yMax: yMax == null ? e.max : yMax,
    count: e.count,
  }
}

/** Project every series' finite y to `0..1` of the domain; nulls stay gaps. */
export function normalizeSeriesToDomain(
  series: ChartSeries[],
  domain: Extent,
): NormalizedSeries[] {
  return series.map((s) => {
    const data: NormalizedPoint[] = (s.data as Array<ChartValue | ChartPoint>).map(
      (entry, i) => {
        const raw = yOf(entry)
        return {
          x: xOf(entry, i),
          y: scaleLinear(raw, domain.yMin, domain.yMax),
          raw,
        }
      },
    )
    return { name: s.name, ...(s.tone ? { tone: s.tone } : {}), data }
  })
}
