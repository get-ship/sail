// sail-table engine — assembler helpers (Backbone #1).
//
// Small pure helpers the `create-table.ts` thin assembler composes: the
// state-slice `snapshot` (the `onStateChange` payload), the `initialState` +
// `grouping`/`expanded` shorthand merge, the depth-first `findRow` over a
// (possibly tree) core-row set, and the `hasActiveFilters` derived. Factored out
// to keep the assembler under the file cap.
//
// PURE TypeScript — imports `derived` from `@ship/sail-runtime` ONLY.
//
// Authority: build-spec §"Backbone #1", §"API sketch" (initialState merge,
// onStateChange snapshot), §States (hasActiveFilters).

import { derived, isSignal, type Derived, type Signal } from '@ship/sail-runtime'
import { isActiveFilterValue } from './filtering'
import type { ReactiveTableState, Row, TableOptions, TableState } from './types'

/** Partition the displayed (paginated) rows into the pinned top / center /
 *  bottom bands per `state.rowPinning`. The center band is THE virtualizer
 *  count source; the top/bottom bands render OUTSIDE the window. For an
 *  unpinned table `center === all` and the pinned bands are empty. */
export interface RowBands<T> {
  topRows: Derived<Row<T>[]>
  centerRows: Derived<Row<T>[]>
  bottomRows: Derived<Row<T>[]>
}
export function buildRowBands<T>(
  paginatedRows: Derived<Row<T>[]>,
  rowPinning: Signal<{ top: string[]; bottom: string[] }>,
): RowBands<T> {
  const topRows = derived<Row<T>[]>(() => {
    const pin = rowPinning().top
    if (pin.length === 0) return []
    const rows = paginatedRows()
    return pin.map((id) => rows.find((r) => r.id === id)).filter((r): r is Row<T> => r != null)
  })
  const bottomRows = derived<Row<T>[]>(() => {
    const pin = rowPinning().bottom
    if (pin.length === 0) return []
    const rows = paginatedRows()
    return pin.map((id) => rows.find((r) => r.id === id)).filter((r): r is Row<T> => r != null)
  })
  const centerRows = derived<Row<T>[]>(() => {
    const { top, bottom } = rowPinning()
    if (top.length === 0 && bottom.length === 0) return paginatedRows()
    const pinned = new Set([...top, ...bottom])
    return paginatedRows().filter((r) => !pinned.has(r.id))
  })
  return { topRows, centerRows, bottomRows }
}

/** Lift a consumer flag (plain value or live signal) to a read-tracked derived
 *  — the engine REFLECTS loading/error; it never owns the source. */
export function reflectFlag<V>(src: V | Signal<V> | undefined, fallback: V): Derived<V> {
  return derived<V>(() => {
    if (src === undefined) return fallback
    return isSignal(src) ? (src as Signal<V>)() : (src as V)
  })
}

/** Empty discriminators (build-spec §States): initial = no core rows; filtered
 *  = filters active, zero filtered rows, but core rows exist. */
export function buildEmptyDiscriminators<T>(
  coreRows: Derived<Row<T>[]>,
  filteredRows: Derived<Row<T>[]>,
  hasActiveFilters: Derived<boolean>,
): { isEmptyInitial: Derived<boolean>; isEmptyFiltered: Derived<boolean> } {
  const isEmptyInitial = derived<boolean>(() => coreRows().length === 0)
  const isEmptyFiltered = derived<boolean>(
    () => hasActiveFilters() && filteredRows().length === 0 && coreRows().length > 0,
  )
  return { isEmptyInitial, isEmptyFiltered }
}

/** Depth-first lookup of a row by id over the (possibly tree) core rows. */
export function findRow<T>(rows: Row<T>[], id: string): Row<T> | undefined {
  for (const row of rows) {
    if (row.id === id) return row
    if (row.subRows.length > 0) {
      const found = findRow(row.subRows, id)
      if (found) return found
    }
  }
  return undefined
}

/** Merge the `grouping`/`expanded` shorthand options over `initialState`. */
export function seedState<T>(options: TableOptions<T>): Partial<TableState> {
  const seed: Partial<TableState> = { ...options.initialState }
  if (options.grouping !== undefined) seed.grouping = options.grouping
  if (options.expanded !== undefined) seed.expanded = options.expanded
  return seed
}

/** `hasActiveFilters` — true when the global query or any column filter narrows. */
export function buildHasActiveFilters(state: ReactiveTableState): Derived<boolean> {
  return derived<boolean>(() => {
    if (state.globalFilter() !== '') return true
    return state.columnFilters().some((f) => isActiveFilterValue(f.value))
  })
}

/** Plain snapshot of every reactive slice (the `onStateChange` payload). */
export function snapshot(state: ReactiveTableState): TableState {
  return {
    sorting: state.sorting(),
    columnFilters: state.columnFilters(),
    globalFilter: state.globalFilter(),
    grouping: state.grouping(),
    expanded: state.expanded(),
    pagination: state.pagination(),
    rowSelection: state.rowSelection(),
    selectionAnchor: state.selectionAnchor(),
    columnSizing: state.columnSizing(),
    columnSizingInfo: state.columnSizingInfo(),
    columnOrder: state.columnOrder(),
    columnVisibility: state.columnVisibility(),
    columnPinning: state.columnPinning(),
    rowPinning: state.rowPinning(),
    editing: state.editing(),
  }
}
