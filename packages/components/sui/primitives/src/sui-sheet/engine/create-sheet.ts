// sui-sheet engine — createSheet() assembler (the headless cell engine).
//
// Wires the derived chain (pipeline.ts) + the interaction slices (selection,
// editing) + geometry into the `Sheet` facade. The reactive cell model lives in
// `state.ts` (one signal per cell → dirty-suffix writes); the read chain in
// `pipeline.ts`; this file is the thin wire-up. Imports `{ batch, derived }` from
// `@ship/sui-runtime` ONLY — no SDK, no network, no DOM (the engine is pure
// reactive state over a cell space; the view renders it via prop:sheet).
//
// SCOPE: the literal cell path, range selection, the editing model, geometry,
// and live-workbook ingestion — plus the formula engine (parse / DAG / eval via
// `formula-engine.ts`), fill-handle series (`fillRange` → `fill.ts`), and
// clipboard re-basing (relative-ref shifting via `refs.ts`, shared by fill +
// `clipboard.ts`). All of these are wired through the facade below; nothing here
// throws a deferred-slice error.
//
// PURE TypeScript. Authority: build-spec §"Backbone — createSheet()", §"API
// sketch", §"Signal graph", §Slices 1/3, create-sheet.test.ts.

import { batch, derived, effect, untrack } from '@ship/sui-runtime'
import { fromA1, toA1 } from './a1'
import { buildEditing } from './editing'
import { computeFill } from './fill'
import { createFastFormulaCore } from './formula/adapter-fast-formula-parser'
import { buildFormulaEngine } from './formula-engine'
import { buildPipeline } from './pipeline'
import { buildSelection } from './selection'
import {
  buildInteractionState,
  emptySelection,
  liftWorkbook,
  makeSelection,
  reflect,
} from './state'
import { makeRange } from './range'
import type { Sheet, SheetOptions } from './contract'
import type {
  CellAddress,
  CellData,
  CellEdit,
  CellRange,
  ResolvedCell,
  ScalarValue,
  SheetDef,
} from './types'

/** Compute the next active address for a `moveActive` direction, clamped to the
 *  logical extent. `tab`/`enter` advance like `right`/`down`. */
function nextAddress(
  cur: CellAddress,
  dir: Parameters<Sheet['moveActive']>[0],
  rows: number,
  cols: number,
): CellAddress {
  const clampRow = (r: number): number => Math.max(0, Math.min(rows - 1, r))
  const clampCol = (c: number): number => Math.max(0, Math.min(cols - 1, c))
  switch (dir) {
    case 'up':
      return { row: clampRow(cur.row - 1), col: cur.col }
    case 'down':
    case 'enter':
      return { row: clampRow(cur.row + 1), col: cur.col }
    case 'left':
      return { row: cur.row, col: clampCol(cur.col - 1) }
    case 'right':
    case 'tab':
      return { row: cur.row, col: clampCol(cur.col + 1) }
    case 'home':
      return { row: cur.row, col: 0 }
    case 'end':
      return { row: cur.row, col: cols - 1 }
    case 'ctrl-home':
      return { row: 0, col: 0 }
    case 'ctrl-end':
      return { row: rows - 1, col: cols - 1 }
  }
}

export function createSheet(options: SheetOptions): Sheet {
  const workbook = liftWorkbook(options.workbook)
  const recalcMode = options.recalcMode ?? 'auto'

  // Active-sheet id slice — seeded from the workbook's active id (or the first
  // sheet). The interaction slices (selection / editing) live alongside it.
  const seedWb = workbook()
  const seedActiveId =
    options.initialState?.activeSheetId ?? seedWb.activeSheetId ?? seedWb.sheets[0]?.id ?? null
  const state = buildInteractionState(seedActiveId, options.initialState?.selection)

  // ── formula engine (DAG × FormulaCore × cell store) ──
  // `readCell` is the eval read-side: a formula precedent resolves to its cached
  // evaluated result (the DAG topo order guarantees it is computed first); a
  // literal precedent resolves to its raw value. Read untracked inside recompute.
  const core = options.formulaCore ?? createFastFormulaCore()
  const formulaEngine = buildFormulaEngine(
    core,
    (a1) => readCellValue(a1),
    recalcMode,
  )

  const pipeline = buildPipeline(workbook, state, formulaEngine)
  const { activeSheetDef, store, cellApi, geometry } = pipeline
  const selection = buildSelection(state, options.onSelectionChange)
  const editing = buildEditing(state)

  function readCellValue(a1: string): ScalarValue {
    const raw = store.read(a1)
    if (raw?.formula != null) return formulaEngine.result(a1)?.value ?? null
    return raw?.value ?? null
  }

  // Re-sync the engine whenever the active sheet's authored formula set or extent
  // changes (init + workbook-identity swap). Local setCell formula edits are
  // folded in by `onCellWrite`; this effect owns the authored baseline.
  effect(() => {
    const def = activeSheetDef()
    const formulas = new Map<string, string>()
    if (def) {
      for (const [a1, data] of Object.entries(def.cells)) {
        if (data.formula != null) formulas.set(a1, data.formula)
      }
    }
    const extent = { rows: def?.rowCount ?? 0, cols: def?.colCount ?? 0 }
    untrack(() => formulaEngine.sync(formulas, extent))
  })

  const emitCells = (edits: CellEdit[]): void => {
    if (edits.length > 0) options.onCellChange?.(edits)
  }

  // ── render surfaces ──
  const activeSheet = derived<SheetDef>(() => activeSheetDef() as SheetDef)
  const rowCount = pipeline.rowCount
  const colCount = pipeline.colCount

  // ── mutators (foundational) ──
  // Every cell write routes through the formula engine: a formula cell re-parses
  // + recomputes its dirty suffix; a literal cell recomputes any formulas that
  // depend on it. (The engine no-ops for a literal write with no dependents.)
  const setCell = (a1: string, data: CellData): void => {
    batch(() => {
      const before = store.read(a1) ?? {}
      store.write(a1, data)
      formulaEngine.onCellWrite(a1, data.formula)
      emitCells([{ a1, before, after: data }])
    })
  }

  const setCells = (edits: CellEdit[]): void => {
    batch(() => {
      for (const e of edits) {
        store.write(e.a1, e.after)
        formulaEngine.onCellWrite(e.a1, e.after.formula)
      }
      emitCells(edits)
    })
  }

  const clearCells = (range: CellRange, what: 'all' | 'values' | 'formats' = 'all'): void => {
    batch(() => {
      const edits: CellEdit[] = []
      for (const addr of range.cells()) {
        const a1 = toA1(addr)
        const before = store.read(a1) ?? {}
        const after: CellData =
          what === 'formats' ? { value: before.value ?? null } : {}
        store.write(a1, after)
        formulaEngine.onCellWrite(a1, after.formula)
        edits.push({ a1, before, after })
      }
      emitCells(edits)
    })
  }

  // ── editing → cell write bridge ──
  const startEditing = (a?: CellAddress, seed?: string): void => {
    editing.startEditing(state.selection().active, a, seed)
  }
  const commitEdit = (opts?: { advance?: 'down' | 'right' | 'none' }): void => {
    const committed = editing.commitEdit(opts)
    if (!committed) return
    setCell(committed.a1, committed.data)
    if (committed.advance !== 'none') {
      const dir = committed.advance === 'right' ? { col: 1, row: 0 } : { col: 0, row: 1 }
      selection.setActiveCell({
        row: committed.cell.row + dir.row,
        col: committed.cell.col + dir.col,
      })
    }
  }

  // ── keyboard nav ──
  // Move the active cell within the logical extent. `extend` grows the active
  // range FROM the anchor (shift-arrow) by delegating to setActiveCell's extend
  // path; otherwise it collapses to a 1×1 selection at the destination.
  const moveActive: Sheet['moveActive'] = (dir, opts) => {
    const rows = rowCount()
    const cols = colCount()
    if (rows === 0 || cols === 0) return
    const cur = selection.activeCell()
    const next = nextAddress(cur, dir, rows, cols)
    selection.setActiveCell(next, opts?.extend ? { extend: true } : undefined)
  }

  // ── fill handle (relative-reference series) ──
  // Extend `from` across `to`: a literal copies / a numeric run series-fills, a
  // formula re-bases its relative refs. The pure computation lives in fill.ts;
  // the writes route through setCells so each fill cell recomputes its suffix.
  const fillRange: Sheet['fillRange'] = (from, to, mode = 'copy') => {
    const edits = computeFill(from, to, mode, (a1) => store.read(a1))
    if (edits.length > 0) setCells(edits)
  }

  // ── structure ──
  const setFrozen = (rows: number, cols: number): void => {
    const def = activeSheetDef()
    if (def) {
      def.frozenRows = rows
      def.frozenCols = cols
    }
  }
  const setColWidth = (col: number, px: number): void => {
    const def = activeSheetDef()
    if (def) (def.colWidths ??= {})[col] = px
  }
  const setRowHeight = (row: number, px: number): void => {
    const def = activeSheetDef()
    if (def) (def.rowHeights ??= {})[row] = px
  }
  const setActiveSheet = (sheetId: string): void => {
    batch(() => {
      state.activeSheetId.set(sheetId)
      // A sheet switch resets selection to the origin of the new sheet.
      state.selection.set(emptySelection())
    })
    options.onActiveSheetChange?.(sheetId)
  }

  const reset = (): void => {
    batch(() => {
      state.selection.set(emptySelection())
      state.editing.set(null)
    })
  }

  return {
    activeSheet,
    rowCount,
    colCount,
    frozenRowCount: geometry.frozenRowCount,
    frozenColCount: geometry.frozenColCount,
    centerRowRange: geometry.centerRowRange,
    centerColRange: geometry.centerColRange,
    getCell: cellApi.getCell,
    getCellAt: cellApi.getCellAt,
    rowHeight: (i) => geometry.rowHeightAt(i),
    colWidth: (i) => geometry.colWidthAt(i),
    totalWidth: geometry.totalWidth,
    totalHeight: geometry.totalHeight,
    selection: selection.selection,
    activeCell: selection.activeCell,
    isInRange: selection.isInRange,
    dependents: (a1) =>
      derived<string[]>(() => {
        formulaEngine.trackGraph()
        return formulaEngine.dependents(a1)
      }),
    precedents: (a1) =>
      derived<string[]>(() => {
        formulaEngine.trackGraph()
        return formulaEngine.precedents(a1)
      }),
    hasCycle: derived<boolean>(() => {
      formulaEngine.trackGraph()
      return formulaEngine.hasCycle()
    }),
    isLoading: reflect(options.isLoading, false),
    error: reflect(options.error, null),
    editing: derived<{ cell: CellAddress; draft: string } | null>(() => state.editing()),
    setActiveCell: selection.setActiveCell,
    setSelection: selection.setSelection,
    moveActive,
    startEditing,
    updateDraft: editing.updateDraft,
    commitEdit,
    cancelEdit: editing.cancelEdit,
    setCell,
    setCells,
    clearCells,
    fillRange,
    setFormat: (range, fmt) => applyToRange(range, (c) => ({ ...c, format: { ...c.format, ...fmt } })),
    setStyle: (range, style) =>
      applyToRange(range, (c) => ({
        ...c,
        format: { ...c.format, style: { ...c.format?.style, ...style } },
      })),
    setFrozen,
    setColWidth,
    setRowHeight,
    setActiveSheet,
    recalc: () => batch(() => formulaEngine.recalcAll()),
    reset,
  }

  /** Apply a per-cell transform over a range, emitting one batched onCellChange. */
  function applyToRange(range: CellRange, fn: (c: CellData) => CellData): void {
    batch(() => {
      const edits: CellEdit[] = []
      for (const addr of range.cells()) {
        const a1 = toA1(addr)
        const before = store.read(a1) ?? {}
        const after = fn(before)
        store.write(a1, after)
        edits.push({ a1, before, after })
      }
      emitCells(edits)
    })
  }
}

// Re-exports so consumers import the engine + its types from one module.
export type { Sheet, SheetOptions } from './contract'
export { fromA1, toA1, makeRange, makeSelection }
export type { ResolvedCell }
