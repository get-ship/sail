// sui-table engine — per-column faceted row model (Slice 3b, Feature #4).
//
// A column's facets answer "what values (and counts) are still reachable if I
// add this column to the filter?" — so the facet list MUST apply every OTHER
// column's filter but NOT its own (else it collapses to the chosen value). It
// branches off `preColumnFilteredRows` (global-filtered, PRE-column-filter), so
// the global filter narrows facets while a column's own selection does not.
//
// LAZY + MEMOIZED: nothing is computed until a `getFaceted*` accessor is first
// called; repeated reads over a stable input return the identity-equal Map. The
// cache invalidates when `preColumnFilteredRows`, the OTHER columns' filters, or
// the column set changes.
//
// MANUAL seam: under `manualFiltering` the facets come from the injected
// `facets` option (server-computed) so server-mode filter UIs have a source.
//
// PURE TypeScript — imports `untrack` from `@ship/sui-runtime` ONLY. No DOM, no
// SDK.
//
// Authority: build-spec §Features #4 ("Faceting"), §"Signal graph" (facets
// branch off preColumnFilteredRows, lazy + memoized), Slice 3b.

import { untrack, type Derived, type Signal } from '@ship/sui-runtime'
import type { CellValue, ColumnDef, ColumnFilter, Row } from './types'

/** A column's resolved cell-level predicate (for OTHER-column filtering). */
export type CellPredicate = (cell: CellValue, value: unknown) => boolean

export interface FacetingInputs<T> {
  preColumnFilteredRows: Derived<Row<T>[]>
  columnFilters: Signal<ColumnFilter[]>
  columns: ColumnDef<T>[]
  /** Resolve a column's predicate from its def + the active filter value. */
  resolvePredicate: (def: ColumnDef<T>, value: unknown) => CellPredicate
  /** Per-column grouping/accessor value (honors getGroupingValue for bucketing). */
  facetValueOf: (row: Row<T>, colId: string) => CellValue
  manualFiltering: boolean
  /** Server-injected facets (manual seam). */
  injected?: Partial<
    Record<string, { uniqueValues?: Map<CellValue, number>; minMax?: [number, number] }>
  >
}

interface FacetCache {
  rowsRef: Row<unknown>[] | null
  filtersRef: ColumnFilter[] | null
  rowModel: Row<unknown>[] | null
  uniqueValues: Map<CellValue, number> | null
  minMax: [number, number] | undefined | null
}

export interface FacetingApi<T> {
  getFacetedRowModel: (colId: string) => Row<T>[]
  getFacetedUniqueValues: (colId: string) => Map<CellValue, number>
  getFacetedMinMaxValues: (colId: string) => [number, number] | undefined
}

/**
 * Build the lazy, per-column faceting accessors. Each `getFaceted*` reads the
 * reactive sources at call time (so it tracks in a `derived`) but memoizes the
 * heavy compute behind an input-identity check, returning the SAME Map on a
 * repeated read over an unchanged input.
 */
export function buildFaceting<T>(inputs: FacetingInputs<T>): FacetingApi<T> {
  const byId = new Map(inputs.columns.map((c) => [c.id, c]))
  const caches = new Map<string, FacetCache>()

  const cacheFor = (colId: string): FacetCache => {
    let c = caches.get(colId)
    if (!c) {
      c = { rowsRef: null, filtersRef: null, rowModel: null, uniqueValues: null, minMax: null }
      caches.set(colId, c)
    }
    return c
  }

  /** The faceted row model for `colId`: preColumnFilteredRows with every OTHER
   *  column's filter applied. Lazy + memoized on (rows, filters) identity. */
  const facetedRows = (colId: string): Row<T>[] => {
    const rows = inputs.preColumnFilteredRows()
    const filters = inputs.columnFilters()
    const cache = cacheFor(colId)
    if (cache.rowModel != null && cache.rowsRef === (rows as Row<unknown>[]) && cache.filtersRef === filters) {
      return cache.rowModel as unknown as Row<T>[]
    }
    // Apply OTHER columns' active filters (skip the column's own).
    const others = filters.filter((f) => f.id !== colId)
    const model =
      others.length === 0
        ? rows
        : rows.filter((row) =>
            others.every((f) => {
              const def = byId.get(f.id)
              if (!def) return true
              return inputs.resolvePredicate(def, f.value)(row.getValue(f.id), f.value)
            }),
          )
    cache.rowsRef = rows as Row<unknown>[]
    cache.filtersRef = filters
    cache.rowModel = model as Row<unknown>[]
    // Input changed → invalidate the derived facet products.
    cache.uniqueValues = null
    cache.minMax = null
    return model
  }

  const getFacetedRowModel = (colId: string): Row<T>[] => facetedRows(colId)

  const getFacetedUniqueValues = (colId: string): Map<CellValue, number> => {
    if (inputs.manualFiltering) {
      const inj = inputs.injected?.[colId]?.uniqueValues
      if (inj) return inj
    }
    const model = facetedRows(colId)
    const cache = cacheFor(colId)
    if (cache.uniqueValues != null) return cache.uniqueValues
    // `untrack` the histogram build so the lazy compute itself adds no extra
    // subscriptions beyond the source reads `facetedRows` already tracked.
    const counts = untrack(() => {
      const map = new Map<CellValue, number>()
      for (const row of model) {
        const v = inputs.facetValueOf(row, colId)
        map.set(v, (map.get(v) ?? 0) + 1)
      }
      return map
    })
    cache.uniqueValues = counts
    return counts
  }

  const getFacetedMinMaxValues = (colId: string): [number, number] | undefined => {
    if (inputs.manualFiltering) {
      const inj = inputs.injected?.[colId]?.minMax
      if (inj) return inj
    }
    const model = facetedRows(colId)
    const cache = cacheFor(colId)
    if (cache.minMax !== null) return cache.minMax
    const result = untrack(() => {
      let min = Number.POSITIVE_INFINITY
      let max = Number.NEGATIVE_INFINITY
      let seen = false
      for (const row of model) {
        const v = row.getValue(colId)
        const n = typeof v === 'number' ? v : Number(v)
        if (Number.isNaN(n)) continue
        seen = true
        if (n < min) min = n
        if (n > max) max = n
      }
      return seen ? ([min, max] as [number, number]) : undefined
    })
    cache.minMax = result
    return result
  }

  return { getFacetedRowModel, getFacetedUniqueValues, getFacetedMinMaxValues }
}
