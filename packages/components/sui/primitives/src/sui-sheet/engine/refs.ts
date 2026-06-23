// sui-sheet engine — relative-reference rebasing (the fill + paste primitive).
//
// Fill-handle series and clipboard paste both need to SHIFT a formula's cell
// references by a (Δrow, Δcol) offset while leaving absolute (`$A$1`) axes fixed
// — the spreadsheet "relative reference" rule. This is the ONE place that
// transform lives; both `fill.ts` and `clipboard.ts` call `rebaseFormula`.
//
// The scan reuses the `a1.ts` coordinate algebra: it walks the formula string,
// finds each A1 token (optionally cross-sheet prefixed), parses it, shifts the
// non-absolute axes, and re-emits via `toA1`. Tokens inside a string literal
// (`"…"`) are left untouched.
//
// PURE TypeScript — `a1.ts` only.
//
// Authority: build-spec §Features #5 (fill series re-bases relative refs) + #6
// (paste re-bases relative refs); CellAddress.absRow/absCol markers.

import { fromA1, toA1 } from './a1'

/** A1 cell-reference token: optional `$`s, letters, `$`, digits. Sheet-qualified
 *  refs (`Sheet2!A1`) keep the prefix verbatim — only the cell body shifts. */
const REF_TOKEN = /(\$?[A-Za-z]{1,3}\$?\d{1,7})/g

/** Shift every RELATIVE cell reference in `formula` by (dRow, dCol); absolute
 *  axes (the `$`-marked ones) stay put. Returns the rebased formula string
 *  (leading `=` preserved). Out-of-bounds shifts clamp at 0 (no negative A1). */
export function rebaseFormula(formula: string, dRow: number, dCol: number): string {
  if (dRow === 0 && dCol === 0) return formula
  return splitOutsideStrings(formula)
    .map((seg) => (seg.isString ? seg.text : shiftRefsInSegment(seg.text, dRow, dCol)))
    .join('')
}

function shiftRefsInSegment(text: string, dRow: number, dCol: number): string {
  return text.replace(REF_TOKEN, (token) => {
    let addr: ReturnType<typeof fromA1>
    try {
      addr = fromA1(token)
    } catch {
      return token // not a real cell ref (e.g. a function name fragment)
    }
    const nextRow = addr.absRow ? addr.row : Math.max(0, addr.row + dRow)
    const nextCol = addr.absCol ? addr.col : Math.max(0, addr.col + dCol)
    return toA1({
      row: nextRow,
      col: nextCol,
      ...(addr.absRow !== undefined ? { absRow: addr.absRow } : {}),
      ...(addr.absCol !== undefined ? { absCol: addr.absCol } : {}),
    })
  })
}

interface Segment {
  text: string
  isString: boolean
}

/** Split a formula into alternating segments so the ref scan never touches text
 *  inside a `"…"` string literal. Each emitted segment carries the text plus a
 *  flag for whether it is inside (or is the delimiter of) a string literal —
 *  string segments include their surrounding quotes so a join() round-trips. */
function splitOutsideStrings(formula: string): Segment[] {
  const out: Segment[] = []
  let buf = ''
  let inString = false
  const flush = (): void => {
    if (buf !== '') out.push({ text: buf, isString: inString })
    buf = ''
  }
  for (let i = 0; i < formula.length; i++) {
    const ch = formula[i]!
    if (ch !== '"') {
      buf += ch
      continue
    }
    if (!inString) {
      // Opening a string: flush the code segment, then start the string buffer.
      flush()
      inString = true
      buf = '"'
      continue
    }
    // A doubled quote ("") inside a string is an escaped quote — keep both.
    if (formula[i + 1] === '"') {
      buf += '""'
      i++
      continue
    }
    // Closing the string: include the closing quote, flush, back to code.
    buf += '"'
    flush()
    inString = false
  }
  flush()
  return out
}
