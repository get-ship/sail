<!--
  <sail-bar-chart> — vertical or horizontal categorical bar chart, SVG-rendered.
  Geometry is pure derived over createChartData + createChartViewport.
  `max=0` means auto domain (engine yMax-null). Loading / empty / error compose
  skeleton / empty-state / error-state.
-->

<scope>
  props: {
    data = ''
    labels = ''
    tone: 'primary' | 'info' | 'success' | 'warning' | 'danger' | 'accent' | 'muted'
    orientation: 'vertical' | 'horizontal' = 'vertical'
    width = 300
    height = 200
    max = 0
    label = 'bar chart'
    loading = false
  }
</scope>

<controller>

import type { Action } from '@ship/sail-runtime'
import { createChartData } from '../engine/create-chart-data'
import { createChartViewport } from '../engine/create-chart-viewport'
import { normalizeSeries } from '../engine/normalize'
import { barGeometry } from '../engine/geometry-cartesian'
import { parseLabels } from '../chart-util'
import type { Scope } from './sail-bar-chart.sail'

// `error` is identity-only (`unknown` has no decodable <prop>); read on mount.
interface BarHost {
  error?: unknown
}

export default controller(({ data, labels, tone, orientation, max, width, height, label, loading }: Scope) => {
    const viewport = createChartViewport({
      initialWidth: width(),
      initialHeight: height(),
    })
    const chart = createChartData({
      series: derived(() => normalizeSeries(data())),
      // Bars grow from zero; max>0 caps y-domain, 0 = auto.
      yMin: 0,
      yMax: derived(() => (max() > 0 ? max() : null)),
      isLoading: loading,
    })

    const bars = derived(() =>
      barGeometry(chart.normalized(), { w: viewport.width(), h: viewport.height() }, {
        horizontal: orientation() === 'horizontal',
        tone: tone(),
        labels: parseLabels(labels()),
      }),
    )
    const ariaLabel = derived(() => `${label()}: ${chart.ariaSummary()}`)

    // SVG mounts/unmounts with the data branch; observe re-attaches each time.
    const observeViewport: Action<void> = (node) => {
      viewport.observe(node)
      return { destroy: () => viewport.dispose() }
    }

    mount((host) => {
      const err = (host as BarHost).error
      if (err !== undefined) chart.setOptions({ error: err })
    })

    return { chart, viewport, observeViewport, bars, ariaLabel }
  })

</controller>
<if {loading()}>
  <skeleton
    shape="block"
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
  <svg
    viewBox={'0 0 ' + viewport.width() + ' ' + viewport.height()}
    height={height()}
    role="img"
    aria-label={ariaLabel()}
    aria-describedby="chart-data"
    use:observeViewport
    style({ display(block).size({ w: 100% }).overflow({ all: hidden }) })
  >
    <for {bars() as b} key={b.id}>
      <rect
        x={b.x}
        y={b.y}
        width={b.w}
        height={b.h}
        rx={b.rx}
        ry={b.ry}
        fill={b.color}
        fill-opacity="0.85"
      >
        <title>{b.title}</title>
      </rect>
    </for>
  </svg>
  <column id="chart-data" style({ srOnly() })>
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
}</if>
