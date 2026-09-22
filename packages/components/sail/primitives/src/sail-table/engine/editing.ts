// sail-table engine — cell editing model (Backbone #1, Slice 4, Feature #15).
//
// The REAL editing state slice that backs the `sail-cell-edit` event — the event
// is NOT shipped without it. `editing` is `{ rowId, columnId } | null`; only one
// cell edits at a time (a new `startEditing` replaces the previous target).
//
//   startEditing(rowId, columnId)  → gate on the column's `enableEditing`; set slice
//   commitEdit(value)              → run the column validation hook; on pass write
//                                    the value back to the row's record + clear slice
//                                    + fire onCellEdit; on fail keep the slice open
//   cancelEdit()                   → clear slice WITHOUT writing
//
// The value write-back mutates the editing row's original record by the column's
// `accessorKey`, then re-emits the data signal so the row pipeline recomputes and
// `row.getValue()` reflects the commit. `isEditing(rowId, columnId)` is the
// query the Cell `getIsEditing()` reflects (tracks the slice in a derived).
//
// PURE TypeScript — imports `{ batch }` from `@ship/sail-runtime` ONLY. No DOM,
// no SDK, no network.
//
// Authority: build-spec §Features #15 ("Cell editing": per-column enableEditing;
// editing slice = { rowId, columnId } | null; startEditing/commitEdit(value)/
// cancelEdit; optional validation hook), §"Cell<T>" (getIsEditing), Slice 4.

import { batch, type Signal } from '@ship/sail-runtime'
import type { CellValue, ColumnDef, EditingState } from './types'

export interface EditingInputs<T> {
  editing: Signal<EditingState | null>
  /** The live data source — mutated + re-emitted on a committed value. */
  data: Signal<T[]>
  columns: ColumnDef<T>[]
  getRowId: (row: T, index: number) => string
  /** Fired on a committed edit (the sail-cell-edit event payload). */
  onCellEdit?: (e: { rowId: string; columnId: string; value: CellValue }) => void
  /** Snapshot fired after a slice write (centralized state ownership). */
  onStateChange?: () => void
}

export interface EditingApi {
  startEditing: (rowId: string, columnId: string) => void
  commitEdit: (value: CellValue) => void
  cancelEdit: () => void
  /** Query the slice — the Cell `getIsEditing()` reflects this (tracks). */
  isEditing: (rowId: string, columnId: string) => boolean
  /** Is ANY cell currently editing? Nav suspends while true. */
  isAnyEditing: () => boolean
}

/** Is a column editable? Only accessor columns opting IN with `enableEditing`. */
function isEditable<T>(def: ColumnDef<T> | undefined): boolean {
  if (!def) return false
  return (
    'accessorKey' in def &&
    (def as { accessorKey?: unknown }).accessorKey != null &&
    (def as { enableEditing?: boolean }).enableEditing === true
  )
}

/** The column's optional validation hook (lives on `meta.validate`). */
function validatorOf<T>(def: ColumnDef<T> | undefined): ((v: CellValue) => boolean) | undefined {
  const v = def?.meta?.['validate']
  return typeof v === 'function' ? (v as (value: CellValue) => boolean) : undefined
}

export function buildEditing<T>(inputs: EditingInputs<T>): EditingApi {
  const { editing, data, columns, getRowId, onCellEdit, onStateChange } = inputs
  const byId = new Map(columns.map((c) => [c.id, c]))

  const isEditing = (rowId: string, columnId: string): boolean => {
    const e = editing()
    return e !== null && e.rowId === rowId && e.columnId === columnId
  }
  const isAnyEditing = (): boolean => editing() !== null

  function startEditing(rowId: string, columnId: string): void {
    if (!isEditable(byId.get(columnId))) return // enableEditing gate
    batch(() => editing.set({ rowId, columnId }))
    onStateChange?.()
  }

  function cancelEdit(): void {
    if (editing() === null) return
    batch(() => editing.set(null))
    onStateChange?.()
  }

  function commitEdit(value: CellValue): void {
    const target = editing()
    if (target === null) return
    const def = byId.get(target.columnId)
    // Validation hook — a failing validation REJECTS the commit (slice stays open).
    const validate = validatorOf(def)
    if (validate && !validate(value)) return

    // Write the value back by an IMMUTABLE update: replace the editing record
    // with a shallow copy carrying the new field in a NEW array, then re-emit
    // the data signal (new identity → suffix recompute; the consumer's original
    // record/array is never mutated — Cardinal 2 immutability).
    const key = (def as { accessorKey?: string } | undefined)?.accessorKey
    if (key != null) {
      const rows = data()
      const idx = rows.findIndex((r, i) => getRowId(r, i) === target.rowId)
      if (idx >= 0) {
        const next = rows.slice()
        next[idx] = { ...(rows[idx] as object), [key]: value } as T
        batch(() => {
          data.set(next)
          editing.set(null)
        })
        onCellEdit?.({ rowId: target.rowId, columnId: target.columnId, value })
        onStateChange?.()
        return
      }
    }
    // No accessor key (display column) — clear the slice; the value is the
    // consumer's to persist via onCellEdit.
    batch(() => editing.set(null))
    onCellEdit?.({ rowId: target.rowId, columnId: target.columnId, value })
    onStateChange?.()
  }

  return { startEditing, commitEdit, cancelEdit, isEditing, isAnyEditing }
}
