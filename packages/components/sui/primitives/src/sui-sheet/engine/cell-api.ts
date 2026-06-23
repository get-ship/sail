// sui-sheet engine — the ResolvedCell factory (per-cell render contract).
//
// `getCell(a1)` / `getCellAt(r,c)` return a MEMOIZED `Derived<ResolvedCell>` —
// the resolved value + formatted display + the per-cell interaction flags
// (`isActive`/`isInSelection`/`isEditing`/`isFrozen`) the view reads. Memoizing
// per A1 keeps reads surgical: a derived for `Z9` only re-runs when `Z9`'s cell
// signal OR the selection slice it reads changes — a write to `A1` leaves it a
// cache hit (the dirty-suffix discipline). The literal path resolves
// `raw.value`; the formula path (deferred) swaps the value SOURCE only.
//
// PURE TypeScript — `derived` from `@ship/sui-runtime` ONLY.
//
// Authority: build-spec §"Authoritative … API" (ResolvedCell members), §"Signal
// graph" (resolvedCells map; only the dirty suffix recomputes), §Features #3.

import { derived, type Derived } from '@ship/sui-runtime'
import { fromA1, toA1 } from './a1'
import { formatValue, isErrorValue, resolveKind } from './format'
import type { CellStore, InteractionState } from './state'
import type {
  CellAddress,
  CellData,
  FormulaError,
  ResolvedCell,
  ScalarValue,
  SheetDef,
} from './types'

const EMPTY_CELL: CellData = {}

/** The formula read-side the cell-api consults for a formula cell: it tracks the
 *  cell's recompute version (so the derived re-runs surgically) and returns the
 *  cached evaluated result. Supplied by the formula engine; absent in pure
 *  literal-only setups (the resolved value then falls through to `null`). */
export interface FormulaResolver {
  trackVersion(a1: string): void
  result(a1: string): { value: ScalarValue; error: FormulaError | null } | undefined
}

export interface CellApi {
  getCell(a1: string): Derived<ResolvedCell>
  getCellAt(row: number, col: number): Derived<ResolvedCell>
}

export function buildCellApi(
  store: CellStore,
  state: InteractionState,
  activeSheet: Derived<SheetDef | undefined>,
  formula?: FormulaResolver,
): CellApi {
  const memo = new Map<string, Derived<ResolvedCell>>()

  const build = (a1: string): Derived<ResolvedCell> => {
    const address = fromA1(a1)
    const d = derived<ResolvedCell>(() => {
      const raw = store.read(a1) ?? EMPTY_CELL
      // Formula path: track the cell's recompute version + read its evaluated
      // result. Literal path: the authored literal. (A formula with no resolver
      // wired — pure literal setup — resolves null.)
      let value: ScalarValue
      if (raw.formula != null && formula) {
        formula.trackVersion(a1)
        value = formula.result(a1)?.value ?? null
      } else {
        value = raw.formula != null ? null : (raw.value ?? null)
      }
      const error: FormulaError | null = isErrorValue(value) ? value.error : null
      const format = raw.format ?? {}
      const sel = state.selection()
      const editing = state.editing()
      const def = activeSheet()
      return {
        a1,
        address,
        raw,
        value,
        display: formatValue(value, format),
        kind: resolveKind(value, format),
        isFormula: raw.formula != null,
        error,
        isInSelection: sel.ranges.some((r) => r.contains(address)),
        isActive: sameAddr(sel.active, address),
        isEditing: editing != null && sameAddr(editing.cell, address),
        isFrozen: frozenKind(def, address),
        style: format.style ?? {},
        format,
      }
    })
    return d
  }

  const getCell = (a1: string): Derived<ResolvedCell> => {
    let d = memo.get(a1)
    if (!d) {
      d = build(a1)
      memo.set(a1, d)
    }
    return d
  }

  const getCellAt = (row: number, col: number): Derived<ResolvedCell> =>
    getCell(toA1({ row, col }))

  return { getCell, getCellAt }
}

function sameAddr(a: CellAddress, b: CellAddress): boolean {
  return a.row === b.row && a.col === b.col
}

/** A cell inside a leading frozen band is sticky in that axis. */
function frozenKind(def: SheetDef | undefined, a: CellAddress): ResolvedCell['isFrozen'] {
  const fr = def?.frozenRows ?? 0
  const fc = def?.frozenCols ?? 0
  const inRow = a.row < fr
  const inCol = a.col < fc
  if (inRow && inCol) return 'both'
  if (inRow) return 'row'
  if (inCol) return 'col'
  return false
}
