<!--
  <sail-donut> — distribution donut; segments drawn as stroked arcs.
  Geometry is pure derived over arcGeometry + createChartData.
  Loading / empty / error compose skeleton / empty-state / error-state.
  Default slot = center content (stack overlay); description overrides aria table.
  `diameter` is not named `size` (that is the framework xs|sm|md|lg channel).
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

import { createChartData } from '../engine/create-chart-data'
import { normalizeSeries } from '../engine/normalize'
import { arcGeometry } from '../engine/geometry-radial'
import type { Scope } from './sail-donut.sail'

// `error` is identity-only (`unknown` has no decodable <prop>); read on mount.
interface DonutHost extends Element {
  error?: unknown
}

export default controller(({ data, diameter, thickness, label, loading }: Scope) => {
    const chart = createChartData({
      // normalizeSeries accepts string | ChartSeries[] (SSR JSON + live identity).
      series: derived(() => normalizeSeries(data())),
      isLoading: loading,
    })

    // Proportion-of-total arcs; null/non-positive slices skipped (gap contract).
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
}{else-if {chart.isEmpty()} =>
  <empty-state></empty-state>
}{else =>
  <stack style({ box: w(fit) })>
    <svg
      viewBox={'0 0 ' + diameter() + ' ' + diameter()}
      width={diameter()}
      height={diameter()}
      role="img"
      aria-label={ariaLabel()}
      aria-describedby="chart-data"
      style({ box: display(block) })
    >
      <circle
        cx={diameter() / 2}
        cy={diameter() / 2}
        r={(diameter() - thickness()) / 2}
        fill="none"
        stroke="var(--muted)"
        stroke-width={thickness()}
      ></circle>
      <for {segments() as seg} key={seg.id}>
        <path
          d={seg.d}
          fill="none"
          stroke={seg.color}
          stroke-width={thickness()}
          stroke-linecap="butt"
        >
          <title>{seg.title}</title>
        </path>
      </for>
    </svg>
    <center style({ paint: fg(foreground).pointerEvents(none) })>
      <span style({
        box: display(contents),
        text: fontFamily(heading).fontWeight(semibold).fontSize(lg).leading(tight),
      })>
        <slot></slot>
      </span>
    </center>
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
    </column>
  </stack>
}</if>
