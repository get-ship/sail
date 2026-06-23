// sui-table engine — Slice 4 assembly helpers (editing · cells · nav).
//
// Factored out of the `create-table.ts` thin assembler so it stays under the
// file cap. Builds the three Slice-4 pieces over the already-assembled state,
// data signal, and column/row deriveds:
//
//   • editing  — the editing slice mutators (startEditing/commit/cancel + gate)
//   • cells    — the late-bound cell binding the rows' getVisibleCells routes to
//                (the Cell.getIsEditing reflects the editing slice)
//   • nav      — the keyboard focus model (navMode, onKeydown, typeahead, …)
//
// PURE TypeScript — imports nothing from `@ship/sui-runtime` itself (the pieces
// it composes already carry their reactivity); no DOM, no SDK.
//
// Authority: build-spec §Features #14 (keyboard-nav) + #15 (cell editing),
// §"Cell<T>" API (getIsEditing), Slice 4.

import { buildCellBinding, type CellBinding } from './cell-api'
import { buildEditing, type EditingApi } from './editing'
import { buildNav, type NavApi } from './nav'
import type { SelectionApi } from './selection'
import type {
  CellValue,
  ColumnApi,
  ColumnDef,
  EditingState,
  ReactiveTableState,
  Row,
} from './types'
import type { Derived, Signal } from '@ship/sui-runtime'

export interface EditingCellsInputs<T> {
  state: ReactiveTableState
  dataSignal: Signal<T[]>
  columns: ColumnDef<T>[]
  getRowId: (row: T, index: number) => string
  visibleLeafColumns: Derived<ColumnApi<T>[]>
  /** The owning table (opaque) — exposed on CellContext.table. */
  tableRef: () => unknown
  onCellEdit?: (e: { rowId: string; columnId: string; value: CellValue }) => void
  onStateChange?: () => void
}

export interface EditingCells<T> {
  editing: EditingApi
  cellBinding: CellBinding<T>
}

/** Build the editing slice mutators + the cell binding (the cell's
 *  getIsEditing reflects the editing slice through the editing api). */
export function buildEditingAndCells<T>(inputs: EditingCellsInputs<T>): EditingCells<T> {
  const editing = buildEditing<T>({
    editing: inputs.state.editing as Signal<EditingState | null>,
    data: inputs.dataSignal,
    columns: inputs.columns,
    getRowId: inputs.getRowId,
    ...(inputs.onCellEdit && { onCellEdit: inputs.onCellEdit }),
    ...(inputs.onStateChange && { onStateChange: inputs.onStateChange }),
  })
  const cellBinding = buildCellBinding<T>(inputs.visibleLeafColumns, {
    isEditing: editing.isEditing,
    table: inputs.tableRef,
  })
  return { editing, cellBinding }
}

export interface NavInputsExtra<T> {
  navMode: 'row' | 'cell'
  rows: Derived<Row<T>[]>
  visibleLeafColumns: Derived<ColumnApi<T>[]>
  editing: EditingApi
  selection: SelectionApi<T>
  pageSize: () => number
}

/** Build the keyboard-nav focus model over the displayed rows + columns. */
export function buildNavSurface<T>(inputs: NavInputsExtra<T>): NavApi {
  return buildNav<T>({
    navMode: inputs.navMode,
    rows: inputs.rows,
    visibleLeafColumns: inputs.visibleLeafColumns,
    isEditingActive: inputs.editing.isAnyEditing,
    toggleRowSelected: inputs.selection.toggleRowSelected,
    startEditing: inputs.editing.startEditing,
    cancelEdit: inputs.editing.cancelEdit,
    pageSize: inputs.pageSize,
  })
}
