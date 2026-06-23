// TESTS-FIRST — sui-sheet FORMULA slice (Slice 3): formula eval through createSheet.
//
// Status: RED. The formula path is DEFERRED in the committed foundational slice:
// `cell-api.ts` resolves a formula cell to `null`, and `create-sheet.ts` stubs
// `dependents`/`precedents`/`hasCycle`/`recalc`. These specs fail against that
// shipped behavior until the formula slice wires the DAG + recalc + the
// FormulaCore eval into the engine. DO NOT add the impl here. DO NOT weaken into
// stubs.
//
// Authority: build-spec §Features #2 ("Cell DAG + recalc"), §"Signal graph"
// ("a write to B2 recomputes B2 and its transitive dependents, not the whole
// sheet"), §"API sketch" (dependents/precedents/hasCycle/recalc; recalcMode),
// §States ("#CYCLE!/#REF!/#NAME? are per-cell errors … they do NOT fault the
// whole grid"), Behavioral spec #2 ("Literals and formulas resolve — a cell
// =A1+A2 shows the sum; editing A1 recomputes the formula cell and only it + its
// dependents; =SUM(A1:A3) evaluates a range"), #3 ("Formula errors are per-cell,
// not fatal — =1/0 shows #DIV/0!; =NOTAFUNC() shows #NAME?; a self-reference
// shows #CYCLE!; the rest of the grid renders fine").
//
// These specs inject the NOOP core where they exercise only the literal/ref/DAG
// path (deterministic, vendor-free), and inject `createFastFormulaCore()` only
// for function-library specs (SUM/range). Swapping the core is behavioral spec
// #16 — eval changes, the engine does not.
//
// Engine purity (Cardinal 4): imports ONLY @ship/sui-runtime + the engine.

import { describe, expect, it, vi } from 'vitest'
// RED: greenfield modules — the formula adapters land with the formula slice.
import { createSheet } from './create-sheet'
import { createNoopFormulaCore } from './formula/adapter-noop'
import { createFastFormulaCore } from './formula/adapter-fast-formula-parser'
import type { WorkbookDef } from './contract'

function book(cells: Record<string, { value?: unknown; formula?: string }>): WorkbookDef {
  return {
    sheets: [{ id: 's1', name: 'Sheet1', rowCount: 100, colCount: 26, cells: cells as never }],
    activeSheetId: 's1',
  }
}

const noopSheet = (cells: Record<string, { value?: unknown; formula?: string }>) =>
  createSheet({ workbook: book(cells), formulaCore: createNoopFormulaCore() })

describe('createSheet — a formula cell resolves to its evaluated value', () => {
  it('=A1+A2 displays the sum (not null — the literal-path deferral is gone)', () => {
    const s = noopSheet({ A1: { value: 2 }, A2: { value: 3 }, A3: { formula: '=A1+A2' } })
    expect(s.getCell('A3')().value).toBe(5)
    expect(s.getCell('A3')().display).toBe('5')
  })

  it('a formula cell is flagged isFormula with the raw formula preserved', () => {
    const s = noopSheet({ A1: { value: 1 }, B1: { formula: '=A1+1' } })
    const cell = s.getCell('B1')()
    expect(cell.isFormula).toBe(true)
    expect(cell.raw.formula).toBe('=A1+1')
  })

  it('=A1*2 chains through a single precedent', () => {
    const s = noopSheet({ A1: { value: 21 }, B1: { formula: '=A1*2' } })
    expect(s.getCell('B1')().value).toBe(42)
  })
})

describe('createSheet — editing a precedent recomputes ONLY dependents (dirty suffix)', () => {
  it('editing A1 recomputes the =A1+A2 cell', () => {
    const s = noopSheet({ A1: { value: 2 }, A2: { value: 3 }, A3: { formula: '=A1+A2' } })
    expect(s.getCell('A3')().value).toBe(5)
    s.setCell('A1', { value: 10 })
    expect(s.getCell('A3')().value).toBe(13)
  })

  it('a transitive chain A1→B1→C1 recomputes all the way down on an A1 edit', () => {
    const s = noopSheet({
      A1: { value: 1 },
      B1: { formula: '=A1+1' },
      C1: { formula: '=B1+1' },
    })
    expect(s.getCell('C1')().value).toBe(3)
    s.setCell('A1', { value: 10 })
    expect(s.getCell('B1')().value).toBe(11)
    expect(s.getCell('C1')().value).toBe(12)
  })

  it('an unrelated cell does NOT recompute when a precedent changes', () => {
    const s = noopSheet({
      A1: { value: 1 },
      B1: { formula: '=A1+1' },
      Z9: { formula: '=5+5' }, // no dependency on A1
    })
    const z9 = s.getCell('Z9')
    expect(z9().value).toBe(10)
    let z9Recomputes = 0
    const before = z9().value
    s.setCell('A1', { value: 99 })
    if (z9().value !== before) z9Recomputes++
    expect(z9Recomputes).toBe(0)
    expect(s.getCell('B1')().value).toBe(100)
  })
})

describe('createSheet — dependents/precedents/hasCycle are real (not stubbed)', () => {
  it('dependents(A1) lists the formula cells referencing A1', () => {
    const s = noopSheet({ A1: { value: 1 }, B1: { formula: '=A1+1' }, C1: { formula: '=A1*2' } })
    expect(s.dependents('A1')()).toEqual(expect.arrayContaining(['B1', 'C1']))
  })

  it('precedents(B1) lists the cells B1 references', () => {
    const s = noopSheet({ A1: { value: 1 }, A2: { value: 2 }, B1: { formula: '=A1+A2' } })
    expect(s.precedents('B1')()).toEqual(expect.arrayContaining(['A1', 'A2']))
  })
})

describe('createSheet — per-cell formula errors do NOT fault the grid', () => {
  it('=1/0 → #DIV/0! in that cell, neighbours render fine', () => {
    const s = noopSheet({ A1: { value: 'ok' }, B1: { formula: '=1/0' } })
    const b1 = s.getCell('B1')()
    expect(b1.error).toBe('#DIV/0!')
    expect(b1.display).toBe('#DIV/0!')
    expect(s.getCell('A1')().value).toBe('ok') // grid intact
  })

  it('=NOTAFUNC() → #NAME?', () => {
    const s = noopSheet({ A1: { formula: '=NOTAFUNC()' } })
    expect(s.getCell('A1')().error).toBe('#NAME?')
  })

  it('a self-reference (A1 = A1) → #CYCLE! and sets hasCycle', () => {
    const s = noopSheet({ A1: { formula: '=A1' } })
    expect(s.getCell('A1')().error).toBe('#CYCLE!')
    expect(s.hasCycle()).toBe(true)
  })

  it('a two-cell cycle marks BOTH cells #CYCLE!', () => {
    const s = noopSheet({ A1: { formula: '=B1' }, B1: { formula: '=A1' } })
    expect(s.getCell('A1')().error).toBe('#CYCLE!')
    expect(s.getCell('B1')().error).toBe('#CYCLE!')
    expect(s.hasCycle()).toBe(true)
  })

  it('a reference to a deleted/out-of-range cell surfaces #REF!', () => {
    // A formula whose reference falls outside the sheet's logical extent is #REF!.
    const s = createSheet({
      workbook: {
        sheets: [{ id: 's1', name: 'Sheet1', rowCount: 2, colCount: 2, cells: { A1: { formula: '=Z99' } } }],
        activeSheetId: 's1',
      },
      formulaCore: createNoopFormulaCore(),
    })
    expect(s.getCell('A1')().error).toBe('#REF!')
  })
})

describe('createSheet — range formulas via the vendored function library', () => {
  it('=SUM(A1:A3) evaluates the range', () => {
    const s = createSheet({
      workbook: book({ A1: { value: 1 }, A2: { value: 2 }, A3: { value: 3 }, B1: { formula: '=SUM(A1:A3)' } }),
      formulaCore: createFastFormulaCore(),
    })
    expect(s.getCell('B1')().value).toBe(6)
  })

  it('editing a cell INSIDE the summed range recomputes the SUM', () => {
    const s = createSheet({
      workbook: book({ A1: { value: 1 }, A2: { value: 2 }, A3: { value: 3 }, B1: { formula: '=SUM(A1:A3)' } }),
      formulaCore: createFastFormulaCore(),
    })
    expect(s.getCell('B1')().value).toBe(6)
    s.setCell('A2', { value: 20 })
    expect(s.getCell('B1')().value).toBe(24)
  })
})

describe('createSheet — recalcMode: manual coalesces dirty cells until recalc()', () => {
  it("'auto' (default) recomputes on every write", () => {
    const s = noopSheet({ A1: { value: 1 }, B1: { formula: '=A1+1' } })
    s.setCell('A1', { value: 5 })
    expect(s.getCell('B1')().value).toBe(6)
  })

  it("'manual' defers the recompute until recalc() is called", () => {
    const s = createSheet({
      workbook: book({ A1: { value: 1 }, B1: { formula: '=A1+1' } }),
      formulaCore: createNoopFormulaCore(),
      recalcMode: 'manual',
    })
    expect(s.getCell('B1')().value).toBe(2)
    s.setCell('A1', { value: 100 })
    expect(s.getCell('B1')().value).toBe(2) // stale until recalc
    s.recalc()
    expect(s.getCell('B1')().value).toBe(101)
  })
})

describe('createSheet — formulaCore is injectable (vendor boundary, spec #16)', () => {
  it('the noop core returns #NAME? for SUM; the fast core computes it — same engine', () => {
    const noop = noopSheet({ A1: { value: 1 }, A2: { value: 2 }, B1: { formula: '=SUM(A1:A2)' } })
    expect(noop.getCell('B1')().error).toBe('#NAME?')

    const fast = createSheet({
      workbook: book({ A1: { value: 1 }, A2: { value: 2 }, B1: { formula: '=SUM(A1:A2)' } }),
      formulaCore: createFastFormulaCore(),
    })
    expect(fast.getCell('B1')().value).toBe(3)
  })

  it('a formula commit fires onCellChange (persistence seam unchanged by eval)', () => {
    const onCellChange = vi.fn()
    const s = createSheet({
      workbook: book({ A1: { value: 1 } }),
      formulaCore: createNoopFormulaCore(),
      onCellChange,
    })
    s.setCell('B1', { formula: '=A1+1' })
    expect(onCellChange).toHaveBeenCalledTimes(1)
    expect(s.getCell('B1')().value).toBe(2)
  })
})
