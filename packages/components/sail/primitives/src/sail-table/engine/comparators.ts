// sail-table engine — typed comparators (Backbone #1, Slice 1a).
//
// Replaces the `String()`-coerced single-column sort with numeric/date/string
// detection: '10' sorts AFTER '2' (the bug ship-agents-table hits on
// skills_count/mcp_count), locale string collation, chronological dates,
// null/undefined sink-last regardless of direction, and a stable result. PURE
// TypeScript — no runtime imports.
//
// Authority: build-spec §"REPLACE" (typed comparators) + §Features #2
// ("Default comparator numeric-aware then locale string").

import type { CellValue } from './types'

export type SortType = 'number' | 'date' | 'string'

/** A bare comparator: negative if a<b, positive if a>b, 0 if equal. */
export type Comparator = (a: unknown, b: unknown) => number

// ── Detection ────────────────────────────────────────────────────────────────

const ISO_DATE = /^\d{4}-\d{2}(-\d{2})?([T ]\d{2}:\d{2}(:\d{2})?)?/

function isEmpty(v: unknown): boolean {
  return v == null || v === ''
}

/** True when `v` is a finite number or a string that parses to a finite number. */
function looksNumeric(v: unknown): boolean {
  if (typeof v === 'number') return Number.isFinite(v)
  if (typeof v === 'string') {
    const t = v.trim()
    if (t === '') return false
    return Number.isFinite(Number(t))
  }
  return false
}

function looksDate(v: unknown): boolean {
  if (v instanceof Date) return !Number.isNaN(v.getTime())
  if (typeof v === 'string') {
    if (!ISO_DATE.test(v)) return false
    return !Number.isNaN(Date.parse(v))
  }
  return false
}

/**
 * Classify a column's values into a sort type by sampling its non-empty
 * members. Numeric wins (so numeric strings sort numerically), then date
 * (ISO-shaped, and NOT already numeric), else string. An all-empty column
 * falls back to string.
 */
export function detectSortType(values: readonly unknown[]): SortType {
  let seen = 0
  let allNumeric = true
  let allDate = true
  for (const v of values) {
    if (isEmpty(v)) continue
    seen++
    if (!looksNumeric(v)) allNumeric = false
    if (!looksDate(v)) allDate = false
    if (!allNumeric && !allDate) break
  }
  if (seen === 0) return 'string'
  if (allNumeric) return 'number'
  if (allDate) return 'date'
  return 'string'
}

// ── Coercion helpers ─────────────────────────────────────────────────────────

function toNumber(v: unknown): number {
  if (typeof v === 'number') return v
  return Number(String(v).trim())
}

function toTime(v: unknown): number {
  if (v instanceof Date) return v.getTime()
  return Date.parse(String(v))
}

// ── Comparison ───────────────────────────────────────────────────────────────

/**
 * Compare two values under a sort type. null/undefined/'' always sink LAST
 * regardless of the type or the caller's direction sign (two empties are
 * equal → 0). The non-empty branch is numeric-aware, locale-string, or
 * chronological per `type`.
 */
export function compareValues(a: unknown, b: unknown, type: SortType): number {
  const ae = isEmpty(a)
  const be = isEmpty(b)
  if (ae && be) return 0
  if (ae) return 1
  if (be) return -1

  switch (type) {
    case 'number': {
      const na = toNumber(a)
      const nb = toNumber(b)
      if (Number.isNaN(na) && Number.isNaN(nb)) return 0
      if (Number.isNaN(na)) return 1
      if (Number.isNaN(nb)) return -1
      return na < nb ? -1 : na > nb ? 1 : 0
    }
    case 'date': {
      const ta = toTime(a)
      const tb = toTime(b)
      if (Number.isNaN(ta) && Number.isNaN(tb)) return 0
      if (Number.isNaN(ta)) return 1
      if (Number.isNaN(tb)) return -1
      return ta < tb ? -1 : ta > tb ? 1 : 0
    }
    default:
      return String(a).localeCompare(String(b))
  }
}

// ── Resolution ───────────────────────────────────────────────────────────────

/**
 * Resolve a sort spec to a bare comparator. A SortType name yields the typed
 * comparator; a user-supplied function passes through UNCHANGED (identity), so
 * a column's custom `sortFn` is honored verbatim.
 */
export function resolveComparator(spec: SortType | Comparator): Comparator {
  if (typeof spec === 'function') return spec
  return (a: unknown, b: unknown) => compareValues(a, b, spec)
}

/** Detect the type of a CellValue column and return its comparator. */
export function comparatorForValues(values: readonly CellValue[]): Comparator {
  return resolveComparator(detectSortType(values))
}
