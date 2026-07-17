// sui-table engine — pagination (Backbone #1, Slice 2, Feature #6).
//
// Client mode slices `prePaginationRows` into the current page window; manual
// mode does NOT slice (the `data` IS the server page) and reads the injected
// `rowCount`/`pageCount` signals. The same `nextPage`/`setPageSize`/
// `getCanNextPage` surface works in both modes — only the row-count source and
// the slicing differ.
//
//   prePaginationRows ─(client ? slice [pageIndex*pageSize, +pageSize] : pass)─► paginatedRows
//
// PURE TypeScript — imports `derived` from `@ship/sail-runtime` only. No DOM, no
// SDK.
//
// Authority: build-spec §Features #6 ("Pagination (client AND manual)"),
// §"Signal graph" (pagination slices prePaginationRows), Slice 2.

import { derived, isSignal, type Derived, type Signal } from '@ship/sail-runtime'
import type { PaginationState, Row } from './types'

/** Read a `number | Signal<number>` total, tracking the signal when present. */
function readTotal(src: number | Signal<number> | undefined): number | undefined {
  if (src == null) return undefined
  if (isSignal(src)) return (src as Signal<number>)()
  return src as number
}

export interface PaginationInputs<T> {
  prePaginationRows: Derived<Row<T>[]>
  pagination: Signal<PaginationState>
  /** Manual mode: do not slice; read injected totals instead. */
  manual: boolean
  /** Server totals (manual mode). May be a live signal. */
  rowCount?: number | Signal<number>
  pageCount?: number | Signal<number>
}

export interface PaginationStages<T> {
  paginatedRows: Derived<Row<T>[]>
  /** Total ROW count — client: prePaginationRows length; manual: injected. */
  rowCount: Derived<number>
  /** ceil(rowCount / pageSize), or the injected pageCount in manual mode. */
  pageCount: Derived<number>
  getPageCount: () => number
  getCanNextPage: () => boolean
  getCanPreviousPage: () => boolean
}

export function buildPaginationStages<T>(
  inputs: PaginationInputs<T>,
): PaginationStages<T> {
  const { prePaginationRows, pagination, manual } = inputs

  const rowCount = derived<number>(() => {
    if (manual) {
      const injected = readTotal(inputs.rowCount)
      if (injected != null) return injected
    }
    return prePaginationRows().length
  })

  const pageCount = derived<number>(() => {
    if (manual) {
      const injected = readTotal(inputs.pageCount)
      if (injected != null) return injected
    }
    const { pageSize } = pagination()
    if (pageSize <= 0) return 0
    return Math.ceil(rowCount() / pageSize)
  })

  const paginatedRows = derived<Row<T>[]>(() => {
    const rows = prePaginationRows()
    if (manual) return rows // server already paged — never re-slice
    const { pageIndex, pageSize } = pagination()
    if (pageSize <= 0) return rows
    const start = pageIndex * pageSize
    return rows.slice(start, start + pageSize)
  })

  const getPageCount = (): number => pageCount()
  const getCanPreviousPage = (): boolean => pagination().pageIndex > 0
  const getCanNextPage = (): boolean => pagination().pageIndex < pageCount() - 1

  return {
    paginatedRows,
    rowCount,
    pageCount,
    getPageCount,
    getCanNextPage,
    getCanPreviousPage,
  }
}
