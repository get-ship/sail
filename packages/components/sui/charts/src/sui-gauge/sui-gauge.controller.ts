import { controller, derived } from '@ship/sui-runtime'
import { createChartData } from '../engine/create-chart-data'
import { gaugeGeometry } from '../engine/geometry-radial'
import type { ChartSeries } from '../engine/types'
import type { Scope } from './sui-gauge.sui'

// `error` arrives via `prop:error={…}` identity (no decodable <prop> for
// `unknown`); read off the host in `mount` and patch the engine.
interface GaugeHost extends Element {
  error?: unknown
}

export default controller({
  scope: ({
    value,
    min,
    max,
    tone,
    lowThreshold,
    highThreshold,
    size,
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
        size: size(),
        thickness: thickness(),
        min: min(),
        max: max(),
        tone: tone(),
        lowThreshold: lowThreshold(),
        highThreshold: highThreshold(),
      }),
    )
    const ariaLabel = derived(() => `${label()}: ${chart.ariaSummary()}`)

    return { chart, arc, ariaLabel }
  },
  mount(_state, host) {
    const err = (host as GaugeHost).error
    if (err !== undefined) _state.chart.setOptions({ error: err })
  },
})
