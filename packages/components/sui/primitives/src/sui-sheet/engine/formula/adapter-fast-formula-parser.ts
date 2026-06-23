// sui-sheet engine — the REAL FormulaCore over the `fast-formula-parser` npm dep.
//
// The ONLY file that imports the parser library (the purity gate asserts this).
// It maps the library's API ↔ OUR `FormulaCore` contract: 1-based parser refs ↔
// our 0-based `CellAddress`, the library's `FormulaError` ↔ our closed
// `FormulaError` vocabulary (`#DIV/0!`/`#VALUE!`/`#NAME?`/…), and the dep-parser
// output ↔ the cell/range dep set the DAG consumes. The engine binds to
// `FormulaCore`, never to anything in this file.
//
// PURE TypeScript — imports `fast-formula-parser` + OUR contract types ONLY.
// `fast-formula-parser` is a `devDependency`: the Rolldown bundler inlines it at
// build, so the published pack carries no runtime dep (FFP is MIT). `FormulaParser`
// is the package's DEFAULT export; `DepParser`/`FormulaError` are named exports.
//
// Authority: build-spec §"The formula-core boundary"
// (`createFastFormulaCore()` maps FFP ↔ FormulaCore, opaque), §Slice 2/6a,
// adapter-fast-formula-parser.test.ts.

import FormulaParser, { DepParser, FormulaError as FFPError } from 'fast-formula-parser'
import type { CellRef, RangeRef } from 'fast-formula-parser'
import type { FormulaCore, FormulaError } from '../contract'
import type { CellAddress, ScalarValue } from '../types'

type Dep = CellAddress | { range: [CellAddress, CellAddress] }

/** Our 0-based address → the vendor's 1-based ref. */
function toRef(a: CellAddress): { row: number; col: number } {
  return { row: a.row + 1, col: a.col + 1 }
}

/** Map a vendor error code to OUR closed `FormulaError`. Unknown vendor codes
 *  (`#ERROR!` from an unimplemented function / parse failure) become `#NAME?`. */
function mapError(code: string): FormulaError {
  switch (code) {
    case '#DIV/0!':
    case '#VALUE!':
    case '#REF!':
    case '#NAME?':
    case '#NUM!':
    case '#N/A':
    case '#NULL!':
      return code as FormulaError
    default:
      return '#NAME?'
  }
}

/** Convert a vendor scalar (number/string/boolean/null/FormulaError) → ours. */
function fromVendorScalar(v: unknown): ScalarValue {
  if (v == null) return null
  if (v instanceof FFPError) return { error: mapError(v.error) }
  if (typeof v === 'number' || typeof v === 'string' || typeof v === 'boolean') return v
  // An array result (array formula) is not a scalar cell — surface #VALUE!.
  return { error: '#VALUE!' }
}

export function createFastFormulaCore(): FormulaCore {
  const depParser = new DepParser()
  // A throwaway parser used only for the supportedFunctions() catalog.
  const catalog = new FormulaParser()

  return {
    parse(formula: string, origin: CellAddress) {
      const body = formula.startsWith('=') ? formula.slice(1) : formula
      const position = { ...toRef(origin), sheet: '__sui_sheet__' }

      let deps: Dep[]
      try {
        deps = (depParser.parse(body, position, true) as Array<CellRef | RangeRef>).map(toDep)
      } catch {
        deps = []
      }

      return {
        deps,
        evaluate(getCell: (a: CellAddress) => ScalarValue): ScalarValue {
          // A fresh parser per evaluate binds this call's getCell — the core
          // stays stateless across cells (no shared mutable onCell closure).
          const parser = new FormulaParser({
            onCell: (ref: CellRef) => toVendorScalar(getCell({ row: ref.row - 1, col: ref.col - 1 })),
            onRange: (ref: RangeRef) => {
              const grid: (number | string | boolean | null)[][] = []
              for (let r = ref.from.row; r <= ref.to.row; r++) {
                const row: (number | string | boolean | null)[] = []
                for (let c = ref.from.col; c <= ref.to.col; c++) {
                  row.push(toVendorScalar(getCell({ row: r - 1, col: c - 1 })))
                }
                grid.push(row)
              }
              return grid
            },
          })
          try {
            const res = parser.parse(body, position, false)
            if (res instanceof FFPError) return { error: mapError(res.error) }
            return fromVendorScalar(res)
          } catch (e) {
            if (e instanceof FFPError) return { error: mapError(e.error) }
            return { error: '#NAME?' }
          }
        },
      }
    },

    supportedFunctions() {
      return catalog.supportedFunctions().map((name) => ({
        name,
        arity: [0, 255] as [number, number],
      }))
    },
  }
}

/** Our scalar → the vendor's accepted cell value (errors pass through; the
 *  vendor recognizes its own `FormulaError`). Empty stays empty (the vendor
 *  coerces a null cell to 0 in arithmetic, matching spreadsheet semantics). */
function toVendorScalar(v: ScalarValue): number | string | boolean | null {
  if (v == null) return null
  if (typeof v === 'object' && 'error' in v) {
    // Re-inflate our error code into a vendor FormulaError so it propagates.
    return new FFPError(v.error) as unknown as never
  }
  return v
}

/** Vendor dep entry (1-based) → our 0-based cell/range dep. */
function toDep(d: CellRef | RangeRef): Dep {
  if ('from' in d) {
    return {
      range: [
        { row: d.from.row - 1, col: d.from.col - 1 },
        { row: d.to.row - 1, col: d.to.col - 1 },
      ],
    }
  }
  return { row: d.row - 1, col: d.col - 1 }
}
