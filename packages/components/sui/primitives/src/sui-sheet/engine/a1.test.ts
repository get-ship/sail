// TESTS-FIRST — sui-sheet FOUNDATIONAL slice (Slice 1): A1 coordinate algebra.
//
// Status: RED. `./a1` does not exist yet (pack is greenfield — there is no
// `sui-sheet/` on disk). The import below is import-not-found until the
// foundational slice ships `engine/a1.ts`. That import failure is the expected
// red for this contract. DO NOT add the impl here. DO NOT weaken these into
// passing stubs.
//
// Authority: build-spec §Features #1 ("A1 coordinate algebra — one module owns
// `(row,col)↔A1`, range parsing (`A1:C3`, `A:A`, `1:1`), absolute/relative
// markers (`$A$1`), cross-sheet (`Sheet2!A1`)"), §"Authoritative Cell /
// Selection / Range / DAG API" (CellAddress.toA1 / fromA1, CellRange members),
// Slice 1 ("`(row,col)↔A1` round-trips (`{0,0}→'A1'`, `{1,27}→'AB2'`); range
// parse … absolute/relative markers (`$A$1` parsed, marker preserved for fill);
// a malformed A1 (`'1A'`, `'AA'` with no row) is rejected with a clear error").
//
// The A1 algebra is the coordinate FLOOR — the only place the (row,col)↔A1
// transform lives; the DAG/selection/fill modules all ask this one. No DAG, no
// formula eval here (the formula engine is DEFERRED to a follow-up slice).

import { describe, expect, it } from 'vitest'
// RED: greenfield module — resolves only once the foundational slice ships.
import { toA1, fromA1, parseRange, colToLabel, labelToCol } from './a1'

describe('a1 — (row,col) ↔ A1 round-trips (0-based internal)', () => {
  it("{row:0, col:0} → 'A1'", () => {
    expect(toA1({ row: 0, col: 0 })).toBe('A1')
  })

  it("{row:1, col:27} → 'AB2' (col 27 is AB, row 1 is the 2nd row)", () => {
    expect(toA1({ row: 1, col: 27 })).toBe('AB2')
  })

  it("{row:0, col:25} → 'Z1' (single-letter ceiling)", () => {
    expect(toA1({ row: 0, col: 25 })).toBe('Z1')
  })

  it("{row:0, col:26} → 'AA1' (first two-letter column)", () => {
    expect(toA1({ row: 0, col: 26 })).toBe('AA1')
  })

  it("fromA1('A1') → {row:0, col:0}", () => {
    expect(fromA1('A1')).toEqual({ row: 0, col: 0 })
  })

  it("fromA1('AB2') → {row:1, col:27}", () => {
    expect(fromA1('AB2')).toEqual({ row: 1, col: 27 })
  })

  it('round-trips a spread of addresses', () => {
    for (const a of [
      { row: 0, col: 0 },
      { row: 9, col: 4 },
      { row: 99, col: 51 },
      { row: 1023, col: 701 }, // 701 → 'ZZ'
    ]) {
      expect(fromA1(toA1(a))).toEqual(a)
    }
  })
})

describe('a1 — column label algebra (bijective base-26)', () => {
  it('colToLabel: 0→A, 25→Z, 26→AA, 701→ZZ, 702→AAA', () => {
    expect(colToLabel(0)).toBe('A')
    expect(colToLabel(25)).toBe('Z')
    expect(colToLabel(26)).toBe('AA')
    expect(colToLabel(701)).toBe('ZZ')
    expect(colToLabel(702)).toBe('AAA')
  })

  it('labelToCol inverts colToLabel', () => {
    for (const c of [0, 25, 26, 51, 701, 702]) {
      expect(labelToCol(colToLabel(c))).toBe(c)
    }
  })

  it('labelToCol is case-insensitive', () => {
    expect(labelToCol('ab')).toBe(labelToCol('AB'))
  })
})

describe('a1 — range parsing', () => {
  it("'A1:C3' → start {0,0}, end {2,2}", () => {
    const r = parseRange('A1:C3')
    expect(r.start).toEqual({ row: 0, col: 0 })
    expect(r.end).toEqual({ row: 2, col: 2 })
  })

  it("whole-column 'A:A' spans the full row extent of column 0", () => {
    const r = parseRange('A:A')
    expect(r.start.col).toBe(0)
    expect(r.end.col).toBe(0)
    expect(r.kind).toBe('col')
  })

  it("whole-row '1:1' spans the full column extent of row 0", () => {
    const r = parseRange('1:1')
    expect(r.start.row).toBe(0)
    expect(r.end.row).toBe(0)
    expect(r.kind).toBe('row')
  })

  it("a single cell 'B2' parses as a 1×1 range", () => {
    const r = parseRange('B2')
    expect(r.start).toEqual({ row: 1, col: 1 })
    expect(r.end).toEqual({ row: 1, col: 1 })
  })

  it("cross-sheet 'Sheet2!A1' surfaces the sheet name", () => {
    const r = parseRange('Sheet2!A1')
    expect(r.sheet).toBe('Sheet2')
    expect(r.start).toEqual({ row: 0, col: 0 })
  })

  it("cross-sheet range 'Sheet2!A1:B3' surfaces sheet + both corners", () => {
    const r = parseRange('Sheet2!A1:B3')
    expect(r.sheet).toBe('Sheet2')
    expect(r.start).toEqual({ row: 0, col: 0 })
    expect(r.end).toEqual({ row: 2, col: 1 })
  })
})

describe('a1 — absolute / relative markers (preserved for fill-series)', () => {
  it("'$A$1' parses to {0,0} with both axes marked absolute", () => {
    const a = fromA1('$A$1')
    expect(a).toMatchObject({ row: 0, col: 0 })
    expect(a.absCol).toBe(true)
    expect(a.absRow).toBe(true)
  })

  it("'$A1' marks the column absolute, the row relative", () => {
    const a = fromA1('$A1')
    expect(a.absCol).toBe(true)
    expect(a.absRow).toBe(false)
  })

  it("'A$1' marks the row absolute, the column relative", () => {
    const a = fromA1('A$1')
    expect(a.absCol).toBe(false)
    expect(a.absRow).toBe(true)
  })

  it("a plain 'A1' carries no absolute markers", () => {
    const a = fromA1('A1')
    expect(a.absCol).toBeFalsy()
    expect(a.absRow).toBeFalsy()
  })

  it('toA1 re-emits the $ markers it was given', () => {
    expect(toA1({ row: 0, col: 0, absRow: true, absCol: true })).toBe('$A$1')
    expect(toA1({ row: 0, col: 0, absCol: true })).toBe('$A1')
  })
})

describe('a1 — malformed addresses are rejected with a clear error', () => {
  it("'1A' (digits before letters) throws", () => {
    expect(() => fromA1('1A')).toThrow()
  })

  it("'AA' (no row number) throws", () => {
    expect(() => fromA1('AA')).toThrow()
  })

  it("'' (empty) throws", () => {
    expect(() => fromA1('')).toThrow()
  })

  it("'A0' (rows are 1-based externally; 0 is invalid) throws", () => {
    expect(() => fromA1('A0')).toThrow()
  })

  it('the thrown error names the offending input', () => {
    expect(() => fromA1('1A')).toThrow(/1A/)
  })
})
