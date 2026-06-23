// Backbone #1 — `createChartData()` engine specs (TESTS-FIRST, RED).
//
// The engine owns the reactive `series → extent → normalized` derivation and
// exposes it as `Derived<…>` signals, plus the centralized `isEmpty`/
// `isLoading`/`error` discriminators and the `ariaSummary`/`ariaTable`
// accessible-data derivations. Pure `@ship/sui-runtime`: signal/derived/batch
// only — no DOM, no ship.*, no fetch, no effect-into-signal.
//
// Build-spec: Backbone #1 "API sketch" + "Signal graph" + Slice 3.

import { describe, expect, it } from 'vitest'
import { signal, derived } from '@ship/sui-runtime'
import { createChartData } from './create-chart-data'
import type { ChartSeries } from './types'

const S = (data: Array<number | null>, name = 'a'): ChartSeries => ({ name, data })

describe('createChartData — normalized derivation', () => {
  it('exposes the normalized series as a Derived', () => {
    const chart = createChartData({ series: [S([1, 2, 3])] })
    expect(typeof chart.normalized).toBe('function')
    expect(chart.normalized()).toHaveLength(1)
  })

  it('maps the max value to the top of the y-domain (1)', () => {
    const chart = createChartData({ series: [S([2, 10, 100])] })
    const ys = chart.normalized()[0]!.data.map((p) => p.y)
    expect(Math.max(...(ys.filter((y) => y != null) as number[]))).toBeCloseTo(1, 5)
  })

  it('maps the min value to the bottom of the y-domain (0)', () => {
    const chart = createChartData({ series: [S([2, 10, 100])] })
    const ys = chart.normalized()[0]!.data.map((p) => p.y)
    expect(Math.min(...(ys.filter((y) => y != null) as number[]))).toBeCloseTo(0, 5)
  })

  it('scales numerically, not lexically (100 ranks above 10 above 2)', () => {
    const chart = createChartData({ series: [S([2, 10, 100])] })
    const ys = chart.normalized()[0]!.data.map((p) => p.y) as number[]
    expect(ys[2]!).toBeGreaterThan(ys[1]!)
    expect(ys[1]!).toBeGreaterThan(ys[0]!)
  })

  it('preserves null as a gap in the normalized output', () => {
    const chart = createChartData({ series: [S([1, null, 3])] })
    expect(chart.normalized()[0]!.data[1]!.y).toBeNull()
  })
})

describe('createChartData — extent', () => {
  it('derives yMin/yMax/count from finite points', () => {
    const chart = createChartData({ series: [S([5, 1, 9])] })
    expect(chart.extent().yMin).toBe(1)
    expect(chart.extent().yMax).toBe(9)
    expect(chart.extent().count).toBe(3)
  })

  it('ignores null points in extent and count', () => {
    const chart = createChartData({ series: [S([5, null, 9])] })
    expect(chart.extent().count).toBe(2)
    expect(chart.extent().yMin).toBe(5)
    expect(chart.extent().yMax).toBe(9)
  })

  it('honors an explicit yMin/yMax override', () => {
    const chart = createChartData({ series: [S([2, 10])], yMin: 0, yMax: 100 })
    expect(chart.extent().yMin).toBe(0)
    expect(chart.extent().yMax).toBe(100)
  })

  it('treats a null override as auto-extent (bar max=0 semantics)', () => {
    const chart = createChartData({ series: [S([2, 10])], yMax: null })
    expect(chart.extent().yMax).toBe(10)
  })
})

describe('createChartData — discriminators', () => {
  it('isEmpty is true when there are no finite points', () => {
    const chart = createChartData({ series: [] })
    expect(chart.isEmpty()).toBe(true)
  })

  it('isEmpty is true when every point is null', () => {
    const chart = createChartData({ series: [S([null, null])] })
    expect(chart.isEmpty()).toBe(true)
  })

  it('isEmpty is false when at least one finite point exists', () => {
    const chart = createChartData({ series: [S([null, 4])] })
    expect(chart.isEmpty()).toBe(false)
  })

  it('isLoading reflects a consumer boolean flag', () => {
    const chart = createChartData({ series: [S([1, 2])], isLoading: true })
    expect(chart.isLoading()).toBe(true)
  })

  it('isLoading reflects a consumer signal', () => {
    const loading = signal(false)
    const chart = createChartData({ series: [S([1, 2])], isLoading: loading })
    expect(chart.isLoading()).toBe(false)
    loading.set(true)
    expect(chart.isLoading()).toBe(true)
  })

  it('error reflects a consumer error value', () => {
    const err = new Error('boom')
    const chart = createChartData({ series: [S([1, 2])], error: err })
    expect(chart.error()).toBe(err)
  })

  it('error is falsy when no error is set', () => {
    const chart = createChartData({ series: [S([1, 2])] })
    expect(chart.error()).toBeFalsy()
  })
})

describe('createChartData — reactive suffix recompute', () => {
  it('recomputes extent when the series signal changes', () => {
    const series = signal<ChartSeries[]>([S([1, 2, 3])])
    const chart = createChartData({ series })
    expect(chart.extent().yMax).toBe(3)
    series.set([S([1, 2, 3, 50])])
    expect(chart.extent().yMax).toBe(50)
  })

  it('does not recompute extent when only isLoading flips', () => {
    const loading = signal(false)
    const series = signal<ChartSeries[]>([S([1, 2, 3])])
    let extentRuns = 0
    const chart = createChartData({ series, isLoading: loading })
    const watch = derived(() => {
      extentRuns++
      return chart.extent()
    })
    watch() // prime
    const before = extentRuns
    loading.set(true)
    watch()
    expect(extentRuns).toBe(before)
  })
})

describe('createChartData — accessible data', () => {
  it('ariaSummary names series count, point count, and range', () => {
    const chart = createChartData({ series: [S([0, 50, 100], 'x'), S([10, 20], 'y')] })
    const summary = chart.ariaSummary()
    expect(summary).toContain('2')
    expect(summary).toMatch(/0/)
    expect(summary).toMatch(/100/)
  })

  it('ariaTable derives rows from the normalized series', () => {
    const chart = createChartData({ series: [S([1, 2, 3])] })
    expect(Array.isArray(chart.ariaTable())).toBe(true)
    expect(chart.ariaTable().length).toBeGreaterThan(0)
  })
})

describe('createChartData — setOptions live reconfig', () => {
  it('re-resolves the domain when yMax is patched', () => {
    const chart = createChartData({ series: [S([2, 10])] })
    expect(chart.extent().yMax).toBe(10)
    chart.setOptions({ yMax: 200 })
    expect(chart.extent().yMax).toBe(200)
  })
})
