import { controller, derived, ref } from '@ship/sui-runtime'
import { createChartData } from '../engine/create-chart-data'
import { createChartViewport } from '../engine/create-chart-viewport'
import { normalizeSeries } from '../engine/normalize'
import { areaGeometry } from '../engine/geometry-cartesian'
import { gridGeometry } from '../engine/geometry-grid'
import type { Scope } from './sui-area-chart.sui'

// `error` is injected via `prop:error={…}` identity — `unknown` has no decodable
// `<prop>` type, so it is not in the compiler-emitted `Scope`. The controller
// reads it off the host in `mount` and patches the engine.
interface AreaHost extends Element {
  error?: unknown
}

export default controller({
  scope: ({ series, width, height, grid, label, loading }: Scope) => {
    const svgEl = ref<SVGSVGElement>(null) // bind:this target → viewport observe
    const viewport = createChartViewport({
      initialWidth: width(),
      initialHeight: height(),
    })
    const chart = createChartData({
      // normalizeSeries accepts string | ChartSeries[] — ONE scope key handles
      // both the SSR JSON string AND a live prop:series identity, no cast.
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

    return { chart, viewport, svgEl, seriesPaths, gridLines, ariaLabel }
  },
  mount(state, host) {
    if (state.svgEl) state.viewport.observe(state.svgEl)
    const err = (host as AreaHost).error
    if (err !== undefined) state.chart.setOptions({ error: err })
    return () => state.viewport.dispose()
  },
})
