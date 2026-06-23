// TESTS-FIRST — sui-sheet FORMULA slice (Slice 2): the noop FormulaCore adapter.
//
// Status: RED. `./adapter-noop` does not exist yet. The import is import-not-found
// until the formula slice ships the literal-only stub core. DO NOT add the impl
// here. DO NOT weaken into passing stubs.
//
// Authority: build-spec §"The formula-core vendor boundary" (the `FormulaCore`
// interface — `parse(formula, origin) → { deps, evaluate(getCell) }` +
// `supportedFunctions()`), §Slice 2 ("`engine/formula/adapter-noop.ts` (a
// literal-only stub so Slice 3 can test the DAG without the vendor) … the noop
// adapter evaluates literals + returns `#NAME?` for any function (so the engine
// boundary is testable without the vendor)"), §"FormulaCore vendor boundary"
// (Behavioral spec #16 — swapping the injected core changes evaluation without
// touching the engine).
//
// WHY a noop core: the DAG slice (Slice 3) must be testable WITHOUT the vendored
// parser. The noop core is the literal-only `FormulaCore` — it parses a formula's
// cell references into the dep set the DAG needs and evaluates arithmetic over
// plain cell refs / numeric literals, but returns `#NAME?` for any function call.
// It is the deterministic, vendor-free implementation of OUR `FormulaCore`
// contract — the seam the engine binds to (never the vendor API).
//
// Engine purity (Cardinal 4): imports ONLY @ship/sui-runtime + OUR contract.

import { describe, expect, it } from 'vitest'
// RED: greenfield module — the noop adapter lands with the formula slice.
import { createNoopFormulaCore } from './adapter-noop'
import type { FormulaCore } from '../contract'
import type { CellAddress, ScalarValue } from '../types'

const ORIGIN: CellAddress = { row: 0, col: 0 }

/** A getCell backed by a plain A1→value map (the DAG hands the engine one of
 *  these; here we fake it to exercise the core in isolation). */
function getterFor(cells: Record<string, ScalarValue>) {
  return (a: CellAddress): ScalarValue => {
    const col = String.fromCharCode(65 + a.col)
    const a1 = `${col}${a.row + 1}`
    return cells[a1] ?? null
  }
}

describe('createNoopFormulaCore — implements the FormulaCore contract', () => {
  it('returns an object exposing parse() and supportedFunctions()', () => {
    const core: FormulaCore = createNoopFormulaCore()
    expect(typeof core.parse).toBe('function')
    expect(typeof core.supportedFunctions).toBe('function')
  })

  it('supportedFunctions() is empty — the noop core has no function library', () => {
    const core = createNoopFormulaCore()
    expect(core.supportedFunctions()).toEqual([])
  })
})

describe('createNoopFormulaCore — parse() surfaces the dependency set (for the DAG)', () => {
  it('=A1 yields A1 as the single dep', () => {
    const core = createNoopFormulaCore()
    const parsed = core.parse('=A1', ORIGIN)
    expect(parsed.deps).toEqual([{ row: 0, col: 0 }])
  })

  it('=A1+B2 yields both cell deps', () => {
    const core = createNoopFormulaCore()
    const parsed = core.parse('=A1+B2', ORIGIN)
    expect(parsed.deps).toContainEqual({ row: 0, col: 0 })
    expect(parsed.deps).toContainEqual({ row: 1, col: 1 })
  })

  it('a literal-only formula (=2+3) yields NO deps', () => {
    const core = createNoopFormulaCore()
    const parsed = core.parse('=2+3', ORIGIN)
    expect(parsed.deps).toEqual([])
  })

  it('a range reference (=A1:A3) surfaces a range dep the DAG can expand', () => {
    const core = createNoopFormulaCore()
    const parsed = core.parse('=A1:A3', ORIGIN)
    expect(parsed.deps).toContainEqual({
      range: [
        { row: 0, col: 0 },
        { row: 2, col: 0 },
      ],
    })
  })
})

describe('createNoopFormulaCore — evaluate() over literals + plain cell refs', () => {
  it('evaluates a numeric literal expression', () => {
    const core = createNoopFormulaCore()
    const parsed = core.parse('=2+3', ORIGIN)
    expect(parsed.evaluate(getterFor({}))).toBe(5)
  })

  it('evaluates a single cell reference to that cell value', () => {
    const core = createNoopFormulaCore()
    const parsed = core.parse('=A1', ORIGIN)
    expect(parsed.evaluate(getterFor({ A1: 7 }))).toBe(7)
  })

  it('evaluates arithmetic over cell refs (=A1+B2)', () => {
    const core = createNoopFormulaCore()
    const parsed = core.parse('=A1+B2', ORIGIN)
    expect(parsed.evaluate(getterFor({ A1: 10, B2: 5 }))).toBe(15)
  })

  it('a reference to an empty cell coerces to 0 in arithmetic', () => {
    const core = createNoopFormulaCore()
    const parsed = core.parse('=A1+1', ORIGIN)
    expect(parsed.evaluate(getterFor({}))).toBe(1)
  })
})

describe('createNoopFormulaCore — error vocabulary (the contract surface)', () => {
  it('any function call returns #NAME? (the noop core has no functions)', () => {
    const core = createNoopFormulaCore()
    const parsed = core.parse('=SUM(A1:A3)', ORIGIN)
    expect(parsed.evaluate(getterFor({ A1: 1, A2: 2, A3: 3 }))).toEqual({ error: '#NAME?' })
  })

  it('division by zero yields #DIV/0!', () => {
    const core = createNoopFormulaCore()
    const parsed = core.parse('=1/0', ORIGIN)
    expect(parsed.evaluate(getterFor({}))).toEqual({ error: '#DIV/0!' })
  })
})
