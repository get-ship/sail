// sui-table engine — row selection (Backbone #1, Slice 2, Feature #7).
//
// A selection model keyed by `getRowId` so a selection survives reorder/repage.
// The state is an include/exclude representation (`{ mode, ids: Set }`) so the
// server-mode "all matching, minus 3" is expressible WITHOUT enumerating 100k
// ids: `toggleAllRowsSelected(true)` flips to `mode:'exclude'` with an empty
// set, and de-selecting a row in that mode ADDS it to the exclusion set.
//
// `toggleRowSelected(id, v?, { range })` selects the contiguous span between the
// `selectionAnchor` (the last plain toggle) and the target over the currently
// DISPLAYED order (post sort/filter/page). A per-row `enableRowSelection`
// predicate gates selectability — a disabled row cannot be selected and is
// skipped inside a range span. A group row cascades to its leaf children and
// drives tri-state via `getIsSomeSelected()`.
//
// PURE TypeScript — imports `derived` + `batch` from `@ship/sui-runtime`. No DOM,
// no SDK.
//
// Authority: build-spec §Features #7 ("Row selection"), §"Signal graph"
// (rowSelection +anchor, include/exclude → selectedRowIds / getIsAll*Selected),
// Slice 2.

import { batch, derived, type Derived, type Signal } from '@ship/sui-runtime'
import type { Row, RowSelectionState } from './types'

export interface SelectionInputs<T> {
  rowSelection: Signal<RowSelectionState>
  selectionAnchor: Signal<string | null>
  /** The displayed (post sort/filter/page) order — the range/select-all scope. */
  displayedRows: Derived<Row<T>[]>
  /** Per-row selectable predicate (table-level `enableRowSelection`). */
  canSelect: (row: Row<T>) => boolean
  fireChange: (sel: RowSelectionState) => void
}

export interface SelectionApi<T> {
  selectedRowIds: Derived<string[]>
  getIsAllRowsSelected: Derived<boolean>
  getIsAllPageRowsSelected: Derived<boolean>
  getIsSomePageRowsSelected: Derived<boolean>
  /** Query: is THIS id currently selected (honors include/exclude + predicate). */
  isSelected: (id: string) => boolean
  canSelectId: (id: string) => boolean
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
}

/** Selectable rows in the current displayed order. */
function selectableDisplayed<T>(
  rows: Row<T>[],
  canSelect: (row: Row<T>) => boolean,
): Row<T>[] {
  return rows.filter((r) => canSelect(r))
}

export function buildSelection<T>(inputs: SelectionInputs<T>): SelectionApi<T> {
  const { rowSelection, selectionAnchor, displayedRows, canSelect, fireChange } = inputs

  /** Resolve a Row<T> by id from the displayed set (range/cascade lookups). */
  const rowById = (id: string): Row<T> | undefined =>
    displayedRows().find((r) => r.id === id)

  const canSelectId = (id: string): boolean => {
    const row = rowById(id)
    return row ? canSelect(row) : false
  }

  /** include: id ∈ ids; exclude: id ∉ ids (the set is the OFF list). */
  const isSelected = (id: string): boolean => {
    if (!canSelectId(id)) return false
    const sel = rowSelection()
    return sel.mode === 'include' ? sel.ids.has(id) : !sel.ids.has(id)
  }

  const selectedRowIds = derived<string[]>(() => {
    const sel = rowSelection()
    const rows = selectableDisplayed(displayedRows(), canSelect)
    if (sel.mode === 'include') {
      return rows.filter((r) => sel.ids.has(r.id)).map((r) => r.id)
    }
    return rows.filter((r) => !sel.ids.has(r.id)).map((r) => r.id)
  })

  const getIsAllRowsSelected = derived<boolean>(() => {
    const rows = selectableDisplayed(displayedRows(), canSelect)
    if (rows.length === 0) return false
    return rows.every((r) => isSelected(r.id))
  })

  const getIsAllPageRowsSelected = getIsAllRowsSelected
  const getIsSomePageRowsSelected = derived<boolean>(() => {
    const rows = selectableDisplayed(displayedRows(), canSelect)
    const some = rows.some((r) => isSelected(r.id))
    return some && !rows.every((r) => isSelected(r.id))
  })

  function setRowSelection(
    u: RowSelectionState | ((prev: RowSelectionState) => RowSelectionState),
  ): void {
    batch(() => {
      const next = typeof u === 'function' ? u(rowSelection()) : u
      rowSelection.set(next)
    })
    fireChange(rowSelection())
  }

  /** Write `id`'s selected-bit honoring the current include/exclude mode. */
  function applyBit(sel: RowSelectionState, id: string, on: boolean): RowSelectionState {
    const ids = new Set(sel.ids)
    if (sel.mode === 'include') {
      if (on) ids.add(id)
      else ids.delete(id)
    } else {
      // exclude mode: the set is the OFF list — selecting removes from it.
      if (on) ids.delete(id)
      else ids.add(id)
    }
    return { mode: sel.mode, ids }
  }

  function toggleRowSelected(
    id: string,
    v?: boolean,
    opts?: { range?: boolean; additive?: boolean },
  ): void {
    if (!canSelectId(id)) return

    if (opts?.range) {
      selectRange(id)
      return
    }

    const target = v ?? !isSelected(id)
    batch(() => {
      rowSelection.set(applyBit(rowSelection(), id, target))
      selectionAnchor.set(id)
    })
    fireChange(rowSelection())
  }

  /** Span the anchor→target window over the displayed order, gating each row by
   *  the predicate; with no anchor the target alone is selected. */
  function selectRange(targetId: string): void {
    const rows = displayedRows()
    const anchor = selectionAnchor() ?? targetId
    const ai = rows.findIndex((r) => r.id === anchor)
    const ti = rows.findIndex((r) => r.id === targetId)
    if (ti < 0) return
    const lo = ai < 0 ? ti : Math.min(ai, ti)
    const hi = ai < 0 ? ti : Math.max(ai, ti)
    const span = rows.slice(lo, hi + 1).filter((r) => canSelect(r))

    batch(() => {
      let sel = rowSelection()
      for (const r of span) sel = applyBit(sel, r.id, true)
      rowSelection.set(sel)
      // The anchor stays put (the range pivots around it).
    })
    fireChange(rowSelection())
  }

  function toggleAllPageRowsSelected(v?: boolean): void {
    const rows = selectableDisplayed(displayedRows(), canSelect)
    const on = v ?? !rows.every((r) => isSelected(r.id))
    batch(() => {
      let sel: RowSelectionState = { mode: 'include', ids: new Set(rowSelection().ids) }
      // Normalize to include mode for a page-scoped op, then set each page row.
      if (rowSelection().mode === 'exclude') {
        sel = { mode: 'include', ids: new Set(selectedRowIds()) }
      }
      for (const r of rows) sel = applyBit(sel, r.id, on)
      rowSelection.set(sel)
    })
    fireChange(rowSelection())
  }

  function toggleAllRowsSelected(v?: boolean): void {
    const on = v ?? !getIsAllRowsSelected()
    batch(() => {
      // exclude+{} = "everything selected" without enumerating ids; include+{} =
      // "nothing selected".
      rowSelection.set({ mode: on ? 'exclude' : 'include', ids: new Set<string>() })
      selectionAnchor.set(null)
    })
    fireChange(rowSelection())
  }

  function resetRowSelection(): void {
    batch(() => {
      rowSelection.set({ mode: 'include', ids: new Set<string>() })
      selectionAnchor.set(null)
    })
    fireChange(rowSelection())
  }

  return {
    selectedRowIds,
    getIsAllRowsSelected,
    getIsAllPageRowsSelected,
    getIsSomePageRowsSelected,
    isSelected,
    canSelectId,
    setRowSelection,
    toggleRowSelected,
    toggleAllPageRowsSelected,
    toggleAllRowsSelected,
    resetRowSelection,
  }
}
