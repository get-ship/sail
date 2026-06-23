// TESTS-FIRST — sui-sheet FORMULA slice (Slice 3): the cell dependency DAG.
//
// Status: RED. `./dag` does not exist yet. The import is import-not-found until
// the formula slice ships the dependency graph + topo recalc order. DO NOT add
// the impl here. DO NOT weaken into passing stubs.
//
// Authority: build-spec §Features #2 ("Cell DAG + recalc — FormulaCore.parse
// yields each formula's dep set; the engine builds a dependency graph, computes
// topo order, and on a cell write recalcs the dirty SUFFIX (the cell + its
// transitive dependents) — not the whole sheet. Cycle detection marks #CYCLE!"),
// §"Authoritative … DAG API" ("dependents(a1), precedents(a1), hasCycle — built
// at parse time from FormulaCore.parse().deps; recalc walks topo order; a
// back-edge marks the participating cells #CYCLE!"), §Slice 3 ("building the DAG
// from parse().deps; topo recalc walks the dirty suffix only … #CYCLE! on a
// back-edge").
//
// The DAG is a PURE graph module — no signals, no eval. It owns: register a
// formula's deps → edges; precedents/dependents lookup; topo order; the dirty
// suffix (a cell + its transitive dependents) in topo order; cycle detection
// (the back-edge set). The recalc/eval wiring is `recalc.ts` + `create-sheet.ts`
// (separate tests). Here we pin the GRAPH algebra in isolation.
//
// Engine purity (Cardinal 4): a pure data-structure module — no runtime imports.

import { describe, expect, it } from 'vitest'
// RED: greenfield module — the DAG lands with the formula slice.
import { createDag } from './dag'

describe('createDag — register a formula, read precedents/dependents', () => {
  it('B1 = A1+1 records A1 as a precedent of B1, B1 as a dependent of A1', () => {
    const dag = createDag()
    dag.setFormula('B1', ['A1'])
    expect(dag.precedents('B1')).toEqual(['A1'])
    expect(dag.dependents('A1')).toEqual(['B1'])
  })

  it('a cell with no formula has no precedents', () => {
    const dag = createDag()
    expect(dag.precedents('A1')).toEqual([])
  })

  it('clearing a formula removes its edges', () => {
    const dag = createDag()
    dag.setFormula('B1', ['A1'])
    dag.clearFormula('B1')
    expect(dag.precedents('B1')).toEqual([])
    expect(dag.dependents('A1')).toEqual([])
  })

  it('re-registering a formula REPLACES its precedent set (no stale edges)', () => {
    const dag = createDag()
    dag.setFormula('C1', ['A1', 'B1'])
    dag.setFormula('C1', ['A1']) // B1 dropped
    expect(dag.precedents('C1')).toEqual(['A1'])
    expect(dag.dependents('B1')).toEqual([])
  })
})

describe('createDag — transitive dependents (the dirty suffix source)', () => {
  it('A1 → B1 → C1: dependents(A1) reaches C1 transitively', () => {
    const dag = createDag()
    dag.setFormula('B1', ['A1'])
    dag.setFormula('C1', ['B1'])
    const suffix = dag.dirtySuffix('A1')
    expect(suffix).toContain('B1')
    expect(suffix).toContain('C1')
  })

  it('the dirty suffix is in topological order (a precedent precedes its dependent)', () => {
    const dag = createDag()
    dag.setFormula('B1', ['A1'])
    dag.setFormula('C1', ['B1'])
    const suffix = dag.dirtySuffix('A1')
    expect(suffix.indexOf('B1')).toBeLessThan(suffix.indexOf('C1'))
  })

  it('the dirty suffix excludes unrelated cells (the surgical-recalc guarantee)', () => {
    const dag = createDag()
    dag.setFormula('B1', ['A1'])
    dag.setFormula('Z9', ['Y9']) // a disjoint chain
    const suffix = dag.dirtySuffix('A1')
    expect(suffix).not.toContain('Z9')
  })

  it('a diamond (A1→B1, A1→C1, B1→D1, C1→D1) recomputes D1 exactly once', () => {
    const dag = createDag()
    dag.setFormula('B1', ['A1'])
    dag.setFormula('C1', ['A1'])
    dag.setFormula('D1', ['B1', 'C1'])
    const suffix = dag.dirtySuffix('A1')
    expect(suffix.filter((a) => a === 'D1').length).toBe(1)
    // D1 still comes after BOTH its precedents.
    expect(suffix.indexOf('D1')).toBeGreaterThan(suffix.indexOf('B1'))
    expect(suffix.indexOf('D1')).toBeGreaterThan(suffix.indexOf('C1'))
  })
})

describe('createDag — cycle detection (back-edge marking)', () => {
  it('a self-reference (A1 = A1) is a cycle', () => {
    const dag = createDag()
    dag.setFormula('A1', ['A1'])
    expect(dag.hasCycle()).toBe(true)
    expect(dag.cycleCells()).toContain('A1')
  })

  it('a two-cell cycle (A1 = B1, B1 = A1) marks BOTH participants', () => {
    const dag = createDag()
    dag.setFormula('A1', ['B1'])
    dag.setFormula('B1', ['A1'])
    expect(dag.hasCycle()).toBe(true)
    expect(dag.cycleCells()).toEqual(expect.arrayContaining(['A1', 'B1']))
  })

  it('an acyclic graph reports NO cycle', () => {
    const dag = createDag()
    dag.setFormula('B1', ['A1'])
    dag.setFormula('C1', ['B1'])
    expect(dag.hasCycle()).toBe(false)
    expect(dag.cycleCells()).toEqual([])
  })

  it('breaking the back-edge clears the cycle', () => {
    const dag = createDag()
    dag.setFormula('A1', ['B1'])
    dag.setFormula('B1', ['A1'])
    expect(dag.hasCycle()).toBe(true)
    dag.clearFormula('B1') // break the cycle
    expect(dag.hasCycle()).toBe(false)
  })
})
