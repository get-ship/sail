// sail-table engine — the createTable() public contract (Backbone #1).
//
// `TableOptions<T>` (the input) + `Table<T>` (the facade the assembler returns).
// Split out of `types.ts` so the shared shapes (ColumnDef union, Row/Cell/
// ColumnApi/Header, the state slices) and the public contract each stay under
// the file cap. `types.ts` re-exports both so existing imports are unaffected —
// one source of truth, no parallel definition.
//
// PURE TypeScript — type-only; imports the Signal/Derived handles from
// `@ship/sail-runtime` and the shared shapes from `./types`.
//
// Authority: build-spec §"API sketch" (TableOptions + Table facade), §"Features"
// (the option/method surface), §Slices.

import type { Derived, Signal } from '@ship/sail-runtime'
import type { NavApi } from './nav'
import type {
  CellValue,
  ColumnApi,
  ColumnDef,
  ColumnFilter,
  ColumnPinningState,
  ColumnSizingState,
  ExpandedState,
  HeaderGroup,
  PaginationState,
  ReactiveTableState,
  Row,
  RowSelectionState,
  SortState,
  TableState,
  VisibilityState,
} from './types'

export interface TableOptions<T> {
  /** Raw array OR a live Signal/Derived identity (growing it adds rows). */
  data: T[] | Signal<T[]> | Derived<T[]>
  columns: ColumnDef<T>[]
  /** REQUIRED — drives the row id, `{#for}` key, selection, expansion. */
  getRowId: (row: T, index: number) => string
  /** Tree source — a row's child rows (Feature #13 expansion / sub-rows). */
  getSubRows?: (row: T) => T[] | undefined
  /** UNCONTROLLED seed; the engine owns each slice thereafter. */
  initialState?: Partial<TableState>
  /** CONTROLLED slices: a slice present here is engine-READ-ONLY — the engine
   *  reads the Signal and fires the matching `on*Change` instead of writing it
   *  (manual / server mode). A slice absent here is internal (engine-owned). */
  state?: Partial<ReactiveTableState>
  /** Seed the grouping slice (column ids) directly (shorthand for initialState). */
  grouping?: string[]
  /** Seed the expansion slice directly (`true` = expand-all). */
  expanded?: ExpandedState
  enableSorting?: boolean
  enableMultiSort?: boolean
  enableFilters?: boolean
  /** Per-row selectable predicate (or a flat on/off). */
  enableRowSelection?: boolean | ((row: Row<T>) => boolean)
  enableGrouping?: boolean
  enableExpanding?: boolean
  /** Tree: surface a matching leaf's ancestor chain (default true for trees). */
  filterFromLeafRows?: boolean
  /** Keyboard contract: row-level list nav vs 2D cell grid (default 'row'). */
  navMode?: 'row' | 'cell'
  // manual flags — manual stages form a PREFIX of the pipeline.
  manualSorting?: boolean
  manualFiltering?: boolean
  manualGrouping?: boolean
  manualPagination?: boolean
  /** Server-computed facets injected per column id (manual-faceting seam). */
  facets?: Partial<
    Record<string, { uniqueValues?: Map<CellValue, number>; minMax?: [number, number] }>
  >
  /** Server totals (manual pagination). May be a live signal. */
  rowCount?: number | Signal<number>
  pageCount?: number | Signal<number>
  /** Consumer-set loading flag the engine REFLECTS via `table.isLoading`
   *  (state ownership stays centralized). May be a live signal. */
  isLoading?: boolean | Signal<boolean>
  /** Consumer-set error slice re-exposed via `table.error`. May be a signal. */
  error?: unknown | Signal<unknown>
  onStateChange?: (s: TableState) => void
  onSortingChange?: (s: SortState[]) => void
  onColumnFiltersChange?: (f: ColumnFilter[]) => void
  onGlobalFilterChange?: (q: string) => void
  onPaginationChange?: (p: PaginationState) => void
  onRowSelectionChange?: (sel: RowSelectionState) => void
  onGroupingChange?: (g: string[]) => void
  onExpandedChange?: (e: ExpandedState) => void
  /** Committed cell edit (the sail-cell-edit event payload). */
  onCellEdit?: (e: { rowId: string; columnId: string; value: CellValue }) => void
  /** Infinite-scroll seam — the virtualizer range() drives this (manual mode). */
  onReachEnd?: (range: { startIndex: number; endIndex: number }) => void
  // column-lane callbacks (orthogonal — fired by the column mutators)
  onColumnSizingChange?: (s: ColumnSizingState) => void
  onColumnOrderChange?: (o: string[]) => void
  onColumnVisibilityChange?: (v: VisibilityState) => void
  onColumnPinningChange?: (p: ColumnPinningState) => void
}

export interface Table<T> {
  rows: Derived<Row<T>[]>
  /** THE virtualizer count source — the center band (paginated rows minus the
   *  pinned top/bottom bands). For an unpinned table `centerRows === rows`. */
  centerRows: Derived<Row<T>[]>
  /** Pinned bands rendered OUTSIDE the virtual window (sticky sibling bands). */
  topRows: Derived<Row<T>[]>
  bottomRows: Derived<Row<T>[]>
  /** No core rows at all — the initial "no items yet" empty branch. */
  isEmptyInitial: Derived<boolean>
  /** Filters active + zero filtered rows but core rows exist — the "no results,
   *  clear filters" empty branch. */
  isEmptyFiltered: Derived<boolean>
  /** Consumer-fed loading flag, reflected centrally. */
  isLoading: Derived<boolean>
  /** Consumer-fed error slice, reflected centrally. */
  error: Derived<unknown>
  preColumnFilteredRows: Derived<Row<T>[]>
  filteredRows: Derived<Row<T>[]>
  sortedRows: Derived<Row<T>[]>
  groupedRows: Derived<Row<T>[]>
  prePaginationRows: Derived<Row<T>[]>
  rowCount: Derived<number>
  /** Total pages — ceil(rowCount / pageSize), or the injected manual pageCount. */
  pageCount: Derived<number>
  /** Per-column footer model (the real tfoot data behind getFooterAggregate). */
  footerGroups: Derived<HeaderGroup<T>[]>
  state: ReactiveTableState
  selectedRowIds: Derived<string[]>
  getIsAllRowsSelected: Derived<boolean>
  getIsAllPageRowsSelected: Derived<boolean>
  getIsSomePageRowsSelected: Derived<boolean>
  hasActiveFilters: Derived<boolean>
  // ── ORTHOGONAL column lane (presentational — never re-derives rows) ─────────
  visibleLeafColumns: Derived<ColumnApi<T>[]>
  leftLeafColumns: Derived<ColumnApi<T>[]>
  rightLeafColumns: Derived<ColumnApi<T>[]>
  /** Sum of resolved px widths of visible columns — body table-width source. */
  getTotalSize: Derived<number>
  // sorting
  toggleSort: (id: string, opts?: { multi?: boolean }) => void
  setSorting: (next: SortState[] | ((prev: SortState[]) => SortState[])) => void
  resetSorting: () => void
  // grouping + expansion
  setGrouping: (g: string[] | ((prev: string[]) => string[])) => void
  toggleGrouping: (id: string) => void
  setExpanded: (u: ExpandedState | ((prev: ExpandedState) => ExpandedState)) => void
  toggleExpanded: (id: string, v?: boolean) => void
  toggleAllExpanded: (v?: boolean) => void
  // filtering
  setColumnFilter: (id: string, value: unknown) => void
  setColumnFilters: (f: ColumnFilter[]) => void
  setGlobalFilter: (q: string) => void
  resetFilters: () => void
  // pagination
  setPageIndex: (i: number) => void
  nextPage: () => void
  previousPage: () => void
  setPageSize: (n: number) => void
  getCanNextPage: () => boolean
  getCanPreviousPage: () => boolean
  getPageCount: () => number
  // selection
  setRowSelection: (
    u: RowSelectionState | ((prev: RowSelectionState) => RowSelectionState),
  ) => void
  toggleRowSelected: (
    id: string,
    v?: boolean,
    opts?: { range?: boolean; additive?: boolean },
  ) => void
  toggleAllPageRowsSelected: (v?: boolean) => void
  toggleAllRowsSelected: (v?: boolean) => void
  resetRowSelection: () => void
  // column lane (orthogonal mutators)
  setColumnSizing: (
    u: ColumnSizingState | ((prev: ColumnSizingState) => ColumnSizingState),
    opts?: { mode?: 'onChange' | 'onEnd' },
  ) => void
  setColumnOrder: (o: string[]) => void
  setColumnVisibility: (
    u: VisibilityState | ((prev: VisibilityState) => VisibilityState),
  ) => void
  setColumnPinning: (
    u: ColumnPinningState | ((prev: ColumnPinningState) => ColumnPinningState),
  ) => void
  pinColumn: (id: string, pos: 'left' | 'right' | false) => void
  /** The single `--col-<id>-size` custom property name (header + body source). */
  getColumnSizeVar: (id: string) => string
  /** Its resolved value, e.g. `'200px'` — the body reads this; the header reads
   *  `getColumn(id).getSize()`; ONE resolved sizing map backs both. */
  getColumnSizeVarValue: (id: string) => string
  // editing model (Feature #15 — backs the sail-cell-edit event)
  startEditing: (rowId: string, columnId: string) => void
  commitEdit: (value: CellValue) => void
  cancelEdit: () => void
  // keyboard-nav (Feature #14 — engine owns the focus MODEL; view owns DOM focus)
  nav: NavApi
  /** Live re-config seam (data/columns/manual flags/callbacks/navMode). */
  setOptions: (patch: Partial<TableOptions<T>>) => void
  // lookups
  getColumn: (id: string) => ColumnApi<T> | undefined
  getRow: (id: string) => Row<T> | undefined
}
