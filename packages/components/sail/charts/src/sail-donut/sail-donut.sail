<!--
  <sail-donut> — distribution donut, segments drawn as stroked arcs.

  MIGRATED onto the createChartData engine + createChartViewport adapter: the
  `[{label,value,tone?}]` JSON decodes through `normalizeSeries` (one series of
  points), segments are a pure `derived` over `arcGeometry` (present at first
  paint), and empty / loading / error branch on the engine discriminators
  composing the full-size primitives. A null-valued slice is skipped per the
  gap contract.

  Props
  ─────────────────────────────────────────────────────────────────────────────
  data       string  reflect   JSON [{label,value,tone?}] SSR fallback; prop:series
                               identity supersedes (normalizeSeries handles both).
  diameter   number  120       outer diameter in px (square viewBox). NOT `size`:
                               that name is the framework's xs|sm|md|lg channel,
                               which would retype this number to a union.
  thickness  number  16        ring stroke width.
  label      string  "donut chart"  aria-label kind (prefixes the data summary).
  loading    bool    reflect   loading branch (engine isLoading) → sail-skeleton.
  error      —       prop:error identity-only → sail-error-state.
  ─────────────────────────────────────────────────────────────────────────────
  Slots: (default) center content · description (described-data override)

  The ring and the center content are one <stack>: both children occupy the same
  grid cell, so the slotted center label lands over the hole with no positioning
  math. The stack shrink-wraps to the `diameter` box.
-->

<scope>
  props: {
    data = ''
    diameter = 120
    thickness = 16
    label = 'donut chart'
    loading = false
  }
</scope>

<controller>
import { controller, derived } from '@ship/sail-runtime'
import { createChartData } from '../engine/create-chart-data'
import { normalizeSeries } from '../engine/normalize'
import { arcGeometry } from '../engine/geometry-radial'
import type { Scope } from './sail-donut.sail'

// `error` arrives via `prop:error={…}` identity (no decodable <prop> for
// `unknown`); read off the host in `mount` and patch the engine.
interface DonutHost extends Element {
  error?: unknown
}

export default controller(({ data, diameter, thickness, label, loading }: Scope) => {
    const chart = createChartData({
      // normalizeSeries accepts string | ChartSeries[] — ONE scope key handles
      // both the SSR JSON string AND a live prop:series identity, no cast.
      series: derived(() => normalizeSeries(data())),
      isLoading: loading,
    })

    // GEOMETRY IS A PURE DERIVED — donut arcs are proportion-of-total, so the
    // helper reads raw series values (not the 0..1 extent normalization). A
    // null/non-positive slice is skipped per the gap contract.
    const segments = derived(() =>
      arcGeometry(chart.series(), { size: diameter(), thickness: thickness() }),
    )
    const ariaLabel = derived(() => `${label()}: ${chart.ariaSummary()}`)

    mount((host) => {
      const err = (host as DonutHost).error
      if (err !== undefined) chart.setOptions({ error: err })
    })

    return { chart, segments, ariaLabel }
  })

</controller>
{#if loading()}
  <span skeleton shape="circle" size="lg" role="status" aria-label="Loading" aria-live="polite" aria-busy="true"></span>
{:else if chart.error()}
  <error-state></error-state>
{:else if chart.isEmpty()}
  <empty-state></empty-state>
{:else}
  <stack sx="w-fit">
    <svg
      viewBox={'0 0 ' + diameter() + ' ' + diameter()}
      width={diameter()}
      height={diameter()}
      role="img"
      aria-label={ariaLabel()}
      aria-describedby="chart-data"
      sx="block">
      <circle cx={diameter() / 2} cy={diameter() / 2} r={(diameter() - thickness()) / 2} fill="none" stroke="var(--muted)" stroke-width={thickness()}>
      </circle>
      {#for seg of segments() key={seg.id}}
        <path d={seg.d} fill="none" stroke={seg.color} stroke-width={thickness()} stroke-linecap="butt">
          <title>{seg.title}</title>
        </path>
      {/for}
    </svg>
    <center sx="font-heading font-semibold text-lg leading-tight text-foreground pointer-events-none"><slot></slot></center>
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
  </stack>
{/if}
