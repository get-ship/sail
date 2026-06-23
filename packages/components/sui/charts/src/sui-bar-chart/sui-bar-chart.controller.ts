import { controller, derived, ref } from '@ship/sui-runtime'
import { createChartData } from '../engine/create-chart-data'
import { createChartViewport } from '../engine/create-chart-viewport'
import { normalizeSeries } from '../engine/normalize'
import { barGeometry } from '../engine/geometry-cartesian'
import { parseLabels } from '../chart-util'
import type { Scope } from './sui-bar-chart.sui'

// `error` arrives via `prop:error={…}` identity (no decodable <prop> for
// `unknown`); read off the host in `mount` and patch the engine.
interface BarHost extends Element {
  error?: unknown
}

export default controller({
  scope: ({ data, labels, tone, orientation, max, width, height, label, loading }: Scope) => {
    const svgEl = ref<SVGSVGElement>(null) // bind:this target → viewport observe
    const viewport = createChartViewport({
      initialWidth: width(),
      initialHeight: height(),
    })
    const chart = createChartData({
      series: derived(() => normalizeSeries(data())),
      // Bars grow from a zero baseline; `max>0` caps the y-domain, `0` = auto
      // (engine yMax-null) — reactive so changing `max` re-resolves the domain.
      yMin: 0,
      yMax: derived(() => (max() > 0 ? max() : null)),
      isLoading: loading,
    })

    // GEOMETRY IS A PURE DERIVED — present at first paint, no effect-into-signal.
    const bars = derived(() =>
      barGeometry(chart.normalized(), { w: viewport.width(), h: viewport.height() }, {
        horizontal: orientation() === 'horizontal',
        tone: tone(),
        labels: parseLabels(labels()),
      }),
    )
    const ariaLabel = derived(() => `${label()}: ${chart.ariaSummary()}`)

    return { chart, viewport, svgEl, bars, ariaLabel }
  },
  mount(state, host) {
    if (state.svgEl) state.viewport.observe(state.svgEl)
    const err = (host as BarHost).error
    if (err !== undefined) state.chart.setOptions({ error: err })
    return () => state.viewport.dispose()
  },
})
