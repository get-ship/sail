// sui-sheet engine — the formula recalc engine (DAG × FormulaCore × cell store).
//
// The reactive half of the formula slice: it parses every formula cell through
// the injected `FormulaCore`, records its deps in the `CellDag`, and on a cell
// write recomputes ONLY the dirty suffix (the changed cell + its transitive
// dependents) in topological order. Results are cached per-A1; a per-cell version
// `Signal` is bumped when a formula's value changes so `cell-api`'s derived
// re-runs surgically — an unrelated formula (or literal) cell stays a cache hit.
//
// Cycle detection marks the participating cells `#CYCLE!`; a reference outside
// the sheet's logical extent is `#REF!`. `recalcMode:'manual'` coalesces dirty
// cells until `recalc()`.
//
// PURE TypeScript — `signal`/`untrack` from `@ship/sui-runtime` + OUR DAG/a1/
// contract. The vendor never appears here (it lives behind `FormulaCore`).
//
// Authority: build-spec §Features #2 (DAG + dirty-suffix recalc), §"Signal graph"
// (a write recomputes the dirty DAG suffix only), §States (per-cell #CYCLE!/#REF!
// do not fault the grid), create-sheet.formula.test.ts.

import { signal, untrack, type Signal } from '@ship/sui-runtime'
import { createDag } from './dag'
import { fromA1, toA1 } from './a1'
import type { FormulaCore } from './contract'
import type { CellAddress, FormulaError, ScalarValue } from './types'

/** The evaluated outcome of a formula cell (cached; the view reads via cell-api). */
export interface FormulaResult {
  value: ScalarValue
  error: FormulaError | null
}

export interface FormulaEngine {
  /** Re-scan the authored formula set against the current sheet (init + on a
   *  workbook-identity swap). Rebuilds the DAG and computes every formula. */
  sync(formulas: ReadonlyMap<string, string>, extent: { rows: number; cols: number }): void
  /** A literal/formula write landed at `a1` — re-parse if it became a formula,
   *  then recompute the dirty suffix (deferred in manual mode until recalc()). */
  onCellWrite(a1: string, formula: string | undefined): void
  /** Force a full recompute of every formula (manual-mode flush / reset). */
  recalcAll(): void
  /** The cached result for a formula cell (synchronous; `null` if not a formula). */
  result(a1: string): FormulaResult | undefined
  /** Track a formula cell's recompute version (read in cell-api's derived). */
  trackVersion(a1: string): void
  /** Track the DAG structure version (read in dependents/precedents/hasCycle
   *  deriveds so they re-run when an edge or cycle changes). */
  trackGraph(): void
  dependents(a1: string): string[]
  precedents(a1: string): string[]
  hasCycle(): boolean
}

export function buildFormulaEngine(
  core: FormulaCore,
  readCell: (a1: string) => ScalarValue,
  recalcMode: 'auto' | 'manual',
): FormulaEngine {
  const dag = createDag()
  const results = new Map<string, FormulaResult>()
  const compiled = new Map<string, ReturnType<FormulaCore['parse']>>()
  const versions = new Map<string, Signal<number>>()
  const graphVersion = signal(0)
  let extent = { rows: 0, cols: 0 }
  const pendingDirty = new Set<string>()

  const version = (a1: string): Signal<number> => {
    let v = versions.get(a1)
    if (!v) {
      v = signal(0)
      versions.set(a1, v)
    }
    return v
  }
  const bump = (a1: string): void => version(a1).set((n) => n + 1)

  /** Expand a parsed dep set into the flat A1 precedent list for the DAG. */
  const depsToA1 = (deps: ReturnType<FormulaCore['parse']>['deps']): string[] => {
    const out: string[] = []
    for (const d of deps) {
      if ('range' in d) {
        const [s, e] = d.range
        for (let r = Math.min(s.row, e.row); r <= Math.max(s.row, e.row); r++) {
          for (let c = Math.min(s.col, e.col); c <= Math.max(s.col, e.col); c++) {
            out.push(toA1({ row: r, col: c }))
          }
        }
      } else {
        out.push(toA1({ row: d.row, col: d.col }))
      }
    }
    return out
  }

  /** Does any dep of `a1` fall outside the sheet's logical extent? → #REF!. */
  const refError = (a1: string): boolean => {
    const parsed = compiled.get(a1)
    if (!parsed) return false
    for (const d of parsed.deps) {
      const corners = 'range' in d ? d.range : [d, d]
      for (const a of corners) {
        if (a.row < 0 || a.col < 0 || a.row >= extent.rows || a.col >= extent.cols) return true
      }
    }
    return false
  }

  const getCellValue = (addr: CellAddress): ScalarValue => readCell(toA1(addr))

  /** Recompute one formula cell, write its cache, and bump its version if the
   *  value changed. Cycle membership / REF are applied here. Runs untracked so
   *  reading peer cells never subscribes a reactive scope to the recompute. */
  const recomputeOne = (a1: string, cyclic: Set<string>): void => {
    const parsed = compiled.get(a1)
    if (!parsed) return
    let next: FormulaResult
    if (cyclic.has(a1)) {
      next = { value: { error: '#CYCLE!' }, error: '#CYCLE!' }
    } else if (refError(a1)) {
      next = { value: { error: '#REF!' }, error: '#REF!' }
    } else {
      const value = untrack(() => parsed.evaluate(getCellValue))
      const error = isError(value) ? value.error : null
      next = { value, error }
    }
    const prev = results.get(a1)
    results.set(a1, next)
    if (!prev || prev.value !== next.value || prev.error !== next.error) bump(a1)
  }

  /** (Re)parse a formula cell and register its deps in the DAG. */
  const compile = (a1: string, formula: string): void => {
    const parsed = core.parse(formula, fromA1(a1))
    compiled.set(a1, parsed)
    dag.setFormula(a1, depsToA1(parsed.deps))
    graphVersion.set((n) => n + 1)
  }

  const drop = (a1: string): void => {
    compiled.delete(a1)
    dag.clearFormula(a1)
    if (results.delete(a1)) bump(a1)
    graphVersion.set((n) => n + 1)
  }

  /** Recompute a set of dirty formula cells in topo order, applying the global
   *  cycle set so every cycle participant resolves to #CYCLE!. */
  const recompute = (dirty: Iterable<string>): void => {
    const cyclic = new Set(dag.cycleCells())
    // Each dirty cell + its transitive dependents, in one merged topo order.
    const order: string[] = []
    const seen = new Set<string>()
    for (const a1 of dirty) {
      if (compiled.has(a1) && !seen.has(a1)) {
        order.push(a1)
        seen.add(a1)
      }
      for (const dep of dag.dirtySuffix(a1)) {
        if (compiled.has(dep) && !seen.has(dep)) {
          order.push(dep)
          seen.add(dep)
        }
      }
    }
    for (const a1 of order) recomputeOne(a1, cyclic)
  }

  /** Recompute every authored formula (init / manual-mode flush / reset). */
  const recalcAll = (): void => {
    pendingDirty.clear()
    recompute(compiled.keys())
  }

  return {
    sync(formulas, ext) {
      extent = ext
      // Drop stale formulas no longer authored.
      for (const a1 of [...compiled.keys()]) {
        if (!formulas.has(a1)) drop(a1)
      }
      for (const [a1, f] of formulas) compile(a1, f)
      recalcAll()
    },

    onCellWrite(a1, formula) {
      if (formula != null) compile(a1, formula)
      else drop(a1)
      if (recalcMode === 'manual') {
        pendingDirty.add(a1)
        return
      }
      recompute([a1])
    },

    recalcAll,

    result(a1) {
      return results.get(a1)
    },

    trackVersion(a1) {
      version(a1)()
    },

    trackGraph() {
      graphVersion()
    },

    dependents(a1) {
      return dag.dependents(a1)
    },
    precedents(a1) {
      return dag.precedents(a1)
    },
    hasCycle() {
      return dag.hasCycle()
    },
  }
}

function isError(v: ScalarValue): v is { error: FormulaError } {
  return typeof v === 'object' && v !== null && 'error' in v
}
