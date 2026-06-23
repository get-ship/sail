// sui-sheet engine — the derived-chain wiring (split from the assembler).
//
// The one-direction derived chain the assembler composes: workbook → activeSheet
// (select by active id) → the cell store + cell-api + geometry + selection. Held
// in its own module from the START (not after the assembler crosses the cap) —
// Table's `create-table.ts` hit 357 lines for lack of this split, so `sui-sheet`
// pushes the chain here so the assembler stays a thin wire-up.
//
// PURE TypeScript — `derived` from `@ship/sui-runtime` ONLY.
//
// Authority: build-spec §"Signal graph" (the derived chain), §Slices (Slice 3:
// pipeline split out from the start).

import { derived, type Derived } from '@ship/sui-runtime'
import { buildCellApi, type CellApi, type FormulaResolver } from './cell-api'
import { buildGeometry, type GridGeometry } from './geometry'
import { buildCellStore, type CellStore, type InteractionState } from './state'
import type { SheetDef, WorkbookDef } from './types'

export interface SheetPipeline {
  workbook: Derived<WorkbookDef>
  activeSheetDef: Derived<SheetDef | undefined>
  rowCount: Derived<number>
  colCount: Derived<number>
  store: CellStore
  cellApi: CellApi
  geometry: GridGeometry
}

/** Build the read-side derived chain over the lifted workbook + the active-sheet
 *  id slice. `activeSheetDef` selects the rendered worksheet (the active id, or
 *  the workbook's `activeSheetId`, or the first sheet). The optional
 *  `formula` resolver swaps in evaluated results for formula cells. */
export function buildPipeline(
  workbook: Derived<WorkbookDef>,
  state: InteractionState,
  formula?: FormulaResolver,
): SheetPipeline {
  const activeSheetDef = derived<SheetDef | undefined>(() => {
    const wb = workbook()
    const id = state.activeSheetId() ?? wb.activeSheetId
    if (wb.sheets.length === 0) return undefined
    return wb.sheets.find((s) => s.id === id) ?? wb.sheets[0]
  })

  const rowCount = derived<number>(() => activeSheetDef()?.rowCount ?? 0)
  const colCount = derived<number>(() => activeSheetDef()?.colCount ?? 0)

  const store = buildCellStore(activeSheetDef)
  const cellApi = buildCellApi(store, state, activeSheetDef, formula)
  const geometry = buildGeometry(activeSheetDef)

  return { workbook, activeSheetDef, rowCount, colCount, store, cellApi, geometry }
}
