// sui-sheet engine — SSR/declarative workbook decode (the sui-combobox pattern).
//
// `prop:sheet={sheet}` identity binding is the load-bearing path; the engine is
// NEVER serialized. This parser exists ONLY for the declarative/SSR fallback: a
// `<sui-sheet workbook='{…}'>` with no parent controller. The compiler's `<prop>`
// vocabulary is `string | bool | number` (`type="json"`/`object`/`array` do not
// compile), so a structured workbook arrives as a JSON string parsed here.
//
// PURE TypeScript — no runtime/SDK imports.
//
// Authority: build-spec §"Prop wiring (LAYER 2 — JSON-string fallback)", §Props
// (the `workbook` SSR fallback).

import type { CellData, SheetDef, WorkbookDef } from './types'

/** Parse the `workbook` string prop into a `WorkbookDef`, or `null` when the
 *  attribute is empty / malformed (the consumer then renders the blank grid). */
export function parseJsonWorkbook(raw: string): WorkbookDef | null {
  if (!raw) return null
  let parsed: unknown
  try {
    parsed = JSON.parse(raw)
  } catch {
    return null
  }
  if (parsed == null || typeof parsed !== 'object') return null
  const o = parsed as Record<string, unknown>
  if (!Array.isArray(o.sheets)) return null
  const sheets = o.sheets
    .map(coerceSheet)
    .filter((s): s is SheetDef => s !== null)
  if (sheets.length === 0) return null
  const wb: WorkbookDef = { sheets }
  if (typeof o.activeSheetId === 'string') wb.activeSheetId = o.activeSheetId
  return wb
}

function coerceSheet(entry: unknown): SheetDef | null {
  if (entry == null || typeof entry !== 'object') return null
  const o = entry as Record<string, unknown>
  if (typeof o.id !== 'string' || typeof o.name !== 'string') return null
  const cells: Record<string, CellData> = {}
  if (o.cells != null && typeof o.cells === 'object') {
    for (const [a1, raw] of Object.entries(o.cells as Record<string, unknown>)) {
      if (raw != null && typeof raw === 'object') cells[a1] = raw as CellData
    }
  }
  return {
    id: o.id,
    name: o.name,
    rowCount: typeof o.rowCount === 'number' ? o.rowCount : 0,
    colCount: typeof o.colCount === 'number' ? o.colCount : 0,
    cells,
    ...(typeof o.frozenRows === 'number' ? { frozenRows: o.frozenRows } : {}),
    ...(typeof o.frozenCols === 'number' ? { frozenCols: o.frozenCols } : {}),
    ...(typeof o.defaultRowHeight === 'number' ? { defaultRowHeight: o.defaultRowHeight } : {}),
    ...(typeof o.defaultColWidth === 'number' ? { defaultColWidth: o.defaultColWidth } : {}),
  }
}
