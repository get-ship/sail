// sui-sheet engine — grid geometry (spacer sizes + frozen bands + center spans).
//
// The windowed-render geometry the two virtualizers bind to: per-index row/col
// sizes (per-index overrides over the sheet defaults), `totalWidth`/`totalHeight`
// over the LOGICAL extent (both scrollbars honest over a sparse grid), the
// frozen-band counts, and the `centerRowRange`/`centerColRange` count-source
// spans — which START after the frozen bands (the virtualizer never counts a
// frozen row/col; they render as sibling sticky quadrants OUTSIDE the window).
//
// PURE TypeScript — `derived` from `@ship/sui-runtime` ONLY.
//
// Authority: build-spec §Features #7 (frozen quadrants) + #9 (infinite grid;
// totalWidth/totalHeight), §"Signal graph" (centerRowRange/centerColRange),
// virtual-grid.test.ts.

import { derived, type Derived } from '@ship/sui-runtime'
import type { SheetDef } from './types'

const DEFAULT_ROW_HEIGHT = 24
const DEFAULT_COL_WIDTH = 80

/** Per-index row height: an override wins, else the sheet default, else 24px. */
export function rowHeightOf(def: SheetDef | undefined, index: number): number {
  return def?.rowHeights?.[index] ?? def?.defaultRowHeight ?? DEFAULT_ROW_HEIGHT
}

/** Per-index column width: an override wins, else the sheet default, else 80px. */
export function colWidthOf(def: SheetDef | undefined, index: number): number {
  return def?.colWidths?.[index] ?? def?.defaultColWidth ?? DEFAULT_COL_WIDTH
}

export interface GridGeometry {
  frozenRowCount: Derived<number>
  frozenColCount: Derived<number>
  totalWidth: Derived<number>
  totalHeight: Derived<number>
  centerRowRange: Derived<{ start: number; end: number }>
  centerColRange: Derived<{ start: number; end: number }>
  /** Per-index size accessors (the virtualizer's `estimateSize` source). */
  rowHeightAt(index: number): number
  colWidthAt(index: number): number
}

/** Build the reactive geometry surfaces over the active sheet. `centerRowRange`/
 *  `centerColRange` span the FULL logical extent past the frozen bands — the
 *  virtualizer then windows a bounded slice of that span; the engine never
 *  windows itself (it owns the count source, not the DOM). */
export function buildGeometry(activeSheet: Derived<SheetDef | undefined>): GridGeometry {
  const frozenRowCount = derived<number>(() => activeSheet()?.frozenRows ?? 0)
  const frozenColCount = derived<number>(() => activeSheet()?.frozenCols ?? 0)

  const totalHeight = derived<number>(() => {
    const def = activeSheet()
    if (!def) return 0
    let sum = 0
    for (let r = 0; r < def.rowCount; r++) sum += rowHeightOf(def, r)
    return sum
  })

  const totalWidth = derived<number>(() => {
    const def = activeSheet()
    if (!def) return 0
    let sum = 0
    for (let c = 0; c < def.colCount; c++) sum += colWidthOf(def, c)
    return sum
  })

  const centerRowRange = derived<{ start: number; end: number }>(() => {
    const def = activeSheet()
    if (!def) return { start: 0, end: 0 }
    return { start: def.frozenRows ?? 0, end: def.rowCount }
  })

  const centerColRange = derived<{ start: number; end: number }>(() => {
    const def = activeSheet()
    if (!def) return { start: 0, end: 0 }
    return { start: def.frozenCols ?? 0, end: def.colCount }
  })

  const rowHeightAt = (index: number): number => rowHeightOf(activeSheet(), index)
  const colWidthAt = (index: number): number => colWidthOf(activeSheet(), index)

  return {
    frozenRowCount,
    frozenColCount,
    totalWidth,
    totalHeight,
    centerRowRange,
    centerColRange,
    rowHeightAt,
    colWidthAt,
  }
}
