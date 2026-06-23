import { controller, derived, ref } from '@ship/sui-runtime'
import { createChartData } from '../engine/create-chart-data'
import { createChartViewport } from '../engine/create-chart-viewport'
import { normalizeSeries } from '../engine/normalize'
import { areaGeometry } from '../engine/geometry-cartesian'
import { toneColor } from '../chart-util'
import type { Scope } from './sui-sparkline.sui'

// `error` arrives via `prop:error={…}` identity (no decodable <prop> for
// `unknown`); read off the host in `mount` and patch the engine.
interface SparkHost extends Element {
  error?: unknown
}

export default controller({
  scope: ({ data, tone, width, height, label, loading }: Scope) => {
    const svgEl = ref<SVGSVGElement>(null) // bind:this target → viewport observe
    const viewport = createChartViewport({
      initialWidth: width(),
      initialHeight: height(),
    })
    const chart = createChartData({
      series: derived(() => normalizeSeries(data())),
      isLoading: loading,
    })

    // GEOMETRY IS A PURE DERIVED — areaGeometry yields both fillD + strokeD so
    // one derivation serves the fill-on and fill-off template branches. The
    // single series' tone overrides the comma-string default (toneColor map).
    const seriesPaths = derived(() => {
      const t = tone()
      return areaGeometry(chart.normalized(), {
        w: viewport.width(),
        h: viewport.height(),
      }).map((p) => ({ ...p, color: toneColor(t) }))
    })
    const ariaLabel = derived(() => `${label()}: ${chart.ariaSummary()}`)

    return { chart, viewport, svgEl, seriesPaths, ariaLabel }
  },
  mount(state, host) {
    if (state.svgEl) state.viewport.observe(state.svgEl)
    const err = (host as SparkHost).error
    if (err !== undefined) state.chart.setOptions({ error: err })
    return () => state.viewport.dispose()
  },
})
