// TESTS-FIRST — sui-sheet FOUNDATIONAL slice: the public contract compiles.
//
// Status: RED. `./contract` does not exist yet (greenfield pack). The
// type-only imports below are import-not-found until the foundational slice
// ships `engine/contract.ts`. DO NOT add the impl here. DO NOT weaken into stubs.
//
// Authority: build-spec §"API sketch" (the `Sheet` facade + `SheetOptions` +
// the CellType / CellFormat / CellStyle / CellData / SheetDef / WorkbookDef
// shapes), §"id per sheet + stable A1 are REQUIRED", §"Authoritative Cell /
// Selection / Range / DAG API". Slice 1: "the … shapes; the `Sheet` facade type
// compiles."
//
// This file is a TYPE-LEVEL contract: if the shapes drift from the spec, the
// type assertions stop compiling (tsc/tsgo is the gate). The runtime `it()` is a
// trivial truthiness check so vitest reports the file; the real assertion is
// that the module type-checks against the declared shapes.
//
// NOTE — `CellType: 'formula'` and the formula-bearing CellData stay in the
// type surface, but the formula ENGINE (parse/DAG/eval) is DEFERRED to a
// follow-up slice. The type compiling here does not imply the eval path exists.

import { describe, expect, it } from 'vitest'
import type {
  CellType,
  CellFormat,
  CellStyle,
  CellData,
  CellAddress,
  CellEdit,
  ResolvedCell,
  SheetDef,
  WorkbookDef,
  SheetOptions,
  Sheet,
} from './contract'

describe('contract — the declared shapes type-check (compile-time gate)', () => {
  it('CellType is the closed union the spec declares', () => {
    const types: CellType[] = [
      'auto',
      'text',
      'number',
      'boolean',
      'date',
      'datetime',
      'currency',
      'percent',
      'formula',
    ]
    expect(types.length).toBe(9)
  })

  it('CellData carries an optional literal value + optional formula + format', () => {
    const literal: CellData = { value: 5 }
    const formula: CellData = { formula: '=A1+A2' }
    const formatted: CellData = {
      value: 0.12,
      format: { type: 'percent', numberFormat: '0.0%', align: 'end' },
    }
    expect(literal.value).toBe(5)
    expect(formula.formula).toBe('=A1+A2')
    expect(formatted.format!.type).toBe('percent')
  })

  it('CellStyle carries token-SLOT names only — never raw color', () => {
    const style: CellStyle = {
      bold: true,
      italic: false,
      tone: 'destructive',
      bg: 'surface-2',
      borders: { top: true, bottom: true },
    }
    // The type permits only the named tone-ramp / surface slots — a raw hex
    // would not type-check (it is not in the union).
    expect(style.tone).toBe('destructive')
    expect(style.bg).toBe('surface-2')
  })

  it('SheetDef REQUIRES a stable id + name + extent + sparse cell map', () => {
    const def: SheetDef = {
      id: 's1',
      name: 'Sheet1',
      rowCount: 100,
      colCount: 26,
      cells: { A1: { value: 1 } },
      frozenRows: 1,
      frozenCols: 1,
    }
    expect(def.id).toBe('s1')
    expect(def.cells.A1!.value).toBe(1)
  })

  it('WorkbookDef is a list of sheets + an optional active id', () => {
    const wb: WorkbookDef = {
      sheets: [{ id: 's1', name: 'Sheet1', rowCount: 1, colCount: 1, cells: {} }],
      activeSheetId: 's1',
    }
    expect(wb.sheets.length).toBe(1)
  })

  it('CellAddress is 0-based with a toA1 projection; CellEdit is before/after', () => {
    const addr: CellAddress = { row: 0, col: 0 }
    const edit: CellEdit = { a1: 'A1', before: { value: 1 }, after: { value: 2 } }
    expect(addr.row).toBe(0)
    expect(edit.a1).toBe('A1')
  })

  it('SheetOptions accepts the workbook + the consumer callback seam', () => {
    const opts: SheetOptions = {
      workbook: { sheets: [] },
      recalcMode: 'auto',
      onCellChange: () => {},
      onSelectionChange: () => {},
    }
    expect(opts.recalcMode).toBe('auto')
  })

  it('Sheet facade exposes the render-surface + read + mutator members (type-only)', () => {
    // A function that only references the members proves they exist on the type.
    const probe = (s: Sheet): void => {
      s.activeSheet()
      s.rowCount()
      s.colCount()
      s.frozenRowCount()
      s.frozenColCount()
      s.centerRowRange()
      s.centerColRange()
      s.getCell('A1')()
      s.getCellAt(0, 0)()
      s.totalWidth()
      s.totalHeight()
      s.selection()
      s.activeCell()
      s.isInRange({ row: 0, col: 0 })()
      s.setActiveCell({ row: 0, col: 0 })
      s.setSelection([])
      s.setCell('A1', { value: 1 })
      s.setCells([])
    }
    expect(typeof probe).toBe('function')
  })

  it('ResolvedCell carries the resolved render contract the view reads', () => {
    const probe = (c: ResolvedCell): void => {
      const _: [string, CellAddress, unknown, string, boolean, boolean] = [
        c.a1,
        c.address,
        c.value,
        c.display,
        c.isActive,
        c.isInSelection,
      ]
      void _
    }
    expect(typeof probe).toBe('function')
  })
})
