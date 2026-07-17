// TESTS-FIRST — Backbone #1, Slice 2: pagination (Feature #6).
//
// Status: RED. `./pagination` and the Slice-2 pagination wiring on `createTable` do
// not exist yet (engine is greenfield). The import below fails to resolve until Slice
// 2 ships — that import-not-found is the expected red for this contract. DO NOT add
// the impl here. DO NOT weaken these into passing stubs.
//
// Authority: build-spec §Features #6 ("Pagination (client AND manual)": client slices
// `prePaginationRows`; manual mode doesn't slice and reads `rowCount`/`pageCount`
// signals; same `nextPage`/`setPageSize`/`getCanNextPage` in both), §"Signal graph"
// (`pagination: {pageIndex, pageSize}`; pagination slices `prePaginationRows`),
// §API sketch (`setPageIndex`, `nextPage`, `previousPage`, `setPageSize`,
// `getCanNextPage`, `getCanPreviousPage`, `pageCount`, `manualPagination`, `rowCount`),
// Slice 2.
//
// Engine purity (Cardinal 4): imports ONLY @ship/sail-runtime.

import { describe, expect, it } from 'vitest'
import { signal } from '@ship/sail-runtime'
// RED: greenfield module — resolves only once Slice 2 ships.
import { createTable } from './create-table'

interface Row {
  id: string
  n: number
}

const COLUMNS = [{ id: 'n', accessorKey: 'n' as const }]

// 23 rows → with pageSize 10: pages [10, 10, 3], pageCount 3.
const ROWS: Row[] = Array.from({ length: 23 }, (_, i) => ({ id: `r${i}`, n: i }))

describe('client pagination — slices prePaginationRows', () => {
  it('first page returns the first pageSize rows', () => {
    const t = createTable<Row>({
      data: ROWS,
      columns: COLUMNS,
      getRowId: (r) => r.id,
      initialState: { pagination: { pageIndex: 0, pageSize: 10 } },
    })
    expect(t.rows().map((r) => r.id)).toEqual(ROWS.slice(0, 10).map((r) => r.id))
    // prePaginationRows is the FULL set (the pre-slice source).
    expect(t.prePaginationRows()).toHaveLength(23)
  })

  it('setPageIndex slices the right window', () => {
    const t = createTable<Row>({
      data: ROWS,
      columns: COLUMNS,
      getRowId: (r) => r.id,
      initialState: { pagination: { pageIndex: 0, pageSize: 10 } },
    })
    t.setPageIndex(2) // last page → 3 rows
    expect(t.rows().map((r) => r.id)).toEqual(['r20', 'r21', 'r22'])
  })

  it('nextPage / previousPage walk the window', () => {
    const t = createTable<Row>({
      data: ROWS,
      columns: COLUMNS,
      getRowId: (r) => r.id,
      initialState: { pagination: { pageIndex: 0, pageSize: 10 } },
    })
    t.nextPage()
    expect(t.state.pagination().pageIndex).toBe(1)
    expect(t.rows().map((r) => r.id)).toEqual(ROWS.slice(10, 20).map((r) => r.id))
    t.previousPage()
    expect(t.state.pagination().pageIndex).toBe(0)
    expect(t.rows()[0]!.id).toBe('r0')
  })

  it('setPageSize re-windows and the page-count recomputes', () => {
    const t = createTable<Row>({
      data: ROWS,
      columns: COLUMNS,
      getRowId: (r) => r.id,
      initialState: { pagination: { pageIndex: 0, pageSize: 10 } },
    })
    t.setPageSize(5)
    expect(t.state.pagination().pageSize).toBe(5)
    expect(t.rows()).toHaveLength(5)
    expect(t.getPageCount()).toBe(5) // ceil(23 / 5)
  })

  it('getPageCount = ceil(rowCount / pageSize)', () => {
    const t = createTable<Row>({
      data: ROWS,
      columns: COLUMNS,
      getRowId: (r) => r.id,
      initialState: { pagination: { pageIndex: 0, pageSize: 10 } },
    })
    expect(t.getPageCount()).toBe(3) // ceil(23 / 10)
  })
})

describe('getCanNextPage / getCanPreviousPage — honest at the edges', () => {
  it('first page: cannot go previous, can go next', () => {
    const t = createTable<Row>({
      data: ROWS,
      columns: COLUMNS,
      getRowId: (r) => r.id,
      initialState: { pagination: { pageIndex: 0, pageSize: 10 } },
    })
    expect(t.getCanPreviousPage()).toBe(false)
    expect(t.getCanNextPage()).toBe(true)
  })

  it('last page: can go previous, cannot go next', () => {
    const t = createTable<Row>({
      data: ROWS,
      columns: COLUMNS,
      getRowId: (r) => r.id,
      initialState: { pagination: { pageIndex: 2, pageSize: 10 } },
    })
    expect(t.getCanPreviousPage()).toBe(true)
    expect(t.getCanNextPage()).toBe(false)
  })

  it('nextPage at the last page is a no-op (does not overscroll)', () => {
    const t = createTable<Row>({
      data: ROWS,
      columns: COLUMNS,
      getRowId: (r) => r.id,
      initialState: { pagination: { pageIndex: 2, pageSize: 10 } },
    })
    t.nextPage()
    expect(t.state.pagination().pageIndex).toBe(2)
  })
})

describe('manual pagination — does NOT slice, reads injected totals', () => {
  it('renders all injected rows (the server page) without slicing', () => {
    // In manual mode `data` IS the server page; the engine must not re-slice it.
    const page: Row[] = ROWS.slice(0, 10)
    const t = createTable<Row>({
      data: page,
      columns: COLUMNS,
      getRowId: (r) => r.id,
      manualPagination: true,
      rowCount: 23, // server total
      pageCount: 3,
      initialState: { pagination: { pageIndex: 0, pageSize: 10 } },
    })
    expect(t.rows()).toHaveLength(10) // the whole server page, unsliced
    t.setPageIndex(1)
    expect(t.rows()).toHaveLength(10) // STILL the whole page — no client slice
  })

  it('reads rowCount / pageCount from the injected values', () => {
    const t = createTable<Row>({
      data: ROWS.slice(0, 10),
      columns: COLUMNS,
      getRowId: (r) => r.id,
      manualPagination: true,
      rowCount: 23,
      pageCount: 3,
      initialState: { pagination: { pageIndex: 0, pageSize: 10 } },
    })
    expect(t.rowCount()).toBe(23)
    expect(t.getPageCount()).toBe(3)
    expect(t.getCanNextPage()).toBe(true)
  })

  it('tracks rowCount / pageCount as live signals', () => {
    const rowCount = signal(23)
    const pageCount = signal(3)
    const t = createTable<Row>({
      data: ROWS.slice(0, 10),
      columns: COLUMNS,
      getRowId: (r) => r.id,
      manualPagination: true,
      rowCount,
      pageCount,
      initialState: { pagination: { pageIndex: 2, pageSize: 10 } },
    })
    expect(t.getCanNextPage()).toBe(false) // page 2 of 3 (0-indexed last)
    // Server reports more rows → totals grow → next becomes possible.
    rowCount.set(35)
    pageCount.set(4)
    expect(t.rowCount()).toBe(35)
    expect(t.getPageCount()).toBe(4)
    expect(t.getCanNextPage()).toBe(true)
  })
})
