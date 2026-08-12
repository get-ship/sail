<!--
  <sail-area-chart> — multi-series filled area SVG. createChartData +
  createChartViewport; series identity supersedes JSON SSR prop.
-->

<scope>
  props: {
    series = ''
    labels = ''
    width = 300
    height = 200
    points = false
    grid = false
    label = 'area chart'
    loading = false
  }
</scope>

<controller>

import type { Action } from '@ship/sail-runtime'
import { createChartData } from '../engine/create-chart-data'
import { createChartViewport } from '../engine/create-chart-viewport'
import { normalizeSeries } from '../engine/normalize'
import { areaGeometry } from '../engine/geometry-cartesian'
import { gridGeometry } from '../engine/geometry-grid'
import type { Scope } from './sail-area-chart.sail'

interface AreaHost {
  error?: unknown
}

export default controller(({ series, width, height, grid, label, loading }: Scope) => {
    const viewport = createChartViewport({
      initialWidth: width(),
      initialHeight: height(),
    })
    const chart = createChartData({
      series: derived(() => normalizeSeries(series())),
      isLoading: loading,
    })

    const seriesPaths = derived(() =>
      areaGeometry(chart.normalized(), { w: viewport.width(), h: viewport.height() }),
    )
    const gridLines = derived(() =>
      grid()
        ? gridGeometry(chart.extent(), { w: viewport.width(), h: viewport.height() })
        : [],
    )
    const ariaLabel = derived(() => `${label()}: ${chart.ariaSummary()}`)

    // SVG mounts/unmounts with loading/error/empty branches — action is re-entrant.
    const observeViewport: Action<void> = (node) => {
      viewport.observe(node)
      return { destroy: () => viewport.dispose() }
    }

    mount((host) => {
      const err = (host as AreaHost).error
      if (err !== undefined) chart.setOptions({ error: err })
    })

    return { chart, viewport, observeViewport, seriesPaths, gridLines, ariaLabel }
  })

</controller>
<column gap="0" style({ box: w(full) })>
  <if {loading()}>
    <skeleton shape="block" size="lg" role="status" aria-label="Loading" aria-live="polite" aria-busy="true"></skeleton>
  {else-if {chart.error()} => <error-state></error-state>}{else-if {chart.isEmpty()} => <empty-state></empty-state>}{else => <svg
      viewBox={'0 0 ' + viewport.width() + ' ' + viewport.height()}
      height={height()}
      role="img"
      aria-label={ariaLabel()}
      aria-describedby="chart-data"
      use:observeViewport
      style({ box: display(block).w(full).overflow(hidden) })
    >
      <for {gridLines() as g} key={g.id}>
        <line x1="0" x2={viewport.width()} y1={g.y} y2={g.y} stroke="var(--border)" stroke-width="1" stroke-dasharray="2 3"></line>
      </for>
      <for {seriesPaths() as s} key={s.name}>
        <path d={s.fillD} fill={s.color} fill-opacity="0.3" stroke="none"></path>
        <path d={s.strokeD} fill="none" stroke={s.color} stroke-width="2" stroke-linecap="round" stroke-linejoin="round"></path>
        <if {points()}>
          <for {s.pts as p} key={p.id}>
            <circle cx={p.cx} cy={p.cy} r="2.5" fill={s.color}></circle>
          </for>
        </if>
      </for>
    </svg>
    <column id="chart-data" style({ paint: srOnly() })>
      <slot name="description">
        <table>
          <for {chart.ariaTable() as row} key={row[0].id}>
            <tr>
              <for {row as cell} key={cell.id}>
                <td>{cell.text}</td>
              </for>
            </tr>
          </for>
        </table>
      </slot>
    </column>}</if>
  <row gap="3" align="stretch" style({ box: mt(1_5), paint: fg(muted-foreground) })>
    <span style({ box: display(contents), text: fontFamily(body).fontSize(xs) })>
      <slot></slot>
    </span>
  </row>
</column>
