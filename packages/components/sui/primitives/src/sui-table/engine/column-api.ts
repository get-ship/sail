// sui-table engine — ColumnApi<T> factory (Backbone #1, Slices 1b + 3a).
//
// Wraps a flat leaf `ColumnDef` into the `ColumnApi<T>` API shape. Slice 1b
// landed the sorting-priority surface (`id`, `columnDef`, `getSortIndex()` /
// `getIsSorted()` / `getCanSort()` reading the live sorting slice). Slice 3a
// wires the ORTHOGONAL column-lane reads — `getSize`/`getStart`/`getIsVisible`/
// `getIsPinned`/`getPinnedIndex`/`getIsLastColumn`/`getCanResize`/`getCanPin`/
// `getCanHide`/`toggleVisibility` — through the `ColumnLane` handle so they
// track the column slices (sizing/order/visibility/pinning) and reflow
// reactively. The remaining members stay inert "feature off" defaults until
// their slice ships.
//
// PURE TypeScript — imports Signal types only; the reads track when invoked
// inside a `derived`/template.
//
// Authority: build-spec §"Authoritative Row / Cell / Column / Header API",
// §Features #2 (`getSortIndex()`), #8-#11 (sizing/order/visibility/pinning
// reads), Slices 1b + 3a.

import type { Signal } from '@ship/sail-runtime'
import type { ColumnLane } from './columns'
import type { CellValue, ColumnApi, ColumnDef, Row, SortDirection, SortState } from './types'

/** The column-lane reads + mutators a `ColumnApi` routes through (Slice 3a/3b). */
export interface ColumnApiDeps<T> {
  lane: ColumnLane<T>
  toggleVisibility: (id: string, v?: boolean) => void
  /** Filtering reads (Slice 2 mutators surfaced on the column). */
  getFilterValue?: (id: string) => unknown
  setFilterValue?: (id: string, v: unknown) => void
  /** Grouping reads (Slice 3b). */
  grouping?: Signal<string[]>
  toggleGrouping?: (id: string) => void
  /** Faceting accessors (Slice 3b) — lazy + memoized per column. */
  getFacetedUniqueValues?: (id: string) => Map<CellValue, number>
  getFacetedMinMaxValues?: (id: string) => [number, number] | undefined
  getFacetedRowModel?: (id: string) => Row<T>[]
  /** Grand-total footer aggregate over filteredRows (Slice 3b). */
  getFooterAggregate?: (id: string) => CellValue
}

/** Is this column an accessor column (the only sortable/valued kind)? */
function isAccessor<T>(def: ColumnDef<T>): boolean {
  return 'accessorKey' in def && (def as { accessorKey?: unknown }).accessorKey != null
}

/**
 * Build the `ColumnApi<T>` for one leaf column. `sorting` is the reactive
 * sorting slice; `getSortIndex`/`getIsSorted` read it so a column's reported
 * priority tracks the live multi-sort order.
 */
export function makeColumnApi<T>(
  def: ColumnDef<T>,
  sorting: Signal<SortState[]>,
  toggleSort: (id: string, opts?: { multi?: boolean }) => void,
  deps: ColumnApiDeps<T>,
): ColumnApi<T> {
  const id = def.id
  const { lane, toggleVisibility } = deps
  const accessor = isAccessor(def)
  const sortable = accessor && (def as { enableSorting?: boolean }).enableSorting !== false
  // Hiding/resizing/pinning default ON unless the def opts out.
  const canHide = def.enableHiding !== false
  const canResize = def.enableResizing !== false
  const canPin = def.enablePinning !== false
  // Filtering/grouping default ON for accessor columns unless opted out.
  const canFilter = accessor && (def as { enableColumnFilter?: boolean }).enableColumnFilter !== false
  const canGroup = accessor && (def as { enableGrouping?: boolean }).enableGrouping !== false

  const getSortIndex = (): number => sorting().findIndex((s) => s.id === id)

  const getIsSorted = (): SortDirection | false => {
    const entry = sorting().find((s) => s.id === id)
    if (!entry) return false
    return entry.desc ? 'desc' : 'asc'
  }

  const getCanSort = (): boolean => sortable

  return {
    id,
    columnDef: def,
    depth: 0,
    getCanSort,
    getIsSorted,
    getSortIndex,
    getNextSortingOrder: () => {
      const cur = getIsSorted()
      if (cur === false) return 'asc'
      if (cur === 'asc') return 'desc'
      return false
    },
    getToggleSortingHandler: () => () => toggleSort(id),
    getCanFilter: () => canFilter,
    getFilterValue: () => deps.getFilterValue?.(id),
    setFilterValue: (v: unknown) => deps.setFilterValue?.(id, v),
    getFacetedUniqueValues: () => deps.getFacetedUniqueValues?.(id) ?? new Map(),
    getFacetedMinMaxValues: () => deps.getFacetedMinMaxValues?.(id),
    getFacetedRowModel: () => deps.getFacetedRowModel?.(id) ?? [],
    getCanGroup: () => canGroup,
    getIsGrouped: () => deps.grouping?.().includes(id) ?? false,
    getCanHide: () => canHide,
    getIsVisible: () => lane.getIsVisible(id),
    toggleVisibility: (v?: boolean) => toggleVisibility(id, v),
    getCanPin: () => canPin,
    getIsPinned: () => lane.getIsPinned(id),
    getPinnedIndex: () => lane.getPinnedIndex(id),
    getIsLastColumn: (pos) => lane.getIsLastColumn(id, pos),
    getStart: (pos) => lane.getStart(id, pos),
    getSize: () => lane.getSize(id),
    getCanResize: () => canResize,
    getResizeHandler: () => () => {},
    getFooterAggregate: () => deps.getFooterAggregate?.(id),
  }
}
