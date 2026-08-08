<!--
  <sail-gauge> — single-value 270° arc gauge with optional tone thresholds.
  Arc geometry is pure derived over gaugeGeometry; loading / error compose
  skeleton / error-state. Default slot is the center readout (stack overlay).
  `diameter` is not named `size` (that is the framework xs|sm|md|lg channel).
-->

<scope>
  props: {
    value = 0
    min = 0
    max = 100
    tone: 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted'
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

// `error` is identity-only (`unknown` has no decodable <prop>); read on mount.
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
    // One-point series so the engine supplies isLoading/error + ariaSummary.
    const series = derived<ChartSeries[]>(() => [{ name: 'value', data: [value()] }])
    const chart = createChartData({ series, isLoading: loading })

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
  <skeleton
    shape="circle"
    size="lg"
    role="status"
    aria-label="Loading"
    aria-live="polite"
    aria-busy="true"
  ></skeleton>
{else-if {chart.error()} =>
  <error-state></error-state>
}{else =>
  <stack style({ box: w(fit) })>
    <svg
      viewBox={'0 0 ' + diameter() + ' ' + diameter()}
      width={diameter()}
      height={diameter()}
      role="img"
      aria-label={ariaLabel()}
      style({ box: display(block) })
    >
      <path
        d={arc().trackD}
        fill="none"
        stroke="var(--muted)"
        stroke-width={thickness()}
        stroke-linecap="round"
      ></path>
      <if {arc().fillD}>
        <path
          d={arc().fillD}
          fill="none"
          stroke={arc().color}
          stroke-width={thickness()}
          stroke-linecap="round"
        ></path>
      </if>
    </svg>
    <center style({ paint: fg(foreground) })>
      <span style({
        box: display(contents),
        text: fontFamily(heading).fontWeight(semibold).fontSize(xl),
      })>
        <slot></slot>
      </span>
    </center>
  </stack>
}</if>
