// sui-sheet view — pixel placement for the editor overlay + fill handle.
//
// The windowed cells carry their own pixel geometry (from the virtualizer), but
// the editor overlay and the fill handle may sit at a cell that is the active
// one regardless of the window slice — so they compute absolute placement
// directly from the engine's per-index sizes (the spacer is positioned from
// row/col 0, so a prefix sum IS the absolute offset).
//
// PURE — reads the engine's `rowHeight`/`colWidth`; no DOM, no reactivity here
// (the controller wraps these in `derived`).

import type { Sheet } from '../engine/create-sheet'
import type { CellAddress } from '../engine/types'

export interface CellBox {
  top: number
  left: number
  width: number
  height: number
}

/** Absolute pixel box of a (row,col) cell within the spacer. */
export function cellBox(sheet: Sheet, addr: CellAddress): CellBox {
  let top = 0
  for (let r = 0; r < addr.row; r++) top += sheet.rowHeight(r)
  let left = 0
  for (let c = 0; c < addr.col; c++) left += sheet.colWidth(c)
  return { top, left, width: sheet.colWidth(addr.col), height: sheet.rowHeight(addr.row) }
}

/** Bottom-right corner point of a range (the fill-handle anchor). */
export function rangeCorner(sheet: Sheet, addr: CellAddress): { top: number; left: number } {
  const box = cellBox(sheet, addr)
  return { top: box.top + box.height, left: box.left + box.width }
}
