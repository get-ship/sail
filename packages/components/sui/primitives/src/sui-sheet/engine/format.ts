// sui-sheet engine — display formatting (the engine formats, the view renders).
//
// `ResolvedCell.display` is the STRING the view renders; the engine derives it
// here from the resolved scalar value + the cell's `CellType`/`numberFormat`.
// The literal path is the foundational slice: numbers/booleans/strings + a
// minimal `0.0%` / currency / thousands pattern. The formula slice extends the
// value SOURCE (evaluated results) but reuses this formatter unchanged.
//
// PURE TypeScript — no runtime/SDK imports.
//
// Authority: build-spec §Features #3 (per-cell types + formats drive
// ResolvedCell.display), §States, behavioral spec #8.

import type { CellFormat, CellType, FormulaError, ScalarValue } from './types'

/** Is this scalar an error wrapper? */
export function isErrorValue(v: ScalarValue): v is { error: FormulaError } {
  return typeof v === 'object' && v !== null && 'error' in v
}

/** Resolve the effective cell type: an explicit `format.type` wins; otherwise
 *  auto-detect from the value (`auto`). */
export function resolveKind(value: ScalarValue, format: CellFormat | undefined): CellType {
  const declared = format?.type
  if (declared && declared !== 'auto') return declared
  if (typeof value === 'number') return 'number'
  if (typeof value === 'boolean') return 'boolean'
  return 'text'
}

/** Format a resolved scalar into the display string. Empty (`null`/`undefined`)
 *  → `''`; an error → its code; numbers honour a small pattern vocabulary. */
export function formatValue(value: ScalarValue, format: CellFormat | undefined): string {
  if (value == null) return ''
  if (isErrorValue(value)) return value.error
  if (typeof value === 'boolean') return value ? 'TRUE' : 'FALSE'
  if (typeof value === 'string') return value

  // number
  const kind = resolveKind(value, format)
  const pattern = format?.numberFormat
  if (kind === 'percent' || pattern?.includes('%')) {
    const decimals = decimalsFromPattern(pattern) ?? 0
    return `${(value * 100).toFixed(decimals)}%`
  }
  if (kind === 'currency') {
    const decimals = decimalsFromPattern(pattern) ?? 2
    return `$${groupThousands(value.toFixed(decimals))}`
  }
  if (pattern) {
    const decimals = decimalsFromPattern(pattern) ?? 0
    const fixed = value.toFixed(decimals)
    return pattern.includes(',') ? groupThousands(fixed) : fixed
  }
  return String(value)
}

/** Count the digits after the `.` in a number pattern (`'#,##0.00' → 2`). */
function decimalsFromPattern(pattern: string | undefined): number | null {
  if (!pattern) return null
  const dot = pattern.indexOf('.')
  if (dot < 0) return 0
  let n = 0
  for (let i = dot + 1; i < pattern.length; i++) {
    const ch = pattern[i]
    if (ch === '0' || ch === '#') n++
    else break
  }
  return n
}

/** Insert thousands separators into the integer part of a fixed-decimal string. */
function groupThousands(fixed: string): string {
  const neg = fixed.startsWith('-')
  const body = neg ? fixed.slice(1) : fixed
  const [int, frac] = body.split('.')
  const grouped = int!.replace(/\B(?=(\d{3})+(?!\d))/g, ',')
  const out = frac != null ? `${grouped}.${frac}` : grouped
  return neg ? `-${out}` : out
}
