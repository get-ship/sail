// sui-sheet engine — shared shapes (the cell / format / style / def vocabulary).
//
// The value types every engine module + the view read: scalar values, formula
// errors, the addressed cell + its format/style, the worksheet + workbook defs,
// the range/selection algebra, and the per-cell edit unit. Split from
// `contract.ts` (the `Sheet` facade + `SheetOptions`) so each file stays under
// the cap; `contract.ts` re-exports these so one import covers both.
//
// PURE TypeScript — type-only (no runtime imports). A1 is a STRING projection of
// the 0-based `(row,col)` internal coordinate; the algebra lives in `a1.ts`.
//
// Authority: build-spec §"API sketch" (the CellType / CellFormat / CellStyle /
// CellData / SheetDef / WorkbookDef shapes), §"Authoritative Cell / Selection /
// Range / DAG API", §"id per sheet + stable A1 are REQUIRED".

// ── Scalar value space (the formula-core contract surface) ──────────────────

/** The closed set of spreadsheet formula errors (Excel/Sheets vocabulary).
 *  `#CYCLE!` is the engine's own back-edge marker. */
export type FormulaError =
  | '#NULL!'
  | '#DIV/0!'
  | '#VALUE!'
  | '#REF!'
  | '#NAME?'
  | '#NUM!'
  | '#N/A'
  | '#CYCLE!'

/** What a cell evaluates to: a literal, an error wrapper, or empty (`null`). */
export type ScalarValue = number | string | boolean | { error: FormulaError } | null

/** A 0-based internal cell coordinate. A1 is the string projection (`a1.ts`).
 *  The `$` markers ride along so fill-series can adjust relative refs while
 *  leaving absolute (`$A$1`) ones fixed. */
export interface CellAddress {
  row: number
  col: number
  /** `$` before the column letter (`$A1`) — the column is fixed under fill. */
  absCol?: boolean
  /** `$` before the row number (`A$1`) — the row is fixed under fill. */
  absRow?: boolean
}

// ── Per-cell type / format / style ──────────────────────────────────────────

export type CellType =
  | 'auto'
  | 'text'
  | 'number'
  | 'boolean'
  | 'date'
  | 'datetime'
  | 'currency'
  | 'percent'
  | 'formula'

/** Token-SLOT names only — never raw color. The engine carries the slot name;
 *  the CSS resolves it to `--tone-*` / `--surface-*` (build-fail-guarded by
 *  `sui-css-literal`). */
export interface CellStyle {
  bold?: boolean
  italic?: boolean
  underline?: boolean
  /** → tone ramp (`--tone-<tone>-bg/-fg/-border`), NOT a raw color. */
  tone?: 'default' | 'muted' | 'info' | 'success' | 'warning' | 'destructive' | 'accent'
  /** Token slot only: a surface layer or the tone slot. No hex. */
  bg?: 'surface-1' | 'surface-2' | 'tone'
  borders?: { top?: boolean; right?: boolean; bottom?: boolean; left?: boolean }
}

/** Per-cell display contract — the view reads, never re-derives the raw value. */
export interface CellFormat {
  type?: CellType
  /** ICU-ish pattern, e.g. `'#,##0.00'` / `'0.0%'` / `'yyyy-mm-dd'`. */
  numberFormat?: string
  align?: 'start' | 'center' | 'end'
  wrap?: boolean
  style?: CellStyle
}

/** What the consumer hands in per addressed cell (SPARSE — only authored). */
export interface CellData {
  /** Literal; ignored when `formula` is present. */
  value?: ScalarValue
  /** Begins with `'='` — parsed via the FormulaCore (formula slice). */
  formula?: string
  format?: CellFormat
}

// ── Worksheet / workbook defs ───────────────────────────────────────────────

export interface SheetDef {
  /** REQUIRED — stable across edits; keys cross-sheet refs + per-sheet state. */
  id: string
  /** Tab label; a valid A1 sheet-name (used in `'Sheet2!A1'` refs). */
  name: string
  /** Logical extent — the grid may be sparse/infinite past authored cells. */
  rowCount: number
  colCount: number
  /** SPARSE — only authored cells present, keyed by A1. */
  cells: Record<string, CellData>
  /** Count of leading frozen bands (sticky quadrants). */
  frozenRows?: number
  frozenCols?: number
  /** Per-index size overrides. */
  rowHeights?: Record<number, number>
  colWidths?: Record<number, number>
  defaultRowHeight?: number
  defaultColWidth?: number
}

export interface WorkbookDef {
  sheets: SheetDef[]
  activeSheetId?: string
}

// ── Range / selection algebra (range-shaped, NOT a row-id set) ───────────────

/** A rectangular cell range value type. Methods normalize drag direction,
 *  report membership, and project to/from A1. */
export interface CellRange {
  start: CellAddress
  end: CellAddress
  /** Top-left / bottom-right regardless of drag direction. */
  normalized(): CellRange
  contains(a: CellAddress): boolean
  /** Column span (inclusive). */
  width(): number
  /** Row span (inclusive). */
  height(): number
  /** `'A1:C3'`. */
  toA1(): string
  cells(): Iterable<CellAddress>
}

export interface SelectionState {
  active: CellAddress
  anchor: CellAddress
  /** Multi-range via Ctrl. */
  ranges: CellRange[]
  mode: 'cell' | 'range' | 'row' | 'col'
  isEmpty(): boolean
}

// ── The resolved render contract (what the view reads per cell) ─────────────

export interface ResolvedCell {
  a1: string
  address: CellAddress
  raw: CellData
  /** Literal or evaluated formula result. */
  value: ScalarValue
  /** Formatted per `numberFormat` / `type` — the string the view renders. */
  display: string
  kind: CellType
  isFormula: boolean
  error: FormulaError | null
  isInSelection: boolean
  isActive: boolean
  isEditing: boolean
  isFrozen: 'row' | 'col' | 'both' | false
  style: CellStyle
  format: CellFormat
}

/** The unit of `onCellChange` / the undo-redo entry / the paste-fill diff. */
export interface CellEdit {
  a1: string
  before: CellData
  after: CellData
}
