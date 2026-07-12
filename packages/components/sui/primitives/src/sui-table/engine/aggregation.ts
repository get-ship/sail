// sui-table engine — the aggregation fn set (Backbone #1, Slice 3b, Feature #5).
//
// Resolves a column's `aggregationFn` (a built-in NAME or a custom function) to
// a reducer over a column's leaf values, used by BOTH the synthetic group row's
// `getValue(colId)` (the per-group aggregate) AND the grand-total footer
// (`getFooterAggregate()` over `filteredRows`). The full set per the build-spec:
//
//   count | sum | min | max | mean | extent | median | unique | uniqueCount
//   | mode | custom
//
// PURE TypeScript — no runtime imports (these are plain reducers; the reactive
// wiring lives in the group stage + the footer computed).
//
// Authority: build-spec §Features #5 ("Aggregation fn set" + "Grand-total /
// footer model" + "a group row's getValue returns the aggregate"), Slice 3b.

import type { AggFn, AggFnName, CellValue, ColumnDef } from './types'

/** Coerce a cell value to a finite number, or NaN when not numeric. */
function num(v: CellValue): number {
  if (typeof v === 'number') return v
  if (typeof v === 'string' && v.trim() !== '') return Number(v)
  return Number.NaN
}

/** Keep only the numeric leaf values (drops null/undefined/non-numeric). */
function numbers(values: CellValue[]): number[] {
  const out: number[] = []
  for (const v of values) {
    const n = num(v)
    if (!Number.isNaN(n)) out.push(n)
  }
  return out
}

/** value→count over the leaf values (the distinct-value histogram). */
function histogram(values: CellValue[]): Map<CellValue, number> {
  const counts = new Map<CellValue, number>()
  for (const v of values) counts.set(v, (counts.get(v) ?? 0) + 1)
  return counts
}

// ── The built-in reducers (each takes the column's leaf values) ───────────────

const REDUCERS: Record<AggFnName, (values: CellValue[]) => CellValue> = {
  count: (values) => values.length,
  sum: (values) => numbers(values).reduce((a, b) => a + b, 0),
  min: (values) => {
    const ns = numbers(values)
    return ns.length ? Math.min(...ns) : undefined
  },
  max: (values) => {
    const ns = numbers(values)
    return ns.length ? Math.max(...ns) : undefined
  },
  mean: (values) => {
    const ns = numbers(values)
    return ns.length ? ns.reduce((a, b) => a + b, 0) / ns.length : undefined
  },
  // extent returns a [min, max] PAIR — typed CellValue at the slot, but the
  // group-row test reads it as a tuple, so the reducer hands the pair through.
  extent: (values) => {
    const ns = numbers(values)
    if (!ns.length) return undefined
    return [Math.min(...ns), Math.max(...ns)] as unknown as CellValue
  },
  median: (values) => {
    const ns = numbers(values).slice().sort((a, b) => a - b)
    if (!ns.length) return undefined
    const mid = Math.floor(ns.length / 2)
    return ns.length % 2 === 0 ? (ns[mid - 1]! + ns[mid]!) / 2 : ns[mid]!
  },
  // unique returns the distinct values as an array (hint for the view).
  unique: (values) => [...histogram(values).keys()] as unknown as CellValue,
  uniqueCount: (values) => histogram(values).size,
  mode: (values) => {
    let best: CellValue
    let bestCount = -1
    for (const [v, c] of histogram(values)) {
      if (c > bestCount) {
        bestCount = c
        best = v
      }
    }
    return best
  },
  // `custom` carries no built-in behavior — a column declaring it MUST supply an
  // `aggregationFn` function (resolved below); the name alone is a no-op floor.
  custom: () => undefined,
}

/**
 * Resolve a column's aggregation reducer. A function `aggregationFn` is honored
 * verbatim (the `custom` path); a NAME resolves to its built-in reducer. A
 * column with no `aggregationFn` has no aggregate (returns null → the group row
 * falls back to its grouping value for that column).
 */
export function resolveAggregator<T>(
  def: ColumnDef<T> | undefined,
): ((columnId: string, leafValues: CellValue[]) => CellValue) | null {
  if (!def) return null
  const fn = (def as { aggregationFn?: AggFnName | AggFn }).aggregationFn
  if (fn == null) return null
  if (typeof fn === 'function') {
    return (columnId, leafValues) => fn(columnId, leafValues)
  }
  const reducer = REDUCERS[fn]
  return reducer ? (_columnId, leafValues) => reducer(leafValues) : null
}

/** Does this column declare an aggregation (drives the footer model)? */
export function hasAggregation<T>(def: ColumnDef<T>): boolean {
  return (def as { aggregationFn?: unknown }).aggregationFn != null
}
