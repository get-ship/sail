// sui-sheet engine — the createSheet() public contract (Backbone, NEW shape).
//
// `SheetOptions` (the input) + `Sheet` (the facade the assembler returns). Split
// from `types.ts` (the shared cell/format/style/def shapes) so each file stays
// under the cap; this module re-exports the shared shapes so one import covers
// both. The engine binds to the `FormulaCore` interface for formulas (the vendor
// seam) — that interface + the parse/DAG/eval path land with the formula slice.
//
// PURE TypeScript — type-only; imports Signal/Derived from `@ship/sui-runtime`
// and the shared shapes from `./types`.
//
// Authority: build-spec §"API sketch" (SheetOptions + Sheet facade), §"id per
// sheet + stable A1 are REQUIRED", §"Authoritative Cell / Selection / Range /
// DAG API", §"Controlled-slice rule".

import type { Derived, Signal } from '@ship/sui-runtime'
import type {
  CellAddress,
  CellData,
  CellEdit,
  CellFormat,
  CellRange,
  CellStyle,
  CellType,
  FormulaError,
  ResolvedCell,
  ScalarValue,
  SelectionState,
  SheetDef,
  WorkbookDef,
} from './types'

export type {
  CellAddress,
  CellData,
  CellEdit,
  CellFormat,
  CellRange,
  CellStyle,
  CellType,
  FormulaError,
  ResolvedCell,
  ScalarValue,
  SelectionState,
  SheetDef,
  WorkbookDef,
}

/** The plain (non-reactive) snapshot of the engine's internal state slices —
 *  the `initialState` seed shape + the `on*Change` payload basis. */
export interface SheetState {
  selection: SelectionState
  editing: { cell: CellAddress; draft: string } | null
  activeSheetId: string | null
}

/** The controlled-slice shape: a slice present in `state` is engine-READ-ONLY
 *  (the engine reads the Signal + fires the matching `on*Change`). A slice
 *  absent here is internal (engine-owned, seeded by `initialState`). */
export interface ReactiveSheetState {
  selection: Signal<SelectionState>
  editing: Signal<{ cell: CellAddress; draft: string } | null>
  activeSheetId: Signal<string | null>
}

/** OUR formula-core contract — the engine binds HERE, never to the vendor API
 *  (the vendor seam). The parse/eval path lands with the formula slice; the
 *  interface is declared now so the engine + view can type against it. */
export interface FormulaCore {
  parse(
    formula: string,
    origin: CellAddress,
  ): {
    deps: ReadonlyArray<CellAddress | { range: [CellAddress, CellAddress] }>
    evaluate(getCell: (a: CellAddress) => ScalarValue): ScalarValue
  }
  supportedFunctions(): ReadonlyArray<{ name: string; arity: [number, number]; about?: string }>
}

export interface SheetOptions {
  /** Raw OR a live Signal/Derived identity (growing/swapping it re-parses). */
  workbook: WorkbookDef | Signal<WorkbookDef> | Derived<WorkbookDef>
  /** Default: the literal-only core; injectable for tests/swap (formula slice). */
  formulaCore?: FormulaCore
  /** A sheet is ALWAYS 2-D cell nav (contrast Table's row mode). */
  navMode?: 'cell'
  /** `'manual'` batches dirty cells until `recalc()` (large books). */
  recalcMode?: 'auto' | 'manual'
  /** Iterative-calc cap for intentional circular refs (default 0 = off). */
  maxIterations?: number
  /** CONTROLLED slices — present ⇒ engine reads-only + fires the matching on*. */
  state?: Partial<ReactiveSheetState>
  /** UNCONTROLLED seed. */
  initialState?: Partial<SheetState>
  /** Server seam — the engine REFLECTS, never fetches. */
  isLoading?: boolean | Signal<boolean>
  error?: unknown | Signal<unknown>
  /** Engine → consumer write path (Cardinal 4 stays in the consumer data binding). */
  onCellChange?: (edits: CellEdit[]) => void
  onSelectionChange?: (sel: SelectionState) => void
  onActiveSheetChange?: (sheetId: string) => void
  onError?: (e: unknown) => void
}

export interface Sheet {
  // ── RENDER SURFACES — split so the virtualizer windows ONLY the center ──
  activeSheet: Derived<SheetDef>
  rowCount: Derived<number>
  colCount: Derived<number>
  frozenRowCount: Derived<number>
  frozenColCount: Derived<number>
  /** Logical row span the row-virtualizer counts (frozen rows excluded). */
  centerRowRange: Derived<{ start: number; end: number }>
  /** Column-virtualizer span (HORIZONTAL windowing; frozen cols excluded). */
  centerColRange: Derived<{ start: number; end: number }>

  // ── CELL READS — pure, tracked when read in derived/template ──
  getCell(a1: string): Derived<ResolvedCell>
  getCellAt(row: number, col: number): Derived<ResolvedCell>
  rowHeight(index: number): number
  colWidth(index: number): number
  /** Spacer geometry source — both scrollbars honest over a sparse extent. */
  totalWidth: Derived<number>
  totalHeight: Derived<number>

  // ── SELECTION (range-shaped — the headline difference from Table) ──
  selection: Derived<SelectionState>
  activeCell: Derived<CellAddress>
  isInRange(a: CellAddress): Derived<boolean>

  // ── FORMULA / DAG (read-only; the eval path lands with the formula slice) ──
  dependents(a1: string): Derived<string[]>
  precedents(a1: string): Derived<string[]>
  hasCycle: Derived<boolean>
  isLoading: Derived<boolean>
  error: Derived<unknown>

  // ── EDITING model — backs sui-cell-commit; nav-integrated ──
  editing: Derived<{ cell: CellAddress; draft: string } | null>

  // ── METHODS — mutators; call ONLY from on:* / mount (Cardinal 3) ──
  setActiveCell(a: CellAddress, opts?: { extend?: boolean }): void
  setSelection(
    ranges: CellRange[] | { start: CellAddress; end: CellAddress }[],
    active?: CellAddress,
    opts?: { additive?: boolean },
  ): void
  moveActive(
    dir:
      | 'up'
      | 'down'
      | 'left'
      | 'right'
      | 'home'
      | 'end'
      | 'ctrl-home'
      | 'ctrl-end'
      | 'tab'
      | 'enter',
    opts?: { extend?: boolean },
  ): void
  startEditing(a?: CellAddress, seed?: string): void
  updateDraft(text: string): void
  commitEdit(opts?: { advance?: 'down' | 'right' | 'none' }): void
  cancelEdit(): void
  setCell(a1: string, data: CellData): void
  setCells(edits: CellEdit[]): void
  clearCells(range: CellRange, what?: 'all' | 'values' | 'formats'): void
  fillRange(from: CellRange, to: CellRange, mode?: 'copy' | 'series'): void
  setFormat(range: CellRange, fmt: Partial<CellFormat>): void
  setStyle(range: CellRange, style: Partial<CellStyle>): void
  setFrozen(rows: number, cols: number): void
  setColWidth(col: number, px: number): void
  setRowHeight(row: number, px: number): void
  setActiveSheet(sheetId: string): void
  recalc(): void
  reset(): void
}
