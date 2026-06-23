// sui-table engine — shared types (Backbone #1, Slice 1a).
//
// The ColumnDef discriminated union (accessor | display | group) plus the
// Row/Cell/Header API shapes and the reactive TableState slices. PURE
// TypeScript — no runtime imports beyond the Signal/Derived type handles
// from `@ship/sui-runtime` (type-only; no value imports here). The engine
// emits no DOM and makes no SDK calls.
//
// Authority: build-spec §"API sketch" (the discriminated union + the
// authoritative Row/Cell/ColumnApi/Header member tables) and §"Features".

import type { Derived, Signal } from '@ship/sui-runtime'

// ── Cell values ─────────────────────────────────────────────────────────────
export type CellValue = string | number | boolean | null | undefined

// ── Comparator / filter / aggregation fn handles ─────────────────────────────
// (concrete impls live in comparators.ts / filtering.ts / faceting.ts)
export type SortFn<T> = (a: Row<T>, b: Row<T>, columnId: string) => number
export type FilterFn<T> = (row: Row<T>, columnId: string, value: unknown) => boolean
export type AggFnName =
  | 'count'
  | 'sum'
  | 'min'
  | 'max'
  | 'mean'
  | 'extent'
  | 'median'
  | 'unique'
  | 'uniqueCount'
  | 'mode'
  | 'custom'
export type AggFn = (columnId: string, leafRows: unknown[]) => CellValue

// ── Column definition — DISCRIMINATED UNION ──────────────────────────────────
export interface ColumnBase<T> {
  /** Stable id — drives ordering/visibility/pin/sizing maps. */
  id: string
  header?: string | ((c: unknown) => string)
  footer?: string | ((c: unknown) => string)
  enableHiding?: boolean
  enableResizing?: boolean
  enablePinning?: boolean
  size?: number | 'auto'
  minSize?: number
  maxSize?: number
  /** fr-like grow weight for remaining-width distribution. */
  grow?: number
  /** align/format/tone — the view reads column.meta; the engine never does. */
  meta?: Record<string, unknown>
}

/** The only valued/sortable/filterable/groupable kind. */
export interface AccessorColumn<T> extends ColumnBase<T> {
  accessorKey: keyof T & string
  accessorFn?: never
  columns?: never
  cell?: (ctx: CellContext<T>) => unknown
  aggregatedCell?: (ctx: CellContext<T>) => unknown
  enableSorting?: boolean
  sortFn?: SortFn<T> | string
  sortDescFirst?: boolean
  enableColumnFilter?: boolean
  filterFn?: FilterFn<T> | string
  /** Opt OUT of the global filter for this column. */
  enableGlobalFilter?: boolean
  enableGrouping?: boolean
  getGroupingValue?: (row: T) => unknown
  aggregationFn?: AggFnName | AggFn
  enableEditing?: boolean
}

/** Header/cell only — checkbox gutter, row actions. May carry an accessorFn. */
export interface DisplayColumn<T> extends ColumnBase<T> {
  accessorKey?: never
  accessorFn?: (row: T) => CellValue
  columns?: never
  cell?: (ctx: CellContext<T>) => unknown
}

/** Multi-row header tree — header-only parent. */
export interface GroupColumn<T> extends ColumnBase<T> {
  columns: ColumnDef<T>[]
  accessorKey?: never
  accessorFn?: never
}

export type ColumnDef<T> = AccessorColumn<T> | DisplayColumn<T> | GroupColumn<T>

// ── Runtime validation of a column def ───────────────────────────────────────
export interface ColumnValidation {
  ok: boolean
  errors: string[]
}

/**
 * Validate that a column def is well-formed: it carries a stable `id` and at
 * most ONE of `accessorKey` / `accessorFn` / `columns` (the discriminant). A
 * column with two of those is ambiguous (e.g. an accessor that is also a group
 * parent) and is rejected. A column with NONE is treated as a display column
 * and is allowed (the floor `{ id }` is a valid display cell).
 */
export function validateColumnDef<T>(def: ColumnDef<T>): ColumnValidation {
  const errors: string[] = []
  const d = def as unknown as Record<string, unknown>

  if (typeof d.id !== 'string' || d.id.length === 0) {
    errors.push('column def requires a non-empty string `id`')
  }

  const present: string[] = []
  if ('accessorKey' in d && d.accessorKey != null) present.push('accessorKey')
  if ('accessorFn' in d && d.accessorFn != null) present.push('accessorFn')
  if ('columns' in d && d.columns != null) present.push('columns')

  if (present.length > 1) {
    errors.push(
      `column "${String(d.id)}" carries more than one of accessorKey/accessorFn/columns: ${present.join(', ')} — a column is exactly one kind (accessor | display | group)`,
    )
  }

  return { ok: errors.length === 0, errors }
}

// ── Sort / filter / pagination / selection state slices ──────────────────────
export type SortDirection = 'asc' | 'desc'
export interface SortState {
  id: string
  desc: boolean
}
export interface ColumnFilter {
  id: string
  value: unknown
}
export interface PaginationState {
  pageIndex: number
  pageSize: number
}
/** Select-all is expressible as include/exclude so server-mode "all minus 3"
 *  needs no id enumeration (build-spec Feature #7). */
export interface RowSelectionState {
  mode: 'include' | 'exclude'
  ids: Set<string>
}
export type ExpandedState = Record<string, boolean> | true
export type VisibilityState = Record<string, boolean>
export type ColumnSizingState = Record<string, number>
export interface ColumnSizingInfoState {
  columnId: string | null
  startSize: number
  deltaOffset: number
  isResizing: boolean
}
export interface ColumnPinningState {
  left: string[]
  right: string[]
}
export interface RowPinningState {
  top: string[]
  bottom: string[]
}
export interface EditingState {
  rowId: string
  columnId: string
}

/** Plain snapshot of all state slices (the `onStateChange` payload / seed). */
export interface TableState {
  sorting: SortState[]
  columnFilters: ColumnFilter[]
  globalFilter: string
  grouping: string[]
  expanded: ExpandedState
  pagination: PaginationState
  rowSelection: RowSelectionState
  selectionAnchor: string | null
  columnSizing: ColumnSizingState
  columnSizingInfo: ColumnSizingInfoState
  columnOrder: string[]
  columnVisibility: VisibilityState
  columnPinning: ColumnPinningState
  rowPinning: RowPinningState
  editing: EditingState | null
}

/** Reactive form — each slice is a Signal (read in a derived, written in batch). */
export type ReactiveTableState = {
  [K in keyof TableState]: Signal<TableState[K]>
}

// ── Row / Cell / Header / Column API shapes (the 8-pack contract) ────────────
export interface CellContext<T> {
  row: Row<T>
  column: ColumnApi<T>
  cell: Cell<T>
  getValue: () => CellValue
  table: unknown
}

export interface Row<T> {
  id: string
  index: number
  depth: number
  original: T
  getValue: (colId: string) => CellValue
  getVisibleCells: () => Cell<T>[]
  subRows: Row<T>[]
  getCanExpand: () => boolean
  getIsExpanded: () => boolean
  toggleExpanded: (v?: boolean) => void
  getParentRow: () => Row<T> | undefined
  getParentRows: () => Row<T>[]
  /** 1-based index of this row among its siblings (its parent's subRows, or the
   *  root row list at depth 0) — drives `aria-posinset` for a tree/treegrid. */
  getPosInSet: () => number
  /** Count of this row's siblings (incl. itself) — drives `aria-setsize`. */
  getSetSize: () => number
  getLeafRows: () => Row<T>[]
  getIsSelected: () => boolean
  getIsSomeSelected: () => boolean
  getCanSelect: () => boolean
  toggleSelected: (v?: boolean, opts?: { range?: boolean }) => void
  getIsPinned: () => 'top' | 'bottom' | false
  pin: (pos: 'top' | 'bottom' | false) => void
  getIsGrouped: () => boolean
  getGroupingValue: (colId: string) => unknown
}

export interface Cell<T> {
  id: string
  row: Row<T>
  column: ColumnApi<T>
  getValue: () => CellValue
  getContext: () => CellContext<T>
  getIsAggregated: () => boolean
  getIsGrouped: () => boolean
  getIsPlaceholder: () => boolean
  getIsEditing: () => boolean
}

export interface ColumnApi<T> {
  id: string
  columnDef: ColumnDef<T>
  depth: number
  getCanSort: () => boolean
  getIsSorted: () => SortDirection | false
  getSortIndex: () => number
  getNextSortingOrder: () => SortDirection | false
  getToggleSortingHandler: () => (e?: unknown) => void
  getCanFilter: () => boolean
  getFilterValue: () => unknown
  setFilterValue: (v: unknown) => void
  getFacetedUniqueValues: () => Map<CellValue, number>
  getFacetedMinMaxValues: () => [number, number] | undefined
  getFacetedRowModel: () => Row<T>[]
  getCanGroup: () => boolean
  getIsGrouped: () => boolean
  getCanHide: () => boolean
  getIsVisible: () => boolean
  toggleVisibility: (v?: boolean) => void
  getCanPin: () => boolean
  getIsPinned: () => 'left' | 'right' | false
  getPinnedIndex: () => number
  getIsLastColumn: (pos: 'left' | 'right' | false) => boolean
  getStart: (pos: 'left' | 'right') => number
  getSize: () => number
  getCanResize: () => boolean
  getResizeHandler: () => (e?: unknown) => void
  getFooterAggregate: () => CellValue
}

export interface Header<T> {
  id: string
  column: ColumnApi<T>
  colSpan: number
  rowSpan: number
  depth: number
  isPlaceholder: boolean
  getContext: () => unknown
  subHeaders: Header<T>[]
  getLeafHeaders: () => Header<T>[]
}

export interface HeaderGroup<T> {
  id: string
  depth: number
  headers: Header<T>[]
}

// ── Derived-typed convenience aliases (used by later slices' facades) ─────────
export type RowModel<T> = Derived<Row<T>[]>

// ── createTable() public contract — TableOptions + Table facade ──────────────
// Split into `contract.ts` (the input + the facade) to keep both this shapes
// module and the contract under the file cap; re-exported here so consumers may
// import the contract from the shared `types` barrel (one source of truth).
export type { Table, TableOptions } from './contract'
