// Backbone #1 — `normalizeSeries()` adapter specs (TESTS-FIRST, RED).
//
// `normalizeSeries` is the single adapter that subsumes the five on-disk chart
// input shapes (comma-string, JSON `[{name,data}]`, 1D array, 2D array, donut
// `[{label,value}]`) PLUS a raw JSON string and a live `ChartSeries[]` identity
// — all collapsing to one canonical `ChartSeries[]`, branching on `typeof`.
// Pure `@ship/sui-runtime` boundary: no DOM, no ship.*, no fetch.
//
// Build-spec: Backbone #1 "Features → 1. Series normalization" + Slice 2.

import { describe, expect, it } from 'vitest'
import { normalizeSeries } from './normalize'
import type { ChartSeries } from './types'

describe('normalizeSeries — one shape, many inputs', () => {
  it('passes a ChartSeries[] identity through unchanged in shape', () => {
    const input: ChartSeries[] = [{ name: 'a', data: [1, 2, 3] }]
    const out = normalizeSeries(input)
    expect(out).toHaveLength(1)
    expect(out[0]!.name).toBe('a')
    expect(out[0]!.data).toEqual([1, 2, 3])
  })

  it('parses a comma-separated number string to one series', () => {
    const out = normalizeSeries('1,2,3,4')
    expect(out).toHaveLength(1)
    expect(out[0]!.data).toEqual([1, 2, 3, 4])
  })

  it('parses a JSON [{name,data}] string to typed series', () => {
    const out = normalizeSeries('[{"name":"a","data":[1,2,3]},{"name":"b","data":[4,5,6]}]')
    expect(out).toHaveLength(2)
    expect(out[0]!.name).toBe('a')
    expect(out[1]!.data).toEqual([4, 5, 6])
  })

  it('parses a JSON [{name,data}] string preserving tone', () => {
    const out = normalizeSeries('[{"name":"a","data":[1,2],"tone":"success"}]')
    expect(out[0]!.tone).toBe('success')
  })

  it('lifts a 1D number array to one series', () => {
    const out = normalizeSeries([3, 6, 9])
    expect(out).toHaveLength(1)
    expect(out[0]!.data).toEqual([3, 6, 9])
  })

  it('lifts a 2D number array to one series per row (heatmap shape)', () => {
    const out = normalizeSeries([
      [1, 2, 3],
      [4, 5, 6],
    ])
    expect(out).toHaveLength(2)
    expect(out[0]!.data).toEqual([1, 2, 3])
    expect(out[1]!.data).toEqual([4, 5, 6])
  })

  it('collapses a donut [{label,value}] list into one series of points', () => {
    const out = normalizeSeries([
      { label: 'red', value: 10 },
      { label: 'blue', value: 20 },
    ])
    expect(out).toHaveLength(1)
    expect(out[0]!.data).toHaveLength(2)
  })

  it('preserves donut labels as point x-values', () => {
    const out = normalizeSeries([
      { label: 'red', value: 10 },
      { label: 'blue', value: 20 },
    ])
    const pts = out[0]!.data as Array<{ x: number | string; y: number | null }>
    expect(pts[0]!.x).toBe('red')
    expect(pts[0]!.y).toBe(10)
  })

  it('preserves donut tone per slice', () => {
    const out = normalizeSeries([{ label: 'red', value: 10, tone: 'destructive' }])
    expect(out[0]!.tone).toBe('destructive')
  })

  it('returns an empty array for an empty string', () => {
    expect(normalizeSeries('')).toEqual([])
  })

  it('returns an empty array for malformed JSON (no throw)', () => {
    expect(normalizeSeries('{not json')).toEqual([])
  })

  it('returns an empty array for an empty input array', () => {
    expect(normalizeSeries([])).toEqual([])
  })

  it('preserves a null gap point in a typed series', () => {
    const out = normalizeSeries('[{"name":"a","data":[1,null,3]}]')
    expect(out[0]!.data).toEqual([1, null, 3])
  })

  it('the string and array forms of the same data round-trip equal', () => {
    const fromString = normalizeSeries('5,10,15')
    const fromArray = normalizeSeries([5, 10, 15])
    expect(fromString[0]!.data).toEqual(fromArray[0]!.data)
  })
})
