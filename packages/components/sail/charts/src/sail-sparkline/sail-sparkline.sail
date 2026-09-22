<!--
  <sail-sparkline> — compact inline trend line, SVG-rendered.
  Geometry is pure derived over createChartData + createChartViewport.
  Lightweight presentation: loading uses spinner (not skeleton); empty is an
  aria-hidden empty SVG (not full-size empty-state).
-->

<scope>
  props: {
    data = ''
    tone: 'primary' | 'info' | 'success' | 'warning' | 'danger' | 'accent' | 'muted'
    width = 80
    height = 20
    fill = false
    label = 'sparkline'
    loading = false
  }
</scope>

<controller>

import type { Action } from '@ship/sail-runtime'
import { createChartData } from '../engine/create-chart-data'
import { createChartViewport } from '../engine/create-chart-viewport'
import { normalizeSeries } from '../engine/normalize'
import { areaGeometry } from '../engine/geometry-cartesian'
import { toneColor } from '../chart-util'
import type { Scope } from './sail-sparkline.sail'

// `error` is identity-only (`unknown` has no decodable <prop>); read on mount.
interface SparkHost {
  error?: unknown
}

export default controller(({ data, tone, width, height, label, loading }: Scope) => {
    const viewport = createChartViewport({
      initialWidth: width(),
      initialHeight: height(),
    })
    const chart = createChartData({
      series: derived(() => normalizeSeries(data())),
      isLoading: loading,
    })

    // areaGeometry yields fillD + strokeD so one derived serves both branches.
    const seriesPaths = derived(() => {
      const t = tone()
      return areaGeometry(chart.normalized(), {
        w: viewport.width(),
        h: viewport.height(),
      }).map((p) => ({ ...p, color: toneColor(t) }))
    })
    const ariaLabel = derived(() => `${label()}: ${chart.ariaSummary()}`)

    // SVG mounts/unmounts with the data branch; observe re-attaches each time.
    const observeViewport: Action<void> = (node) => {
      viewport.observe(node)
      return { destroy: () => viewport.dispose() }
    }

    mount((host) => {
      const err = (host as SparkHost).error
      if (err !== undefined) chart.setOptions({ error: err })
    })

    return { chart, viewport, observeViewport, seriesPaths, ariaLabel }
  })

</controller>
<if {loading()}>
  <spinner></spinner>
{else-if {chart.isEmpty()} =>
  <svg
    viewBox={'0 0 ' + viewport.width() + ' ' + viewport.height()}
    width={viewport.width()}
    height={viewport.height()}
    aria-hidden="true"
    style({ display(inline-block).overflow({ all: visible }) })
  ></svg>
}{else =>
  <svg
    viewBox={'0 0 ' + viewport.width() + ' ' + viewport.height()}
    width={viewport.width()}
    height={viewport.height()}
    role="img"
    aria-label={ariaLabel()}
    use:observeViewport
    style({ display(inline-block).overflow({ all: visible }) })
  >
    <for {seriesPaths() as s} key={s.name}>
      <if {fill()}>
        <path d={s.fillD} fill={s.color} fill-opacity="0.18" stroke="none"></path>
      </if>
      <path
        d={s.strokeD}
        fill="none"
        stroke={s.color}
        stroke-width="1.5"
        stroke-linecap="round"
        stroke-linejoin="round"
      ></path>
    </for>
  </svg>
}</if>
