// sui-sheet engine — fill-handle series computation (relative-ref series fill).
//
// The fill handle extends a source range over a (usually larger) target range:
//   • a literal source COPIES, or SERIES-extrapolates a numeric run (the source
//     step continues — 1,2 → 3,4,5);
//   • a formula source RE-BASES its relative references across each target cell
//     (=A1*2 filled down → =A2*2, =A3*2), absolute axes ($A$1) staying fixed.
//
// `computeFill` is pure: it reads the source cells via the injected reader and
// returns the `CellEdit`s the assembler writes. The relative-ref rebasing lives
// in `refs.ts` (shared with paste).
//
// PURE TypeScript — `a1.ts` + `refs.ts` only.
//
// Authority: build-spec §Features #5 (fill handle relative-reference series).

import { toA1 } from './a1'
import { rebaseFormula } from './refs'
import type { CellData, CellEdit, CellRange } from './types'

/** Read the authored/override cell data at an A1 (the source-cell reader). */
export type CellReader = (a1: string) => CellData | undefined

/** Compute the fill edits to extend `from` across `to`. The fill axis is the
 *  one `to` grows beyond `from`; cells inside `from` are left untouched. */
export function computeFill(
  from: CellRange,
  to: CellRange,
  mode: 'copy' | 'series',
  read: CellReader,
): CellEdit[] {
  const src = from.normalized()
  const dst = to.normalized()
  const vertical = dst.end.row > src.end.row || dst.start.row < src.start.row
  const srcLen = vertical ? src.end.row - src.start.row + 1 : src.end.col - src.start.col + 1

  // Pre-read the source line(s) keyed by position-within-source so each target
  // cell maps back to its source cell by modulo (the copy pattern repeats).
  const edits: CellEdit[] = []
  for (let r = dst.start.row; r <= dst.end.row; r++) {
    for (let c = dst.start.col; c <= dst.end.col; c++) {
      // Skip the source region itself — fill only writes the EXTENSION.
      if (r >= src.start.row && r <= src.end.row && c >= src.start.col && c <= src.end.col) continue
      const a1 = toA1({ row: r, col: c })
      const after = vertical
        ? fillCellVertical(src, r, c, srcLen, mode, read)
        : fillCellHorizontal(src, r, c, srcLen, mode, read)
      edits.push({ a1, before: {}, after })
    }
  }
  return edits
}

function fillCellVertical(
  src: CellRange,
  r: number,
  c: number,
  srcLen: number,
  mode: 'copy' | 'series',
  read: CellReader,
): CellData {
  const stepsPast = r - src.start.row // distance from the source top
  const srcIdx = ((stepsPast % srcLen) + srcLen) % srcLen
  const srcRow = src.start.row + srcIdx
  const base = read(toA1({ row: srcRow, col: c }))
  return projectCell(base, r - srcRow, 0, stepsPast, srcLen, mode, () =>
    seriesValue(src, c, true, read, stepsPast),
  )
}

function fillCellHorizontal(
  src: CellRange,
  r: number,
  c: number,
  srcLen: number,
  mode: 'copy' | 'series',
  read: CellReader,
): CellData {
  const stepsPast = c - src.start.col
  const srcIdx = ((stepsPast % srcLen) + srcLen) % srcLen
  const srcCol = src.start.col + srcIdx
  const base = read(toA1({ row: r, col: srcCol }))
  return projectCell(base, 0, c - srcCol, stepsPast, srcLen, mode, () =>
    seriesValue(src, r, false, read, stepsPast),
  )
}

/** Map a source cell onto a target offset. A formula re-bases; a numeric series
 *  extrapolates; everything else copies verbatim. */
function projectCell(
  base: CellData | undefined,
  dRow: number,
  dCol: number,
  stepsPast: number,
  srcLen: number,
  mode: 'copy' | 'series',
  series: () => number | null,
): CellData {
  if (base?.formula != null) {
    return { formula: rebaseFormula(base.formula, dRow, dCol) }
  }
  if (mode === 'series') {
    const v = series()
    if (v !== null) return { value: v }
  }
  if (base == null) return {}
  return { ...base }
}

/** Linear extrapolation of a numeric source run. The step is the mean delta of
 *  the source values along the fill axis (a single source cell → step 0 = copy).
 *  Returns null when the source is non-numeric (caller falls back to copy). */
function seriesValue(
  src: CellRange,
  lateral: number,
  vertical: boolean,
  read: CellReader,
  stepsPast: number,
): number | null {
  const vals: number[] = []
  const len = vertical ? src.end.row - src.start.row + 1 : src.end.col - src.start.col + 1
  for (let i = 0; i < len; i++) {
    const a1 = vertical
      ? toA1({ row: src.start.row + i, col: lateral })
      : toA1({ row: lateral, col: src.start.col + i })
    const raw = read(a1)
    const v = raw?.value
    if (typeof v !== 'number') return null
    vals.push(v)
  }
  if (vals.length === 0) return null
  const step =
    vals.length > 1 ? (vals[vals.length - 1]! - vals[0]!) / (vals.length - 1) : 0
  if (step === 0) return null // flat run → copy semantics
  // stepsPast is the distance from the source TOP; the last source value sits at
  // (len-1). Continue from the source's first value along the line.
  return vals[0]! + step * stepsPast
}
