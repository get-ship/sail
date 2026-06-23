import { controller, derived } from '@ship/sui-runtime'
import { createChartData } from '../engine/create-chart-data'
import { normalizeSeries } from '../engine/normalize'
import { cellGeometry } from '../engine/geometry-grid'
import { parseLabels } from '../chart-util'
import type { ChartSeries, ChartValue, ChartPoint } from '../engine/types'
import type { Scope } from './sui-heatmap.sui'

// `error` arrives via `prop:error={…}` identity (no decodable <prop> for
// `unknown`); read off the host in `mount` and patch the engine.
interface HeatmapHost extends Element {
  error?: unknown
}

/** Read the y of a flat value or an explicit point. */
function yOf(entry: ChartValue | ChartPoint): ChartValue {
  return typeof entry === 'object' && entry !== null ? entry.y : entry
}

/**
 * Resolve the canonical row-series for the grid. `normalizeSeries` already
 * yields one series per row for 2D JSON; a 1D comma string yields ONE flat
 * series, which is reshaped here into `ceil(n / cols)` rows by the cols label
 * count (preserving null gaps positionally).
 */
function toGrid(raw: string, rowLabels: string[], colLabels: string[]): ChartSeries[] {
  const series = normalizeSeries(raw)
  if (series.length === 0) return []
  // 2D JSON (or already multi-row) — one series per row, kept as-is.
  if (series.length > 1) return series

  // 1D flat series → reshape into rows by the col count.
  const flat = (series[0]!.data as Array<ChartValue | ChartPoint>).map(yOf)
  const nc = colLabels.length || 1
  const nr = rowLabels.length || (flat.length > 0 ? Math.ceil(flat.length / nc) : 0)
  const rows: ChartSeries[] = []
  for (let r = 0; r < nr; r++) {
    rows.push({ name: rowLabels[r] ?? `row-${r}`, data: flat.slice(r * nc, (r + 1) * nc) })
  }
  return rows
}

export default controller({
  scope: ({ data, rows, cols, tone, cellSize, gap, label, loading }: Scope) => {
    const grid = derived<ChartSeries[]>(() =>
      toGrid(data(), parseLabels(rows()), parseLabels(cols())),
    )
    const chart = createChartData({ series: grid, isLoading: loading })

    // Grid dimensions are cell-size driven (not the responsive viewport): the
    // box fed to cellGeometry is sized so each cell renders at exactly cell-size.
    const dims = derived(() => {
      const norm = chart.normalized()
      const nRows = norm.length
      const nCols = nRows > 0 ? Math.max(...norm.map((s) => s.data.length), 0) : 0
      const cs = cellSize()
      const gp = gap()
      return {
        nCols,
        nRows,
        w: nCols > 0 ? nCols * cs + (nCols - 1) * gp : 0,
        h: nRows > 0 ? nRows * cs + (nRows - 1) * gp : 0,
      }
    })
    const svgW = derived(() => dims().w)
    const svgH = derived(() => dims().h)

    // GEOMETRY IS A PURE DERIVED — intensity from the normalized value over the
    // tone role token (no color-mix). A null/zero cell renders empty (opacity 0).
    const cells = derived(() => {
      const d = dims()
      return cellGeometry(chart.normalized(), { w: d.w, h: d.h }, { tone: tone(), gap: gap() })
    })
    const ariaLabel = derived(() => `${label()}: ${chart.ariaSummary()}`)

    return { chart, cells, svgW, svgH, ariaLabel }
  },
  mount(_state, host) {
    const err = (host as HeatmapHost).error
    if (err !== undefined) _state.chart.setOptions({ error: err })
  },
})
