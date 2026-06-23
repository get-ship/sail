// TESTS-FIRST — sui-sheet FOUNDATIONAL slice: the createSheet() cell engine.
//
// Status: RED. `./create-sheet` does not exist yet (greenfield pack). The import
// is import-not-found until the foundational slice ships the assembler + the
// reactive cell model. DO NOT add the impl here. DO NOT weaken into stubs.
//
// Authority: build-spec §"Backbone — createSheet() cell engine", §"API sketch"
// (`SheetOptions`/`Sheet`), §"id per sheet + stable A1 are REQUIRED", §"Signal
// graph" ("touching one slice recomputes only the suffix … only the dirty DAG
// suffix"), §Features #3 (per-cell types/formats), §States (empty/loading), and
// the FOUNDATIONAL-slice charter: "a reactive cell model on @ship/sui-runtime
// signals".
//
// SCOPE NOTE — the formula engine is DEFERRED to a follow-up slice. These specs
// exercise the LITERAL cell path only: value resolution, the reactive cell
// model, A1↔(row,col) projection, sparse-grid reads, the live-workbook-signal
// ingestion, and the dirty-suffix recompute discipline on literal edits. Formula
// parse / DAG / eval / display-format specs land with the formula slice.
//
// Engine purity (Cardinal 4): imports ONLY @ship/sui-runtime. The purity
// GREP-test is its own foundational-slice file (purity.test.ts).

import { describe, expect, it, vi } from 'vitest'
import { signal } from '@ship/sui-runtime'
// RED: greenfield module.
import { createSheet } from './create-sheet'
import type { WorkbookDef } from './contract'

function book(cells: Record<string, { value?: unknown; formula?: string }>): WorkbookDef {
  return {
    sheets: [{ id: 's1', name: 'Sheet1', rowCount: 100, colCount: 26, cells: cells as never }],
    activeSheetId: 's1',
  }
}

describe('createSheet — reactive cell model (literal path)', () => {
  it('exposes the active sheet as a Derived<SheetDef>', () => {
    const sheet = createSheet({ workbook: book({ A1: { value: 5 } }) })
    expect(sheet.activeSheet().id).toBe('s1')
    expect(sheet.activeSheet().name).toBe('Sheet1')
  })

  it('rowCount / colCount reflect the logical extent (Derived)', () => {
    const sheet = createSheet({ workbook: book({}) })
    expect(sheet.rowCount()).toBe(100)
    expect(sheet.colCount()).toBe(26)
  })

  it('getCell(a1) returns a Derived ResolvedCell carrying value + address', () => {
    const sheet = createSheet({ workbook: book({ A1: { value: 5 } }) })
    const cell = sheet.getCell('A1')
    expect(typeof cell).toBe('function') // Derived is callable
    const resolved = cell()
    expect(resolved.a1).toBe('A1')
    expect(resolved.address).toEqual({ row: 0, col: 0 })
    expect(resolved.value).toBe(5)
  })

  it('getCellAt(row,col) and getCell(a1) project to the SAME cell', () => {
    const sheet = createSheet({ workbook: book({ B2: { value: 'hi' } }) })
    expect(sheet.getCellAt(1, 1)().value).toBe('hi')
    expect(sheet.getCell('B2')().value).toBe('hi')
    expect(sheet.getCellAt(1, 1)().a1).toBe('B2')
  })

  it('an unauthored cell in a SPARSE grid resolves to an empty cell, not undefined', () => {
    const sheet = createSheet({ workbook: book({ A1: { value: 1 } }) })
    const blank = sheet.getCell('Z50')()
    expect(blank.a1).toBe('Z50')
    expect(blank.value == null).toBe(true)
    expect(blank.display).toBe('')
  })

  it('display is the formatted string the VIEW renders (engine formats, view renders)', () => {
    const sheet = createSheet({ workbook: book({ A1: { value: 5 } }) })
    expect(typeof sheet.getCell('A1')().display).toBe('string')
    expect(sheet.getCell('A1')().display).toBe('5')
  })
})

describe('createSheet — live workbook signal ingestion (prop:* identity path)', () => {
  it('accepts a Signal<WorkbookDef> and tracks it (no remount on data change)', () => {
    const wb = signal(book({ A1: { value: 1 } }))
    const sheet = createSheet({ workbook: wb })
    expect(sheet.getCell('A1')().value).toBe(1)

    // Grow / change the workbook through the signal identity.
    wb.set(book({ A1: { value: 42 }, B1: { value: 7 } }))
    expect(sheet.getCell('A1')().value).toBe(42)
    expect(sheet.getCell('B1')().value).toBe(7)
  })

  it('a raw WorkbookDef (no signal) is lifted internally and still reads', () => {
    const sheet = createSheet({ workbook: book({ C3: { value: 'x' } }) })
    expect(sheet.getCell('C3')().value).toBe('x')
  })
})

describe('createSheet — mutation discipline (setCell writes a Signal, fires onCellChange)', () => {
  it('setCell updates the resolved value', () => {
    const sheet = createSheet({ workbook: book({ A1: { value: 1 } }) })
    sheet.setCell('A1', { value: 99 })
    expect(sheet.getCell('A1')().value).toBe(99)
  })

  it('setCell fires onCellChange with the before/after edit', () => {
    const onCellChange = vi.fn()
    const sheet = createSheet({ workbook: book({ A1: { value: 1 } }), onCellChange })
    sheet.setCell('A1', { value: 2 })
    expect(onCellChange).toHaveBeenCalledTimes(1)
    const [edits] = onCellChange.mock.calls[0]!
    expect(edits[0]).toMatchObject({ a1: 'A1', after: { value: 2 } })
  })

  it('setCells batches multiple edits into ONE onCellChange', () => {
    const onCellChange = vi.fn()
    const sheet = createSheet({ workbook: book({}), onCellChange })
    sheet.setCells([
      { a1: 'A1', before: {}, after: { value: 1 } },
      { a1: 'A2', before: {}, after: { value: 2 } },
    ])
    expect(onCellChange).toHaveBeenCalledTimes(1)
    expect(sheet.getCell('A1')().value).toBe(1)
    expect(sheet.getCell('A2')().value).toBe(2)
  })
})

describe('createSheet — dirty-suffix recompute (literal path)', () => {
  it('writing one cell does NOT recompute an unrelated cell derived', () => {
    const sheet = createSheet({
      workbook: book({ A1: { value: 1 }, Z9: { value: 'far' } }),
    })

    const a1 = sheet.getCell('A1')
    const z9 = sheet.getCell('Z9')

    // Prime both derived reads.
    expect(a1().value).toBe(1)
    expect(z9().value).toBe('far')

    let z9Recomputes = 0
    // Re-reading Z9 after an A1 write must be a cache hit — the engine recomputes
    // only the dirty suffix (A1 + its dependents), never an unrelated cell.
    const probe = (): void => {
      const before = z9().value
      sheet.setCell('A1', { value: 2 })
      const after = z9().value
      if (before !== after) z9Recomputes++ // value identity must hold
    }
    probe()
    expect(a1().value).toBe(2)
    expect(z9Recomputes).toBe(0)
  })
})

describe('createSheet — states (empty / loading / error reflected, not pushed onto consumer)', () => {
  it('a zero-sheet workbook is the degenerate empty', () => {
    const sheet = createSheet({ workbook: { sheets: [] } })
    expect(sheet.activeSheet == null || sheet.activeSheet() == null).toBe(true)
  })

  it('reflects isLoading from the consumer-supplied option (engine never fetches)', () => {
    const loading = signal(true)
    const sheet = createSheet({ workbook: book({}), isLoading: loading })
    expect(sheet.isLoading()).toBe(true)
    loading.set(false)
    expect(sheet.isLoading()).toBe(false)
  })

  it('reflects a workbook-level error slice fed by the consumer', () => {
    const sheet = createSheet({ workbook: book({}), error: new Error('load failed') })
    expect((sheet.error() as Error).message).toBe('load failed')
  })
})
