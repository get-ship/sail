// sail-table engine — Cell<T> factory (Backbone #1, Slice 4).
//
// Wraps a (row, column) pair into the `Cell<T>` API shape. A cell is the leaf
// of the render contract: it carries the column's `ColumnApi`, the cell value
// (via `row.getValue(column.id)`), the `CellContext` render hint, and the
// discriminators a view branches on — `getIsEditing()` (the editing slice
// reflected here), `getIsGrouped()`/`getIsAggregated()` (group/aggregate cells),
// `getIsPlaceholder()` (a group column's non-grouped cell renders nothing).
//
// Cells are built lazily per `row.getVisibleCells()` through the late-bound
// `bindings.cells` handle so a row never holds DOM and the cell tracks the live
// editing slice when its `getIsEditing()` is read in a reactive read scope.
//
// PURE TypeScript — no runtime imports (the editing read tracks via the
// `isEditing` query the editing slice backs in its own module).
//
// Authority: build-spec §"Authoritative Row / Cell / Column / Header API"
// (the Cell<T> member table), §Features #15 (getIsEditing backs sail-cell-edit),
// Slice 4.

import type { Cell, CellContext, CellValue, ColumnApi, Row } from './types'

/** Late-bound queries the cells route through (the editing slice + the table
 *  handle the `CellContext` carries). */
export interface CellDeps<T> {
  /** Is (rowId, columnId) the currently-editing target? Tracks the slice. */
  isEditing: (rowId: string, columnId: string) => boolean
  /** The owning table — exposed on `CellContext.table` (opaque to the engine). */
  table: () => unknown
}

/**
 * Build one `Cell<T>` for a (row, column) pair. `getValue` reads through the
 * row's accessor map; `getIsGrouped`/`getIsAggregated` reflect a group row's
 * grouping column vs an aggregating column; `getIsPlaceholder` is true for a
 * group row's cell on a column that is neither the grouping column nor an
 * aggregate (it renders nothing).
 */
export function makeCell<T>(
  row: Row<T>,
  column: ColumnApi<T>,
  deps: CellDeps<T>,
): Cell<T> {
  const id = `${row.id}:${column.id}`
  const getValue = (): CellValue => row.getValue(column.id)

  const isGroupedCell = (): boolean =>
    row.getIsGrouped() && column.getIsGrouped()
  const isAggregatedCell = (): boolean => {
    if (!row.getIsGrouped()) return false
    if (column.getIsGrouped()) return false
    const def = column.columnDef as { aggregationFn?: unknown }
    return def.aggregationFn != null
  }
  const isPlaceholder = (): boolean =>
    row.getIsGrouped() && !isGroupedCell() && !isAggregatedCell()

  const cell: Cell<T> = {
    id,
    row,
    column,
    getValue,
    getContext: (): CellContext<T> => ({
      row,
      column,
      cell,
      getValue,
      table: deps.table(),
    }),
    getIsAggregated: isAggregatedCell,
    getIsGrouped: isGroupedCell,
    getIsPlaceholder: isPlaceholder,
    getIsEditing: () => deps.isEditing(row.id, column.id),
  }
  return cell
}

/** The late-bound cell handle a row's `getVisibleCells()` routes through. */
export interface CellBinding<T> {
  getVisibleCells: (row: Row<T>) => Cell<T>[]
}

/**
 * Build the cell binding: `getVisibleCells(row)` maps the current
 * `visibleLeafColumns` to a `Cell<T>` per column. Reads the lane lazily so a
 * column hide/show reflects in the next `getVisibleCells()` call (the lane read
 * tracks when invoked inside a template/reactive read scope).
 */
export function buildCellBinding<T>(
  visibleLeafColumns: () => ColumnApi<T>[],
  deps: CellDeps<T>,
): CellBinding<T> {
  return {
    getVisibleCells: (row: Row<T>): Cell<T>[] =>
      visibleLeafColumns().map((column) => makeCell(row, column, deps)),
  }
}
