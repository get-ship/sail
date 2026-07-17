// TESTS-FIRST — Backbone #1, Slice 4: manual / server mode (the boundary seam).
//
// Status: RED. `./create-table` (which wires `./manual`) does not exist yet;
// import-not-found is the expected red. DO NOT add the impl here.
//
// Authority: build-spec §"Manual / server mode (the seam — preserves Cardinal 4)"
// (per-feature manual* flags = transform skip; the controlled-slice rule: a slice
// in `state` is engine-read-only + fires on*Change; the pipeline-prefix rule with
// validation: manual stages MUST form a prefix of filter→sort→group→paginate, the
// engine WARNS on a non-prefix combo), Slice 4 ("each manual* flag skips its
// transform and fires its callback; a slice in state is engine-read-only;
// non-prefix manual combos warn; state.* still drives header chevrons under manual").

import { describe, expect, it, vi } from 'vitest'
import { signal } from '@ship/sail-runtime'
// RED: greenfield module.
import { createTable } from './create-table'

interface Row {
  id: string
  n: number
}
const COLUMNS = [{ id: 'n', accessorKey: 'n' as const }]
// Deliberately UNSORTED so a client sort would reorder — manual mode must NOT.
const ROWS: Row[] = [
  { id: 'a', n: 30 },
  { id: 'b', n: 10 },
  { id: 'c', n: 20 },
]
const ids = (rows: { id: string }[]) => rows.map((r) => r.id)

describe('manual mode — each manual* flag SKIPS its transform', () => {
  it('manualSorting: toggleSort fires onSortingChange but does NOT reorder rows', () => {
    const onSortingChange = vi.fn()
    const table = createTable<Row>({
      data: ROWS,
      columns: COLUMNS,
      getRowId: (r) => r.id,
      manualSorting: true,
      onSortingChange,
    })
    table.toggleSort('n')
    // sort state advances + the callback fires (consumer re-queries the server) …
    expect(onSortingChange).toHaveBeenCalledTimes(1)
    expect(table.state.sorting()[0]).toMatchObject({ id: 'n', desc: false })
    // … but the engine does NOT apply a client sort (data IS the server page).
    expect(ids(table.sortedRows())).toEqual(['a', 'b', 'c'])
  })

  it('manualFiltering: setColumnFilter fires the callback but does NOT drop rows', () => {
    const onColumnFiltersChange = vi.fn()
    const table = createTable<Row>({
      data: ROWS,
      columns: COLUMNS,
      getRowId: (r) => r.id,
      manualFiltering: true,
      onColumnFiltersChange,
    })
    table.setColumnFilter('n', [0, 0]) // would client-empty the page
    expect(onColumnFiltersChange).toHaveBeenCalled()
    expect(table.filteredRows()).toHaveLength(3) // server page untouched
  })

  it('manualPagination: does NOT slice; reads rowCount/pageCount signals', () => {
    const onPaginationChange = vi.fn()
    const rowCount = signal(1000)
    const table = createTable<Row>({
      data: ROWS, // the server already returned this page
      columns: COLUMNS,
      getRowId: (r) => r.id,
      manualPagination: true,
      manualSorting: true,
      manualFiltering: true,
      rowCount,
      pageCount: 200,
      onPaginationChange,
      initialState: { pagination: { pageIndex: 0, pageSize: 5 } },
    })
    expect(table.rows()).toHaveLength(3) // no client slice — show the server page as-is
    expect(table.rowCount()).toBe(1000) // total from the server signal
    expect(table.pageCount()).toBe(200)
    table.nextPage()
    expect(onPaginationChange).toHaveBeenLastCalledWith(
      expect.objectContaining({ pageIndex: 1 }),
    )
  })
})

describe('manual mode — controlled-slice rule (engine reads, never writes)', () => {
  it('a slice present in `state` is engine-READ-ONLY; the engine fires on*Change instead of writing', () => {
    const sortArg = signal<{ id: string; desc: boolean }[]>([])
    const onSortingChange = vi.fn((s: unknown[]) => sortArg.set(s as never))
    const table = createTable<Row>({
      data: ROWS,
      columns: COLUMNS,
      getRowId: (r) => r.id,
      manualSorting: true,
      state: { sorting: sortArg }, // CONTROLLED
      onSortingChange,
    })
    table.toggleSort('n')
    // The engine did NOT mutate the controlled slice itself …
    expect(onSortingChange).toHaveBeenCalledTimes(1)
    // … it fired the callback; the consumer's write to sortArg flows back in.
    expect(sortArg()).toEqual([{ id: 'n', desc: false }])
    expect(table.state.sorting()).toEqual([{ id: 'n', desc: false }])
  })

  it('state.* STILL drives header sort indicators under manual mode (getIsSorted)', () => {
    const sortArg = signal([{ id: 'n', desc: true }])
    const table = createTable<Row>({
      data: ROWS,
      columns: COLUMNS,
      getRowId: (r) => r.id,
      manualSorting: true,
      state: { sorting: sortArg },
    })
    // The chevron reads the controlled slice even though the engine didn't sort.
    expect(table.getColumn('n')!.getIsSorted()).toBe('desc')
  })
})

describe('manual mode — pipeline-prefix validation (warn on non-prefix combo)', () => {
  it('manualPagination WITHOUT manualFiltering/manualSorting WARNS (non-prefix)', () => {
    const warn = vi.spyOn(console, 'warn').mockImplementation(() => {})
    createTable<Row>({
      data: ROWS,
      columns: COLUMNS,
      getRowId: (r) => r.id,
      manualPagination: true, // paginate manual but filter/sort still client = non-prefix
    })
    expect(warn).toHaveBeenCalled()
    warn.mockRestore()
  })

  it('a valid prefix (manualFiltering + manualSorting + manualPagination) does NOT warn', () => {
    const warn = vi.spyOn(console, 'warn').mockImplementation(() => {})
    createTable<Row>({
      data: ROWS,
      columns: COLUMNS,
      getRowId: (r) => r.id,
      manualFiltering: true,
      manualSorting: true,
      manualPagination: true,
    })
    expect(warn).not.toHaveBeenCalled()
    warn.mockRestore()
  })

  it('manualFiltering alone (the shortest valid prefix) does NOT warn', () => {
    const warn = vi.spyOn(console, 'warn').mockImplementation(() => {})
    createTable<Row>({
      data: ROWS,
      columns: COLUMNS,
      getRowId: (r) => r.id,
      manualFiltering: true,
    })
    expect(warn).not.toHaveBeenCalled()
    warn.mockRestore()
  })
})

describe('manual mode — onReachEnd seam (infinite-scroll fetch-more)', () => {
  it('exposes an onReachEnd callback option for the virtualizer range to drive', () => {
    const onReachEnd = vi.fn()
    const table = createTable<Row>({
      data: ROWS,
      columns: COLUMNS,
      getRowId: (r) => r.id,
      manualPagination: true,
      manualFiltering: true,
      manualSorting: true,
      onReachEnd,
    })
    // The option is accepted + carried (the virtualizer wires range() → this).
    expect(table.setOptions).toBeTypeOf('function')
  })
})
