// TESTS-FIRST — sui-sheet INTERACTIVITY slice: the engine mutators that the
// surface drives (keyboard nav, fill-handle series, clipboard re-basing).
//
// Status: RED until the slice ships. `moveActive` and `fillRange` currently
// THROW the deferred error; the clipboard helpers (`rangeToTsv` / `tsvToEdits`)
// do not exist yet. These specs pin the PURE engine contract the controller's
// on:* handlers call; the component-level wiring lives in the test/ suite.
//
// Authority: build-spec §Features #4 (range selection + keyboard nav), #5 (fill
// handle relative-reference series), #6 (clipboard TSV + relative re-base),
// §"API sketch" (moveActive / fillRange), create-sheet.ts DEFERRED throws.

import { describe, expect, it } from 'vitest'
import { createSheet } from './create-sheet'
import { rangeToTsv, tsvToEdits } from './clipboard'
import { makeRange } from './range'
import type { WorkbookDef } from './contract'

function book(cells: Record<string, { value?: unknown; formula?: string }> = {}): WorkbookDef {
  return {
    sheets: [{ id: 's1', name: 'Sheet1', rowCount: 100, colCount: 26, cells: cells as never }],
    activeSheetId: 's1',
  }
}

describe('moveActive — keyboard nav within the logical extent', () => {
  it('down/right/up/left move the active cell one step', () => {
    const sheet = createSheet({ workbook: book() })
    sheet.setActiveCell({ row: 2, col: 2 }) // C3
    sheet.moveActive('down')
    expect(sheet.activeCell()).toEqual({ row: 3, col: 2 })
    sheet.moveActive('right')
    expect(sheet.activeCell()).toEqual({ row: 3, col: 3 })
    sheet.moveActive('up')
    expect(sheet.activeCell()).toEqual({ row: 2, col: 3 })
    sheet.moveActive('left')
    expect(sheet.activeCell()).toEqual({ row: 2, col: 2 })
  })

  it('clamps at the grid edges (no row -1 / no col past colCount)', () => {
    const sheet = createSheet({ workbook: book() })
    sheet.setActiveCell({ row: 0, col: 0 }) // A1
    sheet.moveActive('up')
    expect(sheet.activeCell()).toEqual({ row: 0, col: 0 })
    sheet.moveActive('left')
    expect(sheet.activeCell()).toEqual({ row: 0, col: 0 })
    sheet.setActiveCell({ row: 99, col: 25 }) // Z100 — the far corner of the 100×26 book
    sheet.moveActive('down')
    sheet.moveActive('right')
    expect(sheet.activeCell()).toEqual({ row: 99, col: 25 })
  })

  it("'home' / 'end' jump to the row's first / last column; ctrl-home/ctrl-end to the grid corners", () => {
    const sheet = createSheet({ workbook: book() })
    sheet.setActiveCell({ row: 5, col: 10 })
    sheet.moveActive('home')
    expect(sheet.activeCell()).toEqual({ row: 5, col: 0 })
    sheet.moveActive('end')
    expect(sheet.activeCell()).toEqual({ row: 5, col: 25 })
    sheet.moveActive('ctrl-home')
    expect(sheet.activeCell()).toEqual({ row: 0, col: 0 })
    sheet.moveActive('ctrl-end')
    expect(sheet.activeCell()).toEqual({ row: 99, col: 25 })
  })

  it('collapses the selection to a 1×1 range at the new active cell', () => {
    const sheet = createSheet({ workbook: book() })
    sheet.setActiveCell({ row: 0, col: 0 })
    sheet.moveActive('down')
    const sel = sheet.selection()
    expect(sel.ranges).toHaveLength(1)
    expect(sel.ranges[0]!.toA1()).toBe('A2')
    expect(sel.anchor).toEqual({ row: 1, col: 0 })
  })

  it('{extend:true} grows the selection range FROM the anchor (shift-arrow)', () => {
    const sheet = createSheet({ workbook: book() })
    sheet.setActiveCell({ row: 1, col: 1 }) // B2 — anchor
    sheet.moveActive('down', { extend: true })
    sheet.moveActive('right', { extend: true })
    const sel = sheet.selection()
    expect(sel.anchor).toEqual({ row: 1, col: 1 })
    expect(sel.active).toEqual({ row: 2, col: 2 })
    expect(sel.ranges[0]!.toA1()).toBe('B2:C3')
  })
})

describe('fillRange — relative-reference series fill', () => {
  it('copy-fills a single literal down a column', () => {
    const sheet = createSheet({ workbook: book({ A1: { value: 7 } }) })
    sheet.fillRange(makeRange({ row: 0, col: 0 }, { row: 0, col: 0 }), makeRange({ row: 0, col: 0 }, { row: 2, col: 0 }), 'copy')
    expect(sheet.getCell('A2')().value).toBe(7)
    expect(sheet.getCell('A3')().value).toBe(7)
  })

  it('series-fills a numeric run by extrapolating the source step', () => {
    const sheet = createSheet({ workbook: book({ A1: { value: 1 }, A2: { value: 2 } }) })
    sheet.fillRange(makeRange({ row: 0, col: 0 }, { row: 1, col: 0 }), makeRange({ row: 0, col: 0 }, { row: 4, col: 0 }), 'series')
    expect(sheet.getCell('A3')().value).toBe(3)
    expect(sheet.getCell('A4')().value).toBe(4)
    expect(sheet.getCell('A5')().value).toBe(5)
  })

  it('re-bases a relative formula across the fill target', () => {
    // B1 = A1*2. Fill B1 down to B3 → B2 = A2*2, B3 = A3*2.
    const sheet = createSheet({
      workbook: book({ A1: { value: 1 }, A2: { value: 2 }, A3: { value: 3 }, B1: { formula: '=A1*2' } }),
    })
    sheet.fillRange(makeRange({ row: 0, col: 1 }, { row: 0, col: 1 }), makeRange({ row: 0, col: 1 }, { row: 2, col: 1 }), 'copy')
    expect(sheet.getCell('B2')().raw.formula).toBe('=A2*2')
    expect(sheet.getCell('B3')().raw.formula).toBe('=A3*2')
    expect(sheet.getCell('B2')().value).toBe(4)
    expect(sheet.getCell('B3')().value).toBe(6)
  })

  it('keeps an absolute reference ($A$1) fixed while re-basing the relative axis', () => {
    const sheet = createSheet({
      workbook: book({ A1: { value: 10 }, B1: { formula: '=$A$1+C1' }, C1: { value: 1 }, C2: { value: 2 } }),
    })
    sheet.fillRange(makeRange({ row: 0, col: 1 }, { row: 0, col: 1 }), makeRange({ row: 0, col: 1 }, { row: 1, col: 1 }), 'copy')
    expect(sheet.getCell('B2')().raw.formula).toBe('=$A$1+C2')
  })
})

describe('clipboard — TSV serialize + relative re-base on paste', () => {
  it('rangeToTsv serializes a range to tab/newline-delimited text (display values)', () => {
    const sheet = createSheet({ workbook: book({ A1: { value: 1 }, B1: { value: 2 }, A2: { value: 3 }, B2: { value: 4 } }) })
    const tsv = rangeToTsv(sheet, makeRange({ row: 0, col: 0 }, { row: 1, col: 1 }))
    expect(tsv).toBe('1\t2\n3\t4')
  })

  it('rangeToTsv emits the formula SOURCE for a formula cell (round-trips the formula)', () => {
    const sheet = createSheet({ workbook: book({ A1: { value: 5 }, B1: { formula: '=A1*2' } }) })
    const tsv = rangeToTsv(sheet, makeRange({ row: 0, col: 1 }, { row: 0, col: 1 }))
    expect(tsv).toBe('=A1*2')
  })

  it('tsvToEdits parses literals into CellEdits at the paste anchor', () => {
    const edits = tsvToEdits('10\t20\n30\t40', { row: 4, col: 4 }) // anchor E5
    expect(edits.map((e) => e.a1)).toEqual(['E5', 'F5', 'E6', 'F6'])
    expect(edits[0]!.after).toEqual({ value: 10 })
    expect(edits[3]!.after).toEqual({ value: 40 })
  })

  it('tsvToEdits re-bases a relative formula by the paste offset', () => {
    // Copied =A1*2 from B1; pasting at B2 (one row down) → =A2*2.
    const edits = tsvToEdits('=A1*2', { row: 1, col: 1 }, { row: 0, col: 1 })
    expect(edits[0]!.a1).toBe('B2')
    expect(edits[0]!.after.formula).toBe('=A2*2')
  })

  it('copy → paste round-trips a literal block to a new anchor', () => {
    const sheet = createSheet({ workbook: book({ A1: { value: 1 }, B1: { value: 2 } }) })
    const tsv = rangeToTsv(sheet, makeRange({ row: 0, col: 0 }, { row: 0, col: 1 }))
    const edits = tsvToEdits(tsv, { row: 2, col: 0 }) // paste at A3
    sheet.setCells(edits)
    expect(sheet.getCell('A3')().value).toBe(1)
    expect(sheet.getCell('B3')().value).toBe(2)
  })
})
