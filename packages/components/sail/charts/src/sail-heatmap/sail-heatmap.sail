<!--
  <sail-heatmap> — 2D SVG rect grid, intensity via fill-opacity on tone token.
  createChartData + cellGeometry; `values` aliases `data` for catalog demos.
-->

<scope>
  props: {
    data = ''
    /** Alias of `data` — catalog demos use `values="…"`. */
    values = ''
    rows = ''
    cols = ''
    tone: 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted'
    'cell-size' = 20
    gap = 3
    label = 'heatmap'
    loading = false
  }
</scope>

<controller>

import { createChartData } from '../engine/create-chart-data'
import { cellGeometry } from '../engine/geometry-grid'
import { parseAxis, toGrid } from './heatmap-grid'
import type { ChartSeries } from '../engine/types'
import type { Scope } from './sail-heatmap.sail'

interface HeatmapHost {
  error?: unknown
}

export default controller(({ data, values, rows, cols, tone, cellSize, gap, label, loading }: Scope) => {
    const rawData = derived(() => {
      const d = String(data() ?? '').trim()
      if (d) return d
      return String(values() ?? '').trim()
    })
    const grid = derived<ChartSeries[]>(() =>
      toGrid(rawData(), parseAxis(rows(), 'r'), parseAxis(cols(), 'c')),
    )
    const chart = createChartData({ series: grid, isLoading: loading })

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

    const cells = derived(() => {
      const d = dims()
      return cellGeometry(chart.normalized(), { w: d.w, h: d.h }, { tone: tone(), gap: gap() })
    })
    const ariaLabel = derived(() => `${label()}: ${chart.ariaSummary()}`)

    mount((host) => {
      const err = (host as HeatmapHost).error
      if (err !== undefined) chart.setOptions({ error: err })
    })

    return { chart, cells, svgW, svgH, ariaLabel }
  })

</controller>
<if {loading()}>
  <skeleton shape="block" size="lg" role="status" aria-label="Loading" aria-live="polite" aria-busy="true"></skeleton>
{else-if {chart.error()} => <error-state></error-state>}{else-if {chart.isEmpty()} => <empty-state></empty-state>}{else => <svg
    viewBox={'0 0 ' + svgW() + ' ' + svgH()}
    width={svgW()}
    height={svgH()}
    role="img"
    aria-label={ariaLabel()}
    aria-describedby="chart-data"
    style({ box: display(inline-block).overflow(visible) })
  >
    <for {cells() as cell} key={cell.id}>
      <rect x={cell.x} y={cell.y} width={cell.w} height={cell.h} rx="2" fill={cell.color} fill-opacity={cell.intensity}>
      </rect>
    </for>
</svg>
  <column id="chart-data" style({ paint: srOnly() })>
    <slot name="description">
      <table>
        <for {chart.ariaTable() as row} key={row[0].id}>
          <tr>
            <for {row as c} key={c.id}>
              <td>{c.text}</td>
            </for>
          </tr>
        </for>
      </table>
    </slot>
  </column>}</if>
