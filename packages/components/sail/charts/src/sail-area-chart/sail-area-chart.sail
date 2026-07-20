<!--
  <sail-area-chart> — multi-series filled area chart, SVG-rendered.

  MIGRATED onto the createChartData engine + createChartViewport adapter:
  geometry is a pure `derived` (present at first paint — no flush), the viewBox
  tracks the measured host box (no `preserveAspectRatio="none"` stretch), and
  loading / empty / error branch on the engine discriminators composing the
  primitives pack. Each series renders an area fill path under a stroke path
  over the y-grid lines.

  Props
  ─────────────────────────────────────────────────────────────────────────────
  series   string  reflect   JSON [{name,data,tone?}] SSR fallback; series
                             identity supersedes (normalizeSeries handles both).
  labels   string            comma-separated x labels (feeds the aria table).
  width    number  300       initial estimate — measured box drives the render.
  height   number  200       same.
  points   bool    reflect   render dots at data points.
  grid     bool    reflect   render subtle y-grid lines.
  label    string  "area chart"  aria-label kind (prefixes the data summary).
  loading  bool    reflect   loading branch (engine isLoading) → sail-skeleton.
  error    —       error identity-only (unknown has no <prop>) → sail-error-state.
  ─────────────────────────────────────────────────────────────────────────────
  Slots: (default) legend · description (described-data override)

  The SVG is the plot box: full-bleed width, `height` px tall. The legend row
  sits under it, one 6px step away.
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
import { controller, derived } from '@ship/sail-runtime'
import type { Action } from '@ship/sail-runtime'
import { createChartData } from '../engine/create-chart-data'
import { createChartViewport } from '../engine/create-chart-viewport'
import { normalizeSeries } from '../engine/normalize'
import { areaGeometry } from '../engine/geometry-cartesian'
import { gridGeometry } from '../engine/geometry-grid'
import type { Scope } from './sail-area-chart.sail'

// `error` is injected via `error={…}` identity — `unknown` has no decodable
// `<prop>` type, so it is not in the compiler-emitted `Scope`. The controller
// reads it off the host in `mount` and patches the engine.
interface AreaHost extends Element {
  error?: unknown
}

export default controller(({ series, width, height, grid, label, loading }: Scope) => {
    const viewport = createChartViewport({
      initialWidth: width(),
      initialHeight: height(),
    })
    const chart = createChartData({
      // normalizeSeries accepts string | ChartSeries[] — ONE scope key handles
      // both the SSR JSON string AND a live series identity, no cast.
      series: derived(() => normalizeSeries(series())),
      isLoading: loading, // engine reflects the consumer flag
    })

    // GEOMETRY IS A PURE DERIVED — present at first paint, no effect-into-signal.
    const seriesPaths = derived(() =>
      areaGeometry(chart.normalized(), { w: viewport.width(), h: viewport.height() }),
    )
    const gridLines = derived(() =>
      grid()
        ? gridGeometry(chart.extent(), { w: viewport.width(), h: viewport.height() })
        : [],
    )
    const ariaLabel = derived(() => `${label()}: ${chart.ariaSummary()}`)

    // The <svg> lives behind {#if}, so it mounts and unmounts as the chart
    // moves between loading/error/empty and data. The action runs on each
    // mount and disposes on each unmount — `observe()` builds a fresh
    // observer per call, so the pair is re-entrant.
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
<column gap="0" sx="w-full">
  {#if loading()}
    <span skeleton shape="block" size="lg" role="status" aria-label="Loading" aria-live="polite" aria-busy="true"></span>
  {:else if chart.error()}
    <error-state></error-state>
  {:else if chart.isEmpty()}
    <empty-state></empty-state>
  {:else}
    <svg
      viewBox={'0 0 ' + viewport.width() + ' ' + viewport.height()}
      height={height()}
      role="img"
      aria-label={ariaLabel()}
      aria-describedby="chart-data"
      sx="block w-full overflow-hidden"
      use:observeViewport>
      {#for g of gridLines() key={g.id}}
        <line x1="0" x2={viewport.width()} y1={g.y} y2={g.y} stroke="var(--border)" stroke-width="1" stroke-dasharray="2 3"></line>
      {/for}
      {#for s of seriesPaths() key={s.name}}
        <path d={s.fillD} fill={s.color} fill-opacity="0.3" stroke="none"></path>
        <path d={s.strokeD} fill="none" stroke={s.color} stroke-width="2" stroke-linecap="round" stroke-linejoin="round"></path>
        {#if points()}
          {#for p of s.pts key={p.id}}
            <circle cx={p.cx} cy={p.cy} r="2.5" fill={s.color}></circle>
          {/for}
        {/if}
      {/for}
    </svg>
    <column id="chart-data" sx="sr-only">
      <slot name="description">
        <table>
          {#for row of chart.ariaTable() key={row[0].id}}
            <tr>
              {#for cell of row key={cell.id}}
                <td>{cell.text}</td>
              {/for}
            </tr>
          {/for}
        </table>
      </slot>
    </column>
  {/if}
  <row gap="3" align="stretch" sx="mt-1_5 font-body text-xs text-muted-foreground"><slot></slot></row>
</column>
