// Ambient types for `fast-formula-parser` 1.0.19 (MIT) — the npm package ships a
// webpack CommonJS bundle with NO bundled `.d.ts`, so this declaration carries the
// surface the FormulaCore adapter binds to. Hand-authored, covering ONLY that
// surface; the full upstream API is opaque.
//
// Export shape (matches the package's CJS interop): `FormulaParser` is the DEFAULT
// export; `DepParser` and `FormulaError` are NAMED exports hung off it.
//
// DO NOT EDIT alongside a version bump without re-checking the upstream shape.

declare module 'fast-formula-parser' {
  /** A cell reference the parser hands to `onCell` / yields from the dep parser.
   *  Row/col are 1-BASED (the parser's external convention). */
  export interface CellRef {
    row: number
    col: number
    sheet?: string
  }

  /** A range reference: two 1-based corners (`onRange` receives this shape). */
  export interface RangeRef {
    from: { row: number; col: number }
    to: { row: number; col: number }
    sheet?: string
  }

  /** The position of the parsed formula (its origin cell), 1-based. */
  export interface Position {
    row: number
    col: number
    sheet?: string
  }

  /** A scalar the parser accepts back from `onCell` / inside an `onRange` grid. */
  export type CellValue = number | string | boolean | null

  /** A formula error instance. Eval errors are RETURNED from `parse`; an unknown
   *  function / parse failure is THROWN. Read `.error` for the code (e.g.
   *  `'#DIV/0!'`, `'#VALUE!'`, `'#NAME?'`, `'#ERROR!'`). */
  export class FormulaError extends Error {
    constructor(error: string, msg?: string, details?: unknown)
    readonly error: string
    equals(err: unknown): boolean
    toString(): string
    static DIV0: FormulaError
    static NA: FormulaError
    static NAME: FormulaError
    static NULL: FormulaError
    static NUM: FormulaError
    static REF: FormulaError
    static VALUE: FormulaError
    static ERROR(msg: string, details?: unknown): FormulaError
  }

  export interface FormulaParserConfig {
    onCell?: (ref: CellRef) => CellValue
    onRange?: (ref: RangeRef) => CellValue[][]
    onVariable?: (name: string, sheet?: string) => unknown
    functions?: Record<string, (...args: unknown[]) => unknown>
    functionsNeedContext?: Record<string, (...args: unknown[]) => unknown>
  }

  /** The synchronous Excel-grammar evaluator. Pass the formula body WITHOUT the
   *  leading `=`. Returns the evaluated scalar, or a `FormulaError` for eval-time
   *  errors; THROWS a `FormulaError` for unknown functions / parse failures. */
  export default class FormulaParser {
    constructor(config?: FormulaParserConfig)
    parse(formula: string, position?: Position, allowReturnArray?: boolean): CellValue | CellValue[][] | FormulaError
    supportedFunctions(): string[]
  }

  /** Extracts the dependency set of a formula (the cells/ranges it references)
   *  WITHOUT evaluating. `parse` returns an array of `CellRef | RangeRef`. */
  export class DepParser {
    constructor(config?: { onVariable?: (name: string, sheet?: string) => unknown })
    parse(formula: string, position?: Position, ignoreError?: boolean): Array<CellRef | RangeRef>
  }
}
