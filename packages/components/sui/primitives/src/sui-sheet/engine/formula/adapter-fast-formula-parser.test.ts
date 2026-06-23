// sui-sheet FORMULA slice (Slice 2 + 6a): the FormulaCore over `fast-formula-parser`.
//
// Authority: build-spec §"The formula-core boundary" ("`createFastFormulaCore()`
// maps FFP ↔ FormulaCore, opaque" — the ONLY file that imports the parser),
// §Slice 2 ("`createFastFormulaCore()` maps `fast-formula-parser` tokens/errors ↔
// `FormulaCore` (`#DIV/0!`, `#REF!`, etc.); `parse()` returns the dep set the DAG
// needs"), §Slice 6a (`fast-formula-parser`, MIT), Behavioral spec
// #2 ("=SUM(A1:A3) evaluates a range"), #3 ("formula errors are per-cell"), #16
// ("swapping the injected formulaCore … changes evaluation without touching the
// engine").
//
// This core is the REAL evaluator (function library: SUM/IF/etc.) over the
// `fast-formula-parser` npm dep (a `devDependency` the bundler inlines at build).
// The adapter is the ONLY seam that imports the parser — the engine binds to the
// `FormulaCore` interface (the purity gate in `purity.test.ts` asserts no other
// engine file imports `fast-formula-parser`).
//
// Engine purity (Cardinal 4): imports ONLY @ship/sui-runtime, OUR contract, and
// `fast-formula-parser` (the one sanctioned import site).

import { describe, expect, it } from 'vitest'
import { createFastFormulaCore } from './adapter-fast-formula-parser'
import type { FormulaCore } from '../contract'
import type { CellAddress, ScalarValue } from '../types'

const ORIGIN: CellAddress = { row: 0, col: 0 }

function getterFor(cells: Record<string, ScalarValue>) {
  return (a: CellAddress): ScalarValue => {
    const col = String.fromCharCode(65 + a.col)
    const a1 = `${col}${a.row + 1}`
    return cells[a1] ?? null
  }
}

describe('createFastFormulaCore — implements the FormulaCore contract', () => {
  it('returns a FormulaCore exposing parse() + supportedFunctions()', () => {
    const core: FormulaCore = createFastFormulaCore()
    expect(typeof core.parse).toBe('function')
    expect(typeof core.supportedFunctions).toBe('function')
  })

  it('supportedFunctions() surfaces the vendor function library (for autocomplete)', () => {
    const core = createFastFormulaCore()
    const fns = core.supportedFunctions()
    expect(fns.length).toBeGreaterThan(0)
    const sum = fns.find((f) => f.name === 'SUM')
    expect(sum).toBeDefined()
    expect(Array.isArray(sum!.arity)).toBe(true)
  })
})

describe('createFastFormulaCore — parse() returns the dep set the DAG consumes', () => {
  it('=A1+B2 surfaces both cell deps', () => {
    const core = createFastFormulaCore()
    const parsed = core.parse('=A1+B2', ORIGIN)
    expect(parsed.deps).toContainEqual({ row: 0, col: 0 })
    expect(parsed.deps).toContainEqual({ row: 1, col: 1 })
  })

  it('=SUM(A1:A3) surfaces a RANGE dep (not three separate cells)', () => {
    const core = createFastFormulaCore()
    const parsed = core.parse('=SUM(A1:A3)', ORIGIN)
    expect(parsed.deps).toContainEqual({
      range: [
        { row: 0, col: 0 },
        { row: 2, col: 0 },
      ],
    })
  })
})

describe('createFastFormulaCore — evaluate() over the vendor function library', () => {
  it('arithmetic over cell refs', () => {
    const core = createFastFormulaCore()
    const parsed = core.parse('=A1+B2', ORIGIN)
    expect(parsed.evaluate(getterFor({ A1: 10, B2: 5 }))).toBe(15)
  })

  it('=SUM(A1:A3) sums the range', () => {
    const core = createFastFormulaCore()
    const parsed = core.parse('=SUM(A1:A3)', ORIGIN)
    expect(parsed.evaluate(getterFor({ A1: 1, A2: 2, A3: 3 }))).toBe(6)
  })

  it('=IF(A1>0, "pos", "neg") branches on the cell value', () => {
    const core = createFastFormulaCore()
    const parsed = core.parse('=IF(A1>0,"pos","neg")', ORIGIN)
    expect(parsed.evaluate(getterFor({ A1: 5 }))).toBe('pos')
    expect(parsed.evaluate(getterFor({ A1: -1 }))).toBe('neg')
  })

  it('string concatenation (=A1&B1)', () => {
    const core = createFastFormulaCore()
    const parsed = core.parse('=A1&B1', ORIGIN)
    expect(parsed.evaluate(getterFor({ A1: 'foo', B1: 'bar' }))).toBe('foobar')
  })
})

describe('createFastFormulaCore — vendor errors map to OUR FormulaError vocabulary', () => {
  it('division by zero → #DIV/0!', () => {
    const core = createFastFormulaCore()
    const parsed = core.parse('=1/0', ORIGIN)
    expect(parsed.evaluate(getterFor({}))).toEqual({ error: '#DIV/0!' })
  })

  it('an unknown function → #NAME?', () => {
    const core = createFastFormulaCore()
    const parsed = core.parse('=NOTAFUNC()', ORIGIN)
    expect(parsed.evaluate(getterFor({}))).toEqual({ error: '#NAME?' })
  })

  it('a #VALUE! type error surfaces as #VALUE!', () => {
    const core = createFastFormulaCore()
    // adding a non-numeric string is a value error in the vendor parser.
    const parsed = core.parse('=A1+1', ORIGIN)
    expect(parsed.evaluate(getterFor({ A1: 'text' }))).toEqual({ error: '#VALUE!' })
  })
})
