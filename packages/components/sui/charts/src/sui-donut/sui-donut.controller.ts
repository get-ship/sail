import { controller, derived } from '@ship/sui-runtime'
import { createChartData } from '../engine/create-chart-data'
import { normalizeSeries } from '../engine/normalize'
import { arcGeometry } from '../engine/geometry-radial'
import type { Scope } from './sui-donut.sui'

// `error` arrives via `prop:error={…}` identity (no decodable <prop> for
// `unknown`); read off the host in `mount` and patch the engine.
interface DonutHost extends Element {
  error?: unknown
}

export default controller({
  scope: ({ data, size, thickness, label, loading }: Scope) => {
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
      arcGeometry(chart.series(), { size: size(), thickness: thickness() }),
    )
    const ariaLabel = derived(() => `${label()}: ${chart.ariaSummary()}`)

    return { chart, segments, ariaLabel }
  },
  mount(_state, host) {
    const err = (host as DonutHost).error
    if (err !== undefined) _state.chart.setOptions({ error: err })
  },
})
