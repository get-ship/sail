// sui-sheet engine — range selection (the headline difference from Table).
//
// Selection is RANGE-shaped: `active` + `anchor` + `ranges[]` — a 2-D cell
// SPACE, not a row-id set. `setActiveCell` collapses to a 1×1 range (or, with
// `{extend}`, grows the active range FROM the anchor = shift-click); `setSelection`
// replaces or (with `{additive}`) appends a disjoint range (Ctrl-click). Every
// mutator writes the slice inside `batch()` and fires `onSelectionChange`. The
// per-cell `isInRange` derived is what the view reads to paint the overlay.
//
// PURE TypeScript — `batch`/`derived` from `@ship/sui-runtime` ONLY.
//
// Authority: build-spec §Features #4 (range selection), §"Authoritative … API"
// (SelectionState + isInRange), selection.test.ts.

import { batch, derived, type Derived } from '@ship/sui-runtime'
import { ensureRange, makeRange } from './range'
import { makeSelection, type InteractionState } from './state'
import type { CellAddress, CellRange, SelectionState } from './types'

export interface SelectionApi {
  selection: Derived<SelectionState>
  activeCell: Derived<CellAddress>
  isInRange(a: CellAddress): Derived<boolean>
  setActiveCell(a: CellAddress, opts?: { extend?: boolean }): void
  setSelection(
    ranges: CellRange[] | { start: CellAddress; end: CellAddress }[],
    active?: CellAddress,
    opts?: { additive?: boolean },
  ): void
}

export function buildSelection(
  state: InteractionState,
  onSelectionChange?: (sel: SelectionState) => void,
): SelectionApi {
  const selection = derived<SelectionState>(() => state.selection())
  const activeCell = derived<CellAddress>(() => state.selection().active)

  const emit = (sel: SelectionState): void => {
    onSelectionChange?.(sel)
  }

  const setActiveCell = (a: CellAddress, opts?: { extend?: boolean }): void => {
    batch(() => {
      const prev = state.selection()
      if (opts?.extend) {
        // Grow the active range FROM the anchor toward the new active cell.
        const range = makeRange(prev.anchor, a)
        const sel = makeSelection(a, prev.anchor, [range])
        state.selection.set(sel)
        emit(sel)
        return
      }
      // Collapse to a 1×1 selection at `a`; it becomes the new anchor.
      const sel = makeSelection(a, a, [makeRange(a, a)])
      state.selection.set(sel)
      emit(sel)
    })
  }

  const setSelection = (
    ranges: CellRange[] | { start: CellAddress; end: CellAddress }[],
    active?: CellAddress,
    opts?: { additive?: boolean },
  ): void => {
    batch(() => {
      const made = ranges.map(ensureRange)
      const prev = state.selection()
      const all = opts?.additive ? [...prev.ranges, ...made] : made
      const last = made[made.length - 1]
      const nextActive = active ?? last?.normalized().start ?? prev.active
      const nextAnchor = opts?.additive ? prev.anchor : (last?.normalized().start ?? nextActive)
      const sel = makeSelection(nextActive, nextAnchor, all)
      state.selection.set(sel)
      emit(sel)
    })
  }

  const isInRange = (a: CellAddress): Derived<boolean> =>
    derived<boolean>(() => state.selection().ranges.some((r) => r.contains(a)))

  return { selection, activeCell, isInRange, setActiveCell, setSelection }
}
