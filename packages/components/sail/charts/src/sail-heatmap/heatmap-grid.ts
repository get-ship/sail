/** Pure grid reshape helpers for sail-heatmap. */

import { parseLabels } from '../chart-util'
import type { ChartSeries, ChartValue, ChartPoint } from '../engine/types'
import { normalizeSeries } from '../engine/normalize'

function yOf(entry: ChartValue | ChartPoint): ChartValue {
  return typeof entry === 'object' && entry !== null ? entry.y : entry
}

/**
 * `rows` / `cols` accept comma labels (`"Mon,Tue"`) or a count integer (`"3"`).
 */
export function parseAxis(raw: string, prefix: string): string[] {
  const s = String(raw ?? '').trim()
  if (!s) return []
  if (/^\d+$/.test(s)) {
    const n = Math.max(0, Math.min(64, Number(s)))
    return Array.from({ length: n }, (_, i) => `${prefix}${i}`)
  }
  return parseLabels(s)
}

/**
 * Canonical row-series for the grid. 2D JSON stays multi-series; 1D comma
 * flattens into `ceil(n / cols)` rows by col label count.
 */
export function toGrid(raw: string, rowLabels: string[], colLabels: string[]): ChartSeries[] {
  const series = normalizeSeries(raw)
  if (series.length === 0) return []
  if (series.length > 1) return series

  const flat = (series[0]!.data as Array<ChartValue | ChartPoint>).map(yOf)
  const nc = colLabels.length || 1
  const nr = rowLabels.length || (flat.length > 0 ? Math.ceil(flat.length / nc) : 0)
  const out: ChartSeries[] = []
  for (let r = 0; r < nr; r++) {
    out.push({ name: rowLabels[r] ?? `row-${r}`, data: flat.slice(r * nc, (r + 1) * nc) })
  }
  return out
}
