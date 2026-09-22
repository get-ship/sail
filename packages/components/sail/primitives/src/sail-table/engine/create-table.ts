// sail-table engine — the createTable() assembler (Backbone #1, Slices 1b + 2).
//
// THE THIN ASSEMBLER. It lifts `data` to a Signal, seeds the reactive state,
// wires the one-direction derived row pipeline (filter → sort → group → page)
// and the orthogonal selection lane, and returns the `Table<T>` facade. Heavy
// logic lives in the siblings it composes (`pipeline.ts`, `filtering.ts`,
// `selection.ts`, `pagination.ts`, `row-api.ts`, `column-api.ts`, `state.ts`,
// `comparators.ts`). Every `set*`/`toggle*` writes its Signal inside `batch()`
// then fires the matching `on*Change`; reads track in `derived`/template.
//
// PURE: imports `{ signal, batch, isSignal }` from `@ship/sail-runtime` ONLY —
// no SDK client, no platform calls, no network, no server-events, no data
// directive.
//
// Authority: build-spec §"Backbone #1 — createTable() engine", §"API sketch",
// §"getRowId is REQUIRED", §"Signal graph" (suffix-only recompute; filter →
// sort → group → paginate order), §Features #2/#3/#6/#7, Slices 1b + 2.

import { signal, batch, derived, isSignal, type Derived, type Signal } from '@ship/sail-runtime'
import { buildEditingAndCells, buildNavSurface } from './assemble-extras'
import { applyControlledState, warnOnNonPrefix } from './manual'
import { makeColumnApi, type ColumnApiDeps } from './column-api'
import { buildColumnMutators } from './column-mutators'
import { buildColumnLane } from './columns'
import { buildFaceting } from './faceting'
import {
  buildEmptyDiscriminators,
  buildHasActiveFilters,
  buildRowBands,
  findRow,
  reflectFlag,
  seedState,
  snapshot,
} from './facade'
import { isActiveFilterValue, resolveCellPredicate } from './filtering'
import { buildFooterGroups, footerAggregateFor, groupingValueOf } from './grouping'
import { buildPaginationStages } from './pagination'
import { buildRowChain } from './row-pipeline'
import { buildRowMutators } from './row-mutators'
import { buildSelection } from './selection'
import { buildState } from './state'
import type {
  CellValue,
  ColumnApi,
  ColumnDef,
  ColumnFilter,
  PaginationState,
  Row,
  RowSelectionState,
  SortState,
  Table,
  TableOptions,
  TableState,
} from './types'

// Re-export the public contract so consumers may import it from either the
// types home or the assembler that produces it.
export type { Table, TableOptions } from './types'

function liftData<T>(data: T[] | Signal<T[]> | Derived<T[]>): Signal<T[]> {
  return isSignal(data) ? (data as Signal<T[]>) : signal(data as T[])
}

export function createTable<T>(options: TableOptions<T>): Table<T> {
  const { columns } = options
  const sortingEnabled = options.enableSorting !== false

  const dataSignal = liftData(options.data)
  const state = buildState(seedState(options))
  // CONTROLLED slices: a slice in `options.state` REPLACES the engine's signal
  // (engine reads it; the mutators below SKIP writing a controlled slice and
  // fire its on*Change instead). Must run BEFORE the row chain reads the slices.
  const controlled = applyControlledState(state, options.state)
  const isControlled = (k: keyof TableState): boolean => controlled.has(k)
  // Pipeline-prefix validation: warn on a non-prefix manual combo (e.g. manual
  // pagination over client filter/sort).
  warnOnNonPrefix(options)
  const snap = (): TableState => snapshot(state)
  const bindings: import('./row-api').RowBindings<T> = {}

  // ── Row pipeline (filter → sort → group/flatten → paginate) ─────────────────
  const chain = buildRowChain<T>({ options, state, bindings, dataSignal })
  const { coreRows, preColumnFilteredRows, filteredRows, sortedRows, prePaginationRows } = chain
  const groupedRows = prePaginationRows

  const pageOpts: Parameters<typeof buildPaginationStages<T>>[0] = {
    prePaginationRows,
    pagination: state.pagination,
    manual: options.manualPagination === true,
  }
  if (options.rowCount !== undefined) pageOpts.rowCount = options.rowCount
  if (options.pageCount !== undefined) pageOpts.pageCount = options.pageCount
  const page = buildPaginationStages<T>(pageOpts)

  // ── Selection lane (orthogonal — scopes the DISPLAYED order, leaf rows) ──────
  const canSelectRow = (row: Row<T>): boolean => {
    if (row.getIsGrouped()) return false
    const e = options.enableRowSelection
    if (e === false) return false
    if (typeof e === 'function') return e(row)
    return true
  }
  const displayedLeaves = page.paginatedRows // leaf + group rows; selection filters groups
  const selection = buildSelection<T>({
    rowSelection: state.rowSelection,
    selectionAnchor: state.selectionAnchor,
    displayedRows: displayedLeaves,
    canSelect: canSelectRow,
    fireChange: (sel) => {
      options.onRowSelectionChange?.(sel)
      options.onStateChange?.(snapshot(state))
    },
  })
  bindings.selection = selection

  // ── Grouping + expansion mutators (+ the row-bound expansion handle) ─────────
  /** Walk the full core tree for expandable ids (expand-all demotion needs them). */
  const expandableIds = (): string[] => {
    const out: string[] = []
    const walk = (rows: Row<T>[]): void => {
      for (const r of rows) {
        if (r.subRows.length > 0) {
          out.push(r.id)
          walk(r.subRows)
        }
      }
    }
    walk(coreRows())
    return out
  }
  const rowMut = buildRowMutators(state, snap, {
    ...(options.onGroupingChange && { onGroupingChange: options.onGroupingChange }),
    ...(options.onExpandedChange && { onExpandedChange: options.onExpandedChange }),
    ...(options.onStateChange && { onStateChange: options.onStateChange }),
    expandableIds,
  })
  bindings.expansion = {
    isExpanded: rowMut.isExpanded,
    toggleExpanded: rowMut.toggleExpanded,
  }

  // ── Sorting mutators (controlled `sorting` is engine-read-only) ──────────────
  function setSorting(next: SortState[] | ((prev: SortState[]) => SortState[])): void {
    const resolved = typeof next === 'function' ? next(state.sorting()) : next
    // A controlled slice is NEVER written by the engine — fire the callback and
    // let the consumer's write flow back in (manual / server mode).
    if (!isControlled('sorting')) batch(() => state.sorting.set(resolved))
    options.onSortingChange?.(resolved)
    options.onStateChange?.(snapshot(state))
  }
  function toggleSort(id: string, opts?: { multi?: boolean }): void {
    if (!sortingEnabled) return
    const cur = state.sorting()
    const existing = cur.find((s) => s.id === id)
    let nextEntry: SortState | null
    if (!existing) nextEntry = { id, desc: false }
    else if (!existing.desc) nextEntry = { id, desc: true }
    else nextEntry = null
    let next: SortState[]
    if (opts?.multi === true) {
      const without = cur.filter((s) => s.id !== id)
      next = nextEntry ? [...without, nextEntry] : without
    } else {
      next = nextEntry ? [nextEntry] : []
    }
    setSorting(next)
  }
  const resetSorting = (): void => setSorting([])

  // ── Filter mutators (controlled slices are engine-read-only) ─────────────────
  function setColumnFilters(f: ColumnFilter[]): void {
    if (!isControlled('columnFilters')) batch(() => state.columnFilters.set(f))
    options.onColumnFiltersChange?.(f)
    options.onStateChange?.(snapshot(state))
  }
  function setColumnFilter(id: string, value: unknown): void {
    const without = state.columnFilters().filter((f) => f.id !== id)
    setColumnFilters(isActiveFilterValue(value) ? [...without, { id, value }] : without)
  }
  function setGlobalFilter(q: string): void {
    if (!isControlled('globalFilter')) batch(() => state.globalFilter.set(q))
    options.onGlobalFilterChange?.(q)
    options.onStateChange?.(snapshot(state))
  }
  function resetFilters(): void {
    batch(() => {
      if (!isControlled('columnFilters')) state.columnFilters.set([])
      if (!isControlled('globalFilter')) state.globalFilter.set('')
    })
    options.onColumnFiltersChange?.([])
    options.onGlobalFilterChange?.('')
    options.onStateChange?.(snapshot(state))
  }

  // ── Pagination mutators (controlled `pagination` is engine-read-only) ────────
  function setPagination(next: PaginationState): void {
    if (!isControlled('pagination')) batch(() => state.pagination.set(next))
    options.onPaginationChange?.(next)
    options.onStateChange?.(snapshot(state))
  }
  const setPageIndex = (i: number): void =>
    setPagination({ ...state.pagination(), pageIndex: i })
  const setPageSize = (n: number): void =>
    setPagination({ ...state.pagination(), pageSize: n })
  function nextPage(): void {
    if (page.getCanNextPage()) setPageIndex(state.pagination().pageIndex + 1)
  }
  function previousPage(): void {
    if (page.getCanPreviousPage()) setPageIndex(state.pagination().pageIndex - 1)
  }

  // ── ORTHOGONAL column lane (Slice 3a — presentational, off the row chain) ────
  const lane = buildColumnLane<T>(columns, {
    columnSizing: state.columnSizing,
    columnOrder: state.columnOrder,
    columnVisibility: state.columnVisibility,
    columnPinning: state.columnPinning,
  })
  const colMutators = buildColumnMutators<T>({
    columns,
    state,
    snapshot: snap,
    callbacks: {
      ...(options.onColumnSizingChange && {
        onColumnSizingChange: options.onColumnSizingChange,
      }),
      ...(options.onColumnOrderChange && { onColumnOrderChange: options.onColumnOrderChange }),
      ...(options.onColumnVisibilityChange && {
        onColumnVisibilityChange: options.onColumnVisibilityChange,
      }),
      ...(options.onColumnPinningChange && {
        onColumnPinningChange: options.onColumnPinningChange,
      }),
      ...(options.onStateChange && { onStateChange: options.onStateChange }),
    },
  })

  // ── Faceting (lazy + memoized per column, off preColumnFilteredRows) ─────────
  const byColId = new Map(columns.map((c) => [c.id, c]))
  const faceting = buildFaceting<T>({
    preColumnFilteredRows,
    columnFilters: state.columnFilters,
    columns,
    resolvePredicate: (def, value) => resolveCellPredicate(def, value),
    facetValueOf: (row, colId) => groupingValueOf(row, byColId.get(colId), colId) as CellValue,
    manualFiltering: options.manualFiltering === true,
    ...(options.facets && { injected: options.facets }),
  })

  // ── Footer grand-total over filteredRows (orthogonal to the row chain) ───────
  const footerAggregate = (id: string): CellValue =>
    footerAggregateFor(byColId.get(id), filteredRows())

  const columnApis = new Map<string, ColumnApi<T>>()
  for (const def of columns) {
    const deps: ColumnApiDeps<T> = {
      lane,
      toggleVisibility: colMutators.toggleVisibility,
      getFilterValue: (id) => state.columnFilters().find((f) => f.id === id)?.value,
      setFilterValue: (id, v) => setColumnFilter(id, v),
      grouping: state.grouping,
      toggleGrouping: rowMut.toggleGrouping,
      getFacetedUniqueValues: faceting.getFacetedUniqueValues,
      getFacetedMinMaxValues: faceting.getFacetedMinMaxValues,
      getFacetedRowModel: faceting.getFacetedRowModel,
      getFooterAggregate: footerAggregate,
    }
    columnApis.set(def.id, makeColumnApi(def, state.sorting, toggleSort, deps))
  }
  const footerGroups = buildFooterGroups<T>(columns, (id) => columnApis.get(id))
  // Facade leaf-column deriveds: map the lane's def arrays to the ColumnApi
  // identities so consumers read the full Column API (the 8-pack contract).
  const toApis = (defs: ColumnDef<T>[]): ColumnApi<T>[] =>
    defs.map((d) => columnApis.get(d.id)!)
  const visibleLeafColumns = derived<ColumnApi<T>[]>(() =>
    toApis(lane.visibleLeafColumns()),
  )
  const leftLeafColumns = derived<ColumnApi<T>[]>(() => toApis(lane.leftLeafColumns()))
  const rightLeafColumns = derived<ColumnApi<T>[]>(() => toApis(lane.rightLeafColumns()))

  // ── Editing slice + cell binding (Slice 4, Feature #15) ──────────────────────
  // A forward holder for the table identity exposed on CellContext.table.
  let tableRef: Table<T> | undefined
  const editingCells = buildEditingAndCells<T>({
    state,
    dataSignal,
    columns,
    getRowId: options.getRowId,
    visibleLeafColumns,
    tableRef: () => tableRef,
    ...(options.onCellEdit && { onCellEdit: options.onCellEdit }),
    ...(options.onStateChange && { onStateChange: () => options.onStateChange?.(snap()) }),
  })
  const { editing } = editingCells
  bindings.cells = editingCells.cellBinding // late-bound: rows' getVisibleCells

  // ── Keyboard-nav focus model (Slice 4, Feature #14) ──────────────────────────
  const nav = buildNavSurface<T>({
    navMode: options.navMode ?? 'row',
    rows: page.paginatedRows,
    visibleLeafColumns,
    editing,
    selection,
    pageSize: () => state.pagination().pageSize,
  })

  // ── Live re-config seam (Feature: setOptions hot-swap) ───────────────────────
  // Patches the safe, identity-preserving options (data / manual flags / callbacks
  // / navMode). Changing data swaps the lifted signal's value; the rest mutate the
  // captured `options` so subsequent mutator reads see the patch.
  function setOptions(patch: Partial<TableOptions<T>>): void {
    if (patch.data !== undefined && !isSignal(patch.data)) {
      batch(() => dataSignal.set(patch.data as T[]))
    }
    Object.assign(options, patch)
  }

  // ── Render bands (row pinning) + empty/loading discriminators ───────────────
  // The center band is THE virtualizer count source; top/bottom render outside
  // the window. For an unpinned table center === paginatedRows.
  const bands = buildRowBands<T>(page.paginatedRows, state.rowPinning)
  const hasActiveFilters = buildHasActiveFilters(state)
  const { isEmptyInitial, isEmptyFiltered } = buildEmptyDiscriminators<T>(
    coreRows,
    filteredRows,
    hasActiveFilters,
  )
  const isLoading = reflectFlag<boolean>(options.isLoading, false)
  const error = reflectFlag<unknown>(options.error, undefined)

  tableRef = {
    rows: page.paginatedRows,
    centerRows: bands.centerRows,
    topRows: bands.topRows,
    bottomRows: bands.bottomRows,
    isEmptyInitial,
    isEmptyFiltered,
    isLoading,
    error,
    preColumnFilteredRows,
    filteredRows,
    sortedRows,
    groupedRows,
    prePaginationRows,
    rowCount: page.rowCount,
    pageCount: page.pageCount,
    footerGroups,
    state,
    selectedRowIds: selection.selectedRowIds,
    getIsAllRowsSelected: selection.getIsAllRowsSelected,
    getIsAllPageRowsSelected: selection.getIsAllPageRowsSelected,
    getIsSomePageRowsSelected: selection.getIsSomePageRowsSelected,
    hasActiveFilters,
    toggleSort,
    setSorting,
    resetSorting,
    setGrouping: rowMut.setGrouping,
    toggleGrouping: rowMut.toggleGrouping,
    setExpanded: rowMut.setExpanded,
    toggleExpanded: rowMut.toggleExpanded,
    toggleAllExpanded: rowMut.toggleAllExpanded,
    setColumnFilter,
    setColumnFilters,
    setGlobalFilter,
    resetFilters,
    setPageIndex,
    nextPage,
    previousPage,
    setPageSize,
    getCanNextPage: page.getCanNextPage,
    getCanPreviousPage: page.getCanPreviousPage,
    getPageCount: page.getPageCount,
    setRowSelection: selection.setRowSelection,
    toggleRowSelected: selection.toggleRowSelected,
    toggleAllPageRowsSelected: selection.toggleAllPageRowsSelected,
    toggleAllRowsSelected: selection.toggleAllRowsSelected,
    resetRowSelection: selection.resetRowSelection,
    // column lane (orthogonal — presentational, never re-derives rows)
    visibleLeafColumns,
    leftLeafColumns,
    rightLeafColumns,
    getTotalSize: lane.getTotalSize,
    setColumnSizing: colMutators.setColumnSizing,
    setColumnOrder: colMutators.setColumnOrder,
    setColumnVisibility: colMutators.setColumnVisibility,
    setColumnPinning: colMutators.setColumnPinning,
    pinColumn: colMutators.pinColumn,
    getColumnSizeVar: lane.getColumnSizeVar,
    getColumnSizeVarValue: lane.getColumnSizeVarValue,
    // editing model (Feature #15) + keyboard nav (Feature #14)
    startEditing: editing.startEditing,
    commitEdit: editing.commitEdit,
    cancelEdit: editing.cancelEdit,
    nav,
    setOptions,
    getColumn: (id) => columnApis.get(id),
    getRow: (id) => findRow(coreRows(), id),
  }
  return tableRef
}
