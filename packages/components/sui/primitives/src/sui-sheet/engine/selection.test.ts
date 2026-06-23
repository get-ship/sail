// TESTS-FIRST — sui-sheet FOUNDATIONAL slice: range selection.
//
// Status: RED. `./create-sheet` does not exist yet (greenfield pack). The import
// is import-not-found until the foundational slice ships. DO NOT add the impl
// here. DO NOT weaken into stubs.
//
// Authority: build-spec §Features #4 ("Range selection — SelectionState is
// range-shaped (active + anchor + ranges[]), NOT a row-id set (the headline
// divergence from sui-table's rowSelection). Shift-extend grows the active range
// from the anchor; Ctrl adds a disjoint range; row/col-header click selects whole
// row/col; Ctrl+A selects the used range then the whole sheet"), §"Authoritative
// Cell / Selection / Range / DAG API" (SelectionState members; CellRange.contains
// / normalized / width / height / toA1; isInRange Derived), §API sketch
// (setActiveCell / setSelection, onSelectionChange).
//
// Range selection is the headline difference from the table data-grid — a 2-D
// cell SPACE, not a record set. No formula eval needed (DEFERRED).
//
// Engine purity (Cardinal 4): imports ONLY @ship/sui-runtime.

import { describe, expect, it, vi } from 'vitest'
// RED: greenfield module.
import { createSheet } from './create-sheet'
import type { WorkbookDef } from './contract'

function book(): WorkbookDef {
  return {
    sheets: [{ id: 's1', name: 'Sheet1', rowCount: 100, colCount: 26, cells: {} }],
    activeSheetId: 's1',
  }
}

const sheet = () => createSheet({ workbook: book() })

describe('selection — active cell + anchor', () => {
  it('setActiveCell collapses the selection to a 1×1 range at that cell', () => {
    const s = sheet()
    s.setActiveCell({ row: 1, col: 1 })
    expect(s.activeCell()).toEqual({ row: 1, col: 1 })
    const sel = s.selection()
    expect(sel.active).toEqual({ row: 1, col: 1 })
    expect(sel.anchor).toEqual({ row: 1, col: 1 })
    expect(sel.ranges.length).toBe(1)
  })

  it('setActiveCell({extend}) grows the active range FROM the anchor (shift-click)', () => {
    const s = sheet()
    s.setActiveCell({ row: 0, col: 0 }) // anchor
    s.setActiveCell({ row: 2, col: 2 }, { extend: true })
    const sel = s.selection()
    expect(sel.anchor).toEqual({ row: 0, col: 0 })
    expect(sel.active).toEqual({ row: 2, col: 2 })
    const r = sel.ranges[0]!.normalized()
    expect(r.start).toEqual({ row: 0, col: 0 })
    expect(r.end).toEqual({ row: 2, col: 2 })
  })
})

describe('selection — isInRange / isInSelection per cell (Derived)', () => {
  it('isInRange is true for cells inside the active range, false outside', () => {
    const s = sheet()
    s.setSelection([{ start: { row: 0, col: 0 }, end: { row: 2, col: 2 } } as never])
    expect(s.isInRange({ row: 1, col: 1 })()).toBe(true)
    expect(s.isInRange({ row: 5, col: 5 })()).toBe(false)
  })

  it('the resolved cell carries isInSelection / isActive flags the view reads', () => {
    const wb: WorkbookDef = {
      sheets: [{ id: 's1', name: 'Sheet1', rowCount: 10, colCount: 10, cells: { A1: { value: 1 } } }],
      activeSheetId: 's1',
    }
    const s = createSheet({ workbook: wb })
    s.setActiveCell({ row: 0, col: 0 })
    const a1 = s.getCell('A1')()
    expect(a1.isActive).toBe(true)
    expect(a1.isInSelection).toBe(true)
    expect(s.getCell('B5')().isActive).toBe(false)
  })
})

describe('selection — shift-extend grows from the anchor', () => {
  it('extending right then down from an anchor produces one normalized range', () => {
    const s = sheet()
    s.setActiveCell({ row: 1, col: 1 })
    s.setSelection(
      [{ start: { row: 1, col: 1 }, end: { row: 3, col: 4 } } as never],
      { row: 3, col: 4 },
    )
    const r = s.selection().ranges[0]!.normalized()
    expect(r.width()).toBe(4) // cols 1..4
    expect(r.height()).toBe(3) // rows 1..3
  })
})

describe('selection — Ctrl adds a disjoint range (multi-range)', () => {
  it('a second setSelection with additive grows the ranges array', () => {
    const s = sheet()
    s.setSelection([{ start: { row: 0, col: 0 }, end: { row: 0, col: 0 } } as never])
    s.setSelection(
      [{ start: { row: 5, col: 5 }, end: { row: 5, col: 5 } } as never],
      { row: 5, col: 5 },
      { additive: true },
    )
    expect(s.selection().ranges.length).toBe(2)
    expect(s.isInRange({ row: 0, col: 0 })()).toBe(true)
    expect(s.isInRange({ row: 5, col: 5 })()).toBe(true)
    expect(s.isInRange({ row: 3, col: 3 })()).toBe(false)
  })
})

describe('selection — whole-row / whole-column selection', () => {
  it('selecting a whole column spans every row of that column', () => {
    const s = sheet()
    s.setSelection([
      { start: { row: 0, col: 2 }, end: { row: 99, col: 2 } } as never,
    ])
    const sel = s.selection()
    expect(s.isInRange({ row: 0, col: 2 })()).toBe(true)
    expect(s.isInRange({ row: 99, col: 2 })()).toBe(true)
    expect(s.isInRange({ row: 50, col: 1 })()).toBe(false)
    expect(sel.ranges[0]!.height()).toBe(100)
  })

  it('selecting a whole row spans every column of that row', () => {
    const s = sheet()
    s.setSelection([
      { start: { row: 4, col: 0 }, end: { row: 4, col: 25 } } as never,
    ])
    expect(s.isInRange({ row: 4, col: 0 })()).toBe(true)
    expect(s.isInRange({ row: 4, col: 25 })()).toBe(true)
    expect(s.isInRange({ row: 3, col: 10 })()).toBe(false)
    expect(s.selection().ranges[0]!.width()).toBe(26)
  })
})

describe('selection — fires onSelectionChange (the engine→consumer write path)', () => {
  it('onSelectionChange fires with the active cell + ranges on every move', () => {
    const onSelectionChange = vi.fn()
    const s = createSheet({ workbook: book(), onSelectionChange })
    s.setActiveCell({ row: 2, col: 3 })
    expect(onSelectionChange).toHaveBeenCalled()
    const [sel] = onSelectionChange.mock.calls.at(-1)!
    expect(sel.active).toEqual({ row: 2, col: 3 })
    expect(Array.isArray(sel.ranges)).toBe(true)
  })
})

describe('selection — CellRange algebra (the range value type)', () => {
  it("normalized() returns top-left / bottom-right regardless of drag direction", () => {
    const s = sheet()
    // drag from bottom-right up to top-left
    s.setSelection([{ start: { row: 3, col: 3 }, end: { row: 1, col: 1 } } as never])
    const r = s.selection().ranges[0]!.normalized()
    expect(r.start).toEqual({ row: 1, col: 1 })
    expect(r.end).toEqual({ row: 3, col: 3 })
  })

  it('contains(a) reports membership; toA1() emits the A1 range string', () => {
    const s = sheet()
    s.setSelection([{ start: { row: 0, col: 0 }, end: { row: 2, col: 2 } } as never])
    const r = s.selection().ranges[0]!
    expect(r.contains({ row: 1, col: 1 })).toBe(true)
    expect(r.contains({ row: 9, col: 9 })).toBe(false)
    expect(r.toA1()).toBe('A1:C3')
  })
})
