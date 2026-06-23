// createChartData — the chart-data engine (Backbone #1, Slice 3).
//
// The thin assembler: lifts `series` to a signal, owns the reactive
// `series → extent → normalized` derivation, and exposes the centralized
// `isEmpty`/`isLoading`/`error` discriminators plus the `ariaSummary`/
// `ariaTable` accessible-data derivations — all as `Derived<…>`.
//
// PURE: imports `{ signal, derived, batch, isSignal }` from `@ship/sui-runtime`
// ONLY. No DOM, no ship.*, no fetch, no <data>, no effect-into-signal — the
// derivation is one-direction `derived`, the single fix for the per-controller
// effect-into-signal anti-pattern.

import { signal, derived, isSignal, type Signal, type Derived } from '@ship/sui-runtime'
import { resolveExtent, normalizeSeriesToDomain } from './scales'
import type { ChartSeries, Extent, NormalizedSeries, AriaCell } from './types'

type Reactive<T> = T | Signal<T> | Derived<T>

export interface ChartDataOptions {
  series: Reactive<ChartSeries[]>
  /** Domain controls — `null`/`undefined` = auto from extent (bar max=0 semantics).
   *  Accepts a reactive value so a consumer's `max` prop re-resolves the domain. */
  yMin?: Reactive<number | null>
  yMax?: Reactive<number | null>
  /** Consumer state the engine reflects (ownership stays centralized). */
  isLoading?: Reactive<boolean>
  /** Consumer-only error path (identity-only — `unknown` has no `<prop>` decoder). */
  error?: Reactive<unknown>
}

export interface ChartData {
  series: Derived<ChartSeries[]>
  extent: Derived<Extent>
  normalized: Derived<NormalizedSeries[]>
  isEmpty: Derived<boolean>
  isLoading: Derived<boolean>
  error: Derived<unknown>
  ariaSummary: Derived<string>
  ariaTable: Derived<AriaCell[][]>
  setOptions(patch: Partial<ChartDataOptions>): void
}

/** Read a `T | Signal<T> | Derived<T>` reactively (tracks when it's a signal). */
function read<T>(v: Reactive<T> | undefined, fallback: T): T {
  if (v === undefined) return fallback
  return isSignal(v) ? (v as Signal<T>)() : (v as T)
}

export function createChartData(o: ChartDataOptions): ChartData {
  // `series` lifts to a signal; a raw array is wrapped, a live signal passes
  // through so a consumer's `<data live>` drives the chain without a re-wrap.
  const seriesSig: Signal<ChartSeries[]> = isSignal(o.series)
    ? (o.series as Signal<ChartSeries[]>)
    : signal(o.series as ChartSeries[])

  // Hold the (possibly reactive) consumer sources so setOptions can re-point them.
  const yMinSrc = signal<Reactive<number | null> | undefined>(o.yMin)
  const yMaxSrc = signal<Reactive<number | null> | undefined>(o.yMax)
  const loadingSrc = signal<Reactive<boolean> | undefined>(o.isLoading)
  const errorSrc = signal<Reactive<unknown> | undefined>(o.error)

  const series = derived(() => seriesSig())
  const extent = derived(() =>
    resolveExtent(series(), read(yMinSrc(), null), read(yMaxSrc(), null)),
  )
  const normalized = derived(() => normalizeSeriesToDomain(series(), extent()))

  const isEmpty = derived(() => extent().count === 0)
  const isLoading = derived(() => read(loadingSrc(), false))
  const error = derived(() => read(errorSrc(), undefined))

  const ariaSummary = derived(() => summarize(series(), extent()))
  const ariaTable = derived(() => buildAriaTable(normalized()))

  function setOptions(patch: Partial<ChartDataOptions>): void {
    if ('series' in patch && patch.series !== undefined && !isSignal(patch.series)) {
      seriesSig.set(patch.series as ChartSeries[])
    }
    if ('yMin' in patch) yMinSrc.set(patch.yMin)
    if ('yMax' in patch) yMaxSrc.set(patch.yMax)
    if ('isLoading' in patch) loadingSrc.set(patch.isLoading)
    if ('error' in patch) errorSrc.set(patch.error)
  }

  return {
    series,
    extent,
    normalized,
    isEmpty,
    isLoading,
    error,
    ariaSummary,
    ariaTable,
    setOptions,
  }
}

/** "N series, M points, range lo–hi" — one-line accessible summary. */
function summarize(series: ChartSeries[], extent: Extent): string {
  const seriesWord = series.length === 1 ? 'series' : 'series'
  const pointWord = extent.count === 1 ? 'point' : 'points'
  if (extent.count === 0) {
    return `${series.length} ${seriesWord}, no data`
  }
  return `${series.length} ${seriesWord}, ${extent.count} ${pointWord}, range ${fmt(extent.yMin)}–${fmt(extent.yMax)}`
}

function fmt(n: number): string {
  return Number.isInteger(n) ? String(n) : n.toFixed(2)
}

/** Header row + one row per series of the raw (pre-normalization) values. */
function buildAriaTable(normalized: NormalizedSeries[]): AriaCell[][] {
  if (normalized.length === 0) return []
  const cols = Math.max(...normalized.map((s) => s.data.length), 0)
  const header: AriaCell[] = [{ text: 'Series', header: true }]
  for (let i = 0; i < cols; i++) header.push({ text: String(i + 1), header: true })
  const rows: AriaCell[][] = [header]
  for (const s of normalized) {
    const row: AriaCell[] = [{ text: s.name, header: true }]
    for (let i = 0; i < cols; i++) {
      const pt = s.data[i]
      row.push({ text: pt && pt.raw != null ? fmt(pt.raw) : '—' })
    }
    rows.push(row)
  }
  return rows
}
