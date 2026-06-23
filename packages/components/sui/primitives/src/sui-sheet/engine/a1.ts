// sui-sheet engine — A1 coordinate algebra (Slice 1, the coordinate FLOOR).
//
// The ONE place the `(row,col) ↔ A1` transform lives. The DAG, selection, fill,
// and clipboard modules all ask THIS module — there is no second coordinate
// system. Owns: column-label base-26 bijection, A1 parse/emit (with absolute/
// relative `$` markers preserved for fill-series), range parsing (`A1:C3`,
// whole-column `A:A`, whole-row `1:1`), and cross-sheet (`Sheet2!A1`) splitting.
//
// PURE TypeScript — zero runtime/SDK imports (no reactivity needed here).
//
// Authority: build-spec §Features #1 ("A1 coordinate algebra"), §"Authoritative
// Cell / Selection / Range / DAG API" (CellAddress.toA1 / fromA1), Slice 1.

import type { CellAddress } from './types'

/** The full row extent used when a whole-column range (`A:A`) is parsed without
 *  a bound sheet. A consumer with a concrete `rowCount` clamps to its extent. */
const MAX_ROW_INDEX = 1_048_575 // Excel's 1,048,576-row ceiling, 0-based.
/** The full column extent for a whole-row range (`1:1`). */
const MAX_COL_INDEX = 16_383 // Excel's XFD (16,384-column) ceiling, 0-based.

/** Column index → bijective base-26 label. `0 → 'A'`, `25 → 'Z'`, `26 → 'AA'`,
 *  `701 → 'ZZ'`, `702 → 'AAA'`. */
export function colToLabel(col: number): string {
  if (!Number.isInteger(col) || col < 0) {
    throw new Error(`a1: column index must be a non-negative integer, got ${col}`)
  }
  let n = col
  let label = ''
  // Bijective base-26: there is no zero digit, so each step subtracts one.
  do {
    label = String.fromCharCode(65 + (n % 26)) + label
    n = Math.floor(n / 26) - 1
  } while (n >= 0)
  return label
}

/** Bijective base-26 label → column index. Case-insensitive. Inverse of
 *  `colToLabel`. `'A' → 0`, `'AA' → 26`, `'ZZ' → 701`. */
export function labelToCol(label: string): number {
  if (!/^[A-Za-z]+$/.test(label)) {
    throw new Error(`a1: column label must be letters only, got ${JSON.stringify(label)}`)
  }
  let col = 0
  for (const ch of label.toUpperCase()) {
    col = col * 26 + (ch.charCodeAt(0) - 64) // 'A' (65) → 1 in the 1-based digit space
  }
  return col - 1 // back to 0-based
}

const CELL_RE = /^(\$?)([A-Za-z]+)(\$?)([0-9]+)$/

/** Parse an A1 cell reference → a 0-based `CellAddress`, preserving the `$`
 *  absolute markers for fill-series adjustment. Throws (naming the input) on a
 *  malformed address: digits-before-letters (`'1A'`), a bare column (`'AA'`),
 *  empty, or a 0 row (rows are 1-based externally). */
export function fromA1(a1: string): CellAddress {
  const m = CELL_RE.exec(a1)
  if (!m) {
    throw new Error(`a1: malformed cell reference ${JSON.stringify(a1)}`)
  }
  const [, absColMark, letters, absRowMark, digits] = m as unknown as [
    string,
    string,
    string,
    string,
    string,
  ]
  const rowOneBased = Number(digits)
  if (rowOneBased < 1) {
    throw new Error(`a1: row is 1-based — ${JSON.stringify(a1)} has an invalid row`)
  }
  const addr: CellAddress = { row: rowOneBased - 1, col: labelToCol(letters) }
  // When either `$` marker is present the reference is partially/fully absolute;
  // surface BOTH axes explicitly (the absent one as `false`) so fill-series can
  // read a definite flag per axis. A plain `A1` carries neither (both undefined).
  if (absColMark === '$' || absRowMark === '$') {
    addr.absCol = absColMark === '$'
    addr.absRow = absRowMark === '$'
  }
  return addr
}

/** Emit an A1 cell reference from a 0-based `CellAddress`, re-emitting the `$`
 *  markers it carries (`{row:0,col:0,absRow:true,absCol:true} → '$A$1'`). */
export function toA1(addr: CellAddress): string {
  const colPart = `${addr.absCol ? '$' : ''}${colToLabel(addr.col)}`
  const rowPart = `${addr.absRow ? '$' : ''}${addr.row + 1}`
  return `${colPart}${rowPart}`
}

/** A parsed range: both corners (0-based), the cross-sheet name when present,
 *  and a `kind` discriminator (`cell` | `range` | `col` | `row`). */
export interface ParsedRange {
  start: CellAddress
  end: CellAddress
  kind: 'cell' | 'range' | 'col' | 'row'
  sheet?: string
}

const COL_ONLY_RE = /^\$?([A-Za-z]+)$/
const ROW_ONLY_RE = /^\$?([0-9]+)$/

/** Parse a range string into both corners + a `kind`. Handles single cells
 *  (`'B2'` → 1×1), `'A1:C3'`, whole-column `'A:A'`, whole-row `'1:1'`, and the
 *  cross-sheet prefix (`'Sheet2!A1'`, `'Sheet2!A1:B3'`). */
export function parseRange(ref: string): ParsedRange {
  let sheet: string | undefined
  let body = ref
  const bang = ref.indexOf('!')
  if (bang >= 0) {
    sheet = stripSheetQuotes(ref.slice(0, bang))
    body = ref.slice(bang + 1)
  }

  const colon = body.indexOf(':')
  if (colon < 0) {
    const start = fromA1(body)
    return withSheet({ start, end: { ...start }, kind: 'cell' }, sheet)
  }

  const left = body.slice(0, colon)
  const right = body.slice(colon + 1)

  // Whole-column: `A:A` / `A:C`.
  const lCol = COL_ONLY_RE.exec(left)
  const rCol = COL_ONLY_RE.exec(right)
  if (lCol && rCol) {
    const c0 = labelToCol(lCol[1]!)
    const c1 = labelToCol(rCol[1]!)
    return withSheet(
      {
        start: { row: 0, col: Math.min(c0, c1) },
        end: { row: MAX_ROW_INDEX, col: Math.max(c0, c1) },
        kind: 'col',
      },
      sheet,
    )
  }

  // Whole-row: `1:1` / `1:5`.
  const lRow = ROW_ONLY_RE.exec(left)
  const rRow = ROW_ONLY_RE.exec(right)
  if (lRow && rRow) {
    const r0 = Number(lRow[1]) - 1
    const r1 = Number(rRow[1]) - 1
    if (r0 < 0 || r1 < 0) {
      throw new Error(`a1: row is 1-based — ${JSON.stringify(ref)} has an invalid row`)
    }
    return withSheet(
      {
        start: { row: Math.min(r0, r1), col: 0 },
        end: { row: Math.max(r0, r1), col: MAX_COL_INDEX },
        kind: 'row',
      },
      sheet,
    )
  }

  // A bounded cell range: `A1:C3`.
  const start = fromA1(left)
  const end = fromA1(right)
  return withSheet({ start, end, kind: 'range' }, sheet)
}

function withSheet(r: ParsedRange, sheet: string | undefined): ParsedRange {
  return sheet !== undefined ? { ...r, sheet } : r
}

/** A sheet name in a cross-sheet ref may be quoted (`'My Sheet'!A1`). Strip the
 *  surrounding single quotes and unescape doubled inner quotes. */
function stripSheetQuotes(name: string): string {
  if (name.length >= 2 && name.startsWith("'") && name.endsWith("'")) {
    return name.slice(1, -1).replace(/''/g, "'")
  }
  return name
}
