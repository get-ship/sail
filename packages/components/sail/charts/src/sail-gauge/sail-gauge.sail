<!--
  <sail-gauge> — single-value 270° arc gauge with optional tone thresholds.

  MIGRATED onto the createChartData engine + gaugeGeometry helper: the track +
  value arc + threshold color are a pure `derived` over `gaugeGeometry` (present
  at first paint), and loading / error branch on the engine discriminators
  composing the full-size primitives (gauge is full-size despite being compact).

  Props
  ─────────────────────────────────────────────────────────────────────────────
  value           number  reflect   the measured value (default 0).
  min             number  0         domain floor.
  max             number  100       domain ceiling.
  tone            union   reflect   explicit arc color; empty = resolve from thresholds.
  low-threshold   number  0         0 = unset (below → destructive).
  high-threshold  number  0         0 = unset (below → warning, above → success).
  diameter        number  140       outer diameter in px. NOT `size`: that name is
                                    the framework's xs|sm|md|lg channel, which would
                                    retype this number to a union.
  thickness       number  12        arc stroke width.
  label           string  "gauge"   aria-label kind (prefixes the data summary).
  loading         bool    reflect   loading branch (engine isLoading) → sail-skeleton.
  error           —       error identity-only → sail-error-state.
  ─────────────────────────────────────────────────────────────────────────────
  Slots: (default) center label

  The arc and the center label are one <stack>: both children occupy the same
  grid cell, so the slotted readout lands inside the arc with no positioning
  math. The stack shrink-wraps to the `diameter` box.
-->

<scope>
  props: {
    value = 0
    min = 0
    max = 100
    tone: 'default' | 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted' = 'default'
    'low-threshold' = 0
    'high-threshold' = 0
    diameter = 140
    thickness = 12
    label = 'gauge'
    loading = false
  }
</scope>

<controller>

import { createChartData } from '../engine/create-chart-data'
import { gaugeGeometry } from '../engine/geometry-radial'
import type { ChartSeries } from '../engine/types'
import type { Scope } from './sail-gauge.sail'

// `error` arrives via `error={…}` identity (no decodable <prop> for
// `unknown`); read off the host in `mount` and patch the engine.
interface GaugeHost extends Element {
  error?: unknown
}

export default controller(({
    value,
    min,
    max,
    tone,
    lowThreshold,
    highThreshold,
    diameter,
    thickness,
    label,
    loading,
  }: Scope) => {
    // The single value is lifted to a one-point series so the engine supplies
    // the same isLoading/error discriminators + ariaSummary as every chart.
    const series = derived<ChartSeries[]>(() => [{ name: 'value', data: [value()] }])
    const chart = createChartData({ series, isLoading: loading })

    // GEOMETRY IS A PURE DERIVED — track + clamped value arc + threshold color.
    const arc = derived(() =>
      gaugeGeometry(value(), {
        size: diameter(),
        thickness: thickness(),
        min: min(),
        max: max(),
        tone: tone(),
        lowThreshold: lowThreshold(),
        highThreshold: highThreshold(),
      }),
    )
    const ariaLabel = derived(() => `${label()}: ${chart.ariaSummary()}`)

    mount((host) => {
      const err = (host as GaugeHost).error
      if (err !== undefined) chart.setOptions({ error: err })
    })

    return { chart, arc, ariaLabel }
  })

</controller>
<if {loading()}>
  <skeleton shape="circle" size="lg" role="status" aria-label="Loading" aria-live="polite" aria-busy="true"></skeleton>
{else-if {chart.error()} => <error-state></error-state>}{else => <stack>
    style({ box: w(fit) })

    <svg
      viewBox={'0 0 ' + diameter() + ' ' + diameter()}
      width={diameter()}
      height={diameter()}
      role="img"
      aria-label={ariaLabel()}
>
      style({ box: display(block) })

      <path d={arc().trackD} fill="none" stroke="var(--muted)" stroke-width={thickness()} stroke-linecap="round">
      </path>
      <if {arc().fillD}>
        <path d={arc().fillD} fill="none" stroke={arc().color} stroke-width={thickness()} stroke-linecap="round">
        </path>
      </if>
    </svg>
    <center>
      style({ paint: fg(foreground) })
      <span>
        style({ box: display(contents), text: fontFamily(heading).fontWeight(semibold).fontSize(xl) })
        <slot></slot>
      </span>
</center>
  </stack>}</if>
