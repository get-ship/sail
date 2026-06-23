// sui-sheet engine — clipboard TSV serialize + parse (copy / paste).
//
// Copy projects a selected range to tab/newline-delimited text (TSV — the
// native spreadsheet clipboard format every host app round-trips); a formula
// cell emits its SOURCE (`=A1*2`) so a paste re-bases it, not its evaluated
// value. Paste parses TSV back into `CellEdit`s anchored at the paste target,
// re-basing each relative formula reference by the (anchor − sourceAnchor)
// offset (the same rule as the fill handle — `refs.ts`).
//
// PURE TypeScript — `a1.ts` + `refs.ts` + `editing.ts` (draft → CellData) only.
//
// Authority: build-spec §Features #6 (clipboard TSV + relative re-base).

import { toA1 } from './a1'
import { draftToCellData } from './editing'
import { rebaseFormula } from './refs'
import type { Sheet } from './contract'
import type { CellAddress, CellEdit, CellRange } from './types'

/** Serialize a range to TSV. A formula cell emits its `formula` source so paste
 *  can re-base it; a literal cell emits its display string. Empty cells emit ''. */
export function rangeToTsv(sheet: Sheet, range: CellRange): string {
  const n = range.normalized()
  const rows: string[] = []
  for (let r = n.start.row; r <= n.end.row; r++) {
    const cols: string[] = []
    for (let c = n.start.col; c <= n.end.col; c++) {
      const resolved = sheet.getCell(toA1({ row: r, col: c }))()
      cols.push(resolved.raw.formula ?? resolved.display)
    }
    rows.push(cols.join('\t'))
  }
  return rows.join('\n')
}

/** Parse TSV into `CellEdit`s placed at `anchor` (top-left of the paste target).
 *  When `sourceAnchor` is given, a relative formula is re-based by the
 *  (anchor − sourceAnchor) offset — the copy-then-paste relative-ref rule. */
export function tsvToEdits(
  tsv: string,
  anchor: CellAddress,
  sourceAnchor?: CellAddress,
): CellEdit[] {
  const dRow = sourceAnchor ? anchor.row - sourceAnchor.row : 0
  const dCol = sourceAnchor ? anchor.col - sourceAnchor.col : 0
  const edits: CellEdit[] = []
  const lines = tsv.replace(/\r\n?/g, '\n').split('\n')
  for (let r = 0; r < lines.length; r++) {
    const cells = lines[r]!.split('\t')
    for (let c = 0; c < cells.length; c++) {
      const a1 = toA1({ row: anchor.row + r, col: anchor.col + c })
      const after = cellFromText(cells[c]!, dRow, dCol)
      edits.push({ a1, before: {}, after })
    }
  }
  return edits
}

/** Map one TSV cell to `CellData`, re-basing a formula by the paste offset. */
function cellFromText(text: string, dRow: number, dCol: number): CellEdit['after'] {
  if (text.startsWith('=')) {
    return { formula: rebaseFormula(text, dRow, dCol) }
  }
  return draftToCellData(text)
}
