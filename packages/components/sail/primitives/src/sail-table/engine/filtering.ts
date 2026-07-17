// sail-table engine — column + global filtering (Backbone #1, Slice 2).
//
// Builds the head-of-pipeline filter stages from the build-spec signal graph:
//
//   coreRows ─(globalFilter only)─► preColumnFilteredRows ─(columnFilters)─► filteredRows
//
// `preColumnFilteredRows` is global-filtered but PRE-column-filter — it is the
// facet source (Slice 3b reads it). `filteredRows` then applies the per-column
// filters. Composition order is exactly `data ▸ globalFilter ▸ columnFilters`.
//
// Filter-fn resolution (build-spec §Features #3): a column's own `filterFn`
// wins — a function passes through, a string name resolves against the built-in
// table. With no declared fn the engine picks a default from the filter VALUE's
// shape: a [min,max] tuple → inNumberRange, an array → arrIncludes, a string →
// includesString, any other primitive → equals. Per-fn value shapes are the
// widget↔engine contract (text=string, range=[min,max], set=unknown[]).
//
// Global scope: all filterable accessor columns by default; a column opts out
// with `enableGlobalFilter: false`. Manual mode (`manualFiltering`) skips both
// stages (the server already filtered the page).
//
// PURE TypeScript — imports `derived` from `@ship/sail-runtime` only. No DOM, no
// SDK.
//
// Authority: build-spec §Features #3 ("Column + global filtering"), §"Signal
// graph" (`preColumnFilteredRows` the facet source), Slice 2.

import { derived, type Derived, type Signal } from '@ship/sail-runtime'
import type { CellValue, ColumnDef, ColumnFilter, Row } from './types'

/** A resolved predicate over one cell value + the active filter value. */
type Predicate = (cell: CellValue, value: unknown) => boolean

// ── Built-in filter fns (value-shape contract per the build-spec) ─────────────

/** Text: case-insensitive substring. Value shape = string. */
const includesString: Predicate = (cell, value) => {
  if (value == null || value === '') return true
  return cellToText(cell).toLowerCase().includes(String(value).toLowerCase())
}

/** Number range: value shape = [min, max]; an undefined bound is unbounded. */
const inNumberRange: Predicate = (cell, value) => {
  if (!Array.isArray(value)) return true
  const [min, max] = value as [number | undefined, number | undefined]
  if (min == null && max == null) return true
  const n = typeof cell === 'number' ? cell : Number(cell)
  if (Number.isNaN(n)) return false
  if (min != null && n < min) return false
  if (max != null && n > max) return false
  return true
}

/** Set/multi: value shape = unknown[]; the cell (or any member, if the cell is
 *  itself an array) must appear in the selected set. */
const arrIncludes: Predicate = (cell, value) => {
  if (!Array.isArray(value) || value.length === 0) return true
  const wanted = value as unknown[]
  if (Array.isArray(cell)) return cell.some((c) => wanted.includes(c))
  return wanted.includes(cell)
}

/** Primitive equality. */
const equals: Predicate = (cell, value) => {
  if (value == null) return true
  return cell === value
}

/** Fuzzy: each char of the query appears in order (non-contiguous) in the cell. */
const fuzzy: Predicate = (cell, value) => {
  if (value == null || value === '') return true
  const hay = cellToText(cell).toLowerCase()
  const needle = String(value).toLowerCase()
  let i = 0
  for (const ch of hay) {
    if (ch === needle[i]) i++
    if (i === needle.length) return true
  }
  return needle.length === 0
}

const BUILT_INS: Record<string, Predicate> = {
  includesString,
  inNumberRange,
  arrIncludes,
  equals,
  fuzzy,
}

/** Flatten any cell value to searchable text (arrays joined, null → ''). */
function cellToText(cell: CellValue): string {
  if (cell == null) return ''
  if (Array.isArray(cell)) return (cell as unknown[]).join(' ')
  return String(cell)
}

/**
 * Global-filter match for ONE cell: a scalar matches by case-insensitive
 * substring; an array column matches when the query equals one of its elements
 * (whole-token, NOT substring) — so a global query of "orange" finds a `tags`
 * member but "an" does not spuriously match inside "orange".
 */
function globalMatch(cell: CellValue, query: string): boolean {
  if (Array.isArray(cell)) {
    const q = query.toLowerCase()
    return (cell as unknown[]).some((e) => String(e).toLowerCase() === q)
  }
  return cellToText(cell).toLowerCase().includes(query.toLowerCase())
}

// ── Resolution ────────────────────────────────────────────────────────────────

/** Resolve a column's declared `filterFn` (fn or name) to a predicate, or null
 *  when the column declares none (the value-shape default then applies). */
function declaredPredicate<T>(def: ColumnDef<T>): Predicate | null {
  const ff = (def as { filterFn?: unknown }).filterFn
  if (typeof ff === 'function') {
    const fn = ff as <U>(row: Row<U>, columnId: string, value: unknown) => boolean
    // A user FilterFn takes (row, columnId, value); adapt it to the cell-level
    // predicate by ignoring the row wrapper the caller doesn't have here — the
    // engine passes the cell value through the row at the call site instead.
    return (cell, value) =>
      fn({ getValue: () => cell } as unknown as Row<T>, def.id, value)
  }
  if (typeof ff === 'string') return BUILT_INS[ff] ?? null
  return null
}

/** Pick the value-shape default fn when a column declares none. */
function defaultPredicate(value: unknown): Predicate {
  if (Array.isArray(value)) {
    // A [min,max] PAIR of numbers (possibly with undefined bounds) is a range;
    // any other array is a set membership filter.
    if (isRangeTuple(value)) return inNumberRange
    return arrIncludes
  }
  if (typeof value === 'string') return includesString
  return equals
}

function isRangeTuple(value: unknown[]): boolean {
  if (value.length !== 2) return false
  return value.every((v) => v == null || typeof v === 'number')
}

// ── Stage builders ─────────────────────────────────────────────────────────────

export interface FilterInputs<T> {
  coreRows: Derived<Row<T>[]>
  columns: ColumnDef<T>[]
  columnFilters: Signal<ColumnFilter[]>
  globalFilter: Signal<string>
  /** Manual mode: pass both stages through unchanged (server filtered). */
  manual: boolean
}

export interface FilterStages<T> {
  preColumnFilteredRows: Derived<Row<T>[]>
  filteredRows: Derived<Row<T>[]>
}

/** Columns eligible for the global filter: accessor columns not opted out. */
function globalScopeColumns<T>(columns: ColumnDef<T>[]): ColumnDef<T>[] {
  return columns.filter((c) => {
    const isAccessor = 'accessorKey' in c && (c as { accessorKey?: unknown }).accessorKey != null
    if (!isAccessor) return false
    return (c as { enableGlobalFilter?: boolean }).enableGlobalFilter !== false
  })
}

export function buildFilterStages<T>(inputs: FilterInputs<T>): FilterStages<T> {
  const { coreRows, columns, columnFilters, globalFilter, manual } = inputs
  const globalCols = globalScopeColumns(columns)
  const byId = new Map(columns.map((c) => [c.id, c]))

  const preColumnFilteredRows = derived<Row<T>[]>(() => {
    const rows = coreRows()
    if (manual) return rows
    const q = globalFilter()
    if (q == null || q === '') return rows
    return rows.filter((row) =>
      globalCols.some((c) => globalMatch(row.getValue(c.id), q)),
    )
  })

  const filteredRows = derived<Row<T>[]>(() => {
    const rows = preColumnFilteredRows()
    if (manual) return rows
    const filters = columnFilters()
    if (filters.length === 0) return rows
    return rows.filter((row) =>
      filters.every((f) => {
        const def = byId.get(f.id)
        if (!def) return true
        const predicate = declaredPredicate(def) ?? defaultPredicate(f.value)
        return predicate(row.getValue(f.id), f.value)
      }),
    )
  })

  return { preColumnFilteredRows, filteredRows }
}

/**
 * Resolve a column's cell-level predicate from its def + the active filter
 * value (declared `filterFn` wins, else the value-shape default). Exported so
 * faceting reuses the SAME resolution the row-filter stage uses — one filter
 * contract, no parallel system.
 */
export function resolveCellPredicate<T>(
  def: ColumnDef<T>,
  value: unknown,
): (cell: CellValue, value: unknown) => boolean {
  return declaredPredicate(def) ?? defaultPredicate(value)
}

/** True when the value would actually narrow the set (an empty string / empty
 *  array is treated as "no filter" so `hasActiveFilters` is honest). */
export function isActiveFilterValue(value: unknown): boolean {
  if (value == null || value === '') return false
  if (Array.isArray(value)) {
    if (value.length === 0) return false
    if (isRangeTuple(value)) return value.some((v) => v != null)
    return true
  }
  return true
}
