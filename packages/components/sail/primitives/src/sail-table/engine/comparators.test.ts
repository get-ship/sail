// TESTS-FIRST — Backbone #1, Slice 1a: typed comparators.
//
// Status: RED. `./comparators` does not exist yet (engine is greenfield).
// The import below fails to resolve until the impl lands — that import-not-found
// is the expected red for this contract. DO NOT add the impl here.
//
// Authority: build-spec §"REPLACE" (typed comparators replacing `String()`-coerced
// single-column sort), §Features #2 ("Default comparator numeric-aware then locale
// string … Fixes the '10' before '2' bug"), Slice 1a ("typed comparators
// (numeric/date/string detection, stable, null/undefined ordering, tri-state) —
// `'10'` sorts after `'2'`").

import { describe, expect, it } from 'vitest'
// RED: greenfield module — resolves only once Slice 1a ships.
import {
  compareValues,
  detectSortType,
  resolveComparator,
  type SortType,
} from './comparators'

describe('detectSortType', () => {
  it('classifies all-numeric values as numeric', () => {
    const t: SortType = detectSortType([1, 2, 3])
    expect(t).toBe('number')
  })

  it('classifies numeric strings as numeric (so "10" is not lexicographically before "2")', () => {
    expect(detectSortType(['10', '2', '100'])).toBe('number')
  })

  it('classifies ISO date strings as date', () => {
    expect(detectSortType(['2026-01-01', '2025-12-31'])).toBe('date')
  })

  it('falls back to string for mixed/non-numeric values', () => {
    expect(detectSortType(['alpha', 'beta', '3'])).toBe('string')
  })
})

describe('compareValues — numeric awareness (the core bug fix)', () => {
  it("orders numbers 2, 10, 100 — NOT lexicographic 10, 100, 2", () => {
    const sorted = [10, 2, 100].sort((a, b) => compareValues(a, b, 'number'))
    expect(sorted).toEqual([2, 10, 100])
  })

  it("orders numeric STRINGS '2', '10', '100' numerically", () => {
    const sorted = ['10', '2', '100'].sort((a, b) => compareValues(a, b, 'number'))
    expect(sorted).toEqual(['2', '10', '100'])
  })

  it('orders strings by locale collation', () => {
    const sorted = ['Banana', 'apple', 'Cherry'].sort((a, b) =>
      compareValues(a, b, 'string'),
    )
    expect(sorted).toEqual(['apple', 'Banana', 'Cherry'])
  })

  it('orders dates chronologically', () => {
    const sorted = ['2026-03-01', '2025-12-31', '2026-01-15'].sort((a, b) =>
      compareValues(a, b, 'date'),
    )
    expect(sorted).toEqual(['2025-12-31', '2026-01-15', '2026-03-01'])
  })
})

describe('compareValues — null/undefined ordering (explicit contract)', () => {
  it('sorts null/undefined last regardless of direction sign', () => {
    const sorted = [3, null, 1, undefined, 2].sort((a, b) =>
      compareValues(a, b, 'number'),
    )
    // The three present values keep numeric order; the two empties trail.
    expect(sorted.slice(0, 3)).toEqual([1, 2, 3])
    expect(sorted.slice(3).every((v) => v == null)).toBe(true)
  })

  it('treats two empties as equal (returns 0)', () => {
    expect(compareValues(null, undefined, 'number')).toBe(0)
    expect(compareValues(undefined, null, 'string')).toBe(0)
  })
})

describe('resolveComparator', () => {
  it('returns a comparator from a SortType name', () => {
    const cmp = resolveComparator('number')
    expect(cmp(10, 2)).toBeGreaterThan(0)
  })

  it('passes a user-supplied comparator function through unchanged', () => {
    const custom = (a: unknown, b: unknown) => String(a).length - String(b).length
    expect(resolveComparator(custom)).toBe(custom)
  })
})

describe('stable sort guarantee', () => {
  it('preserves the original order of equal-key rows', () => {
    const rows = [
      { id: 'a', k: 1 },
      { id: 'b', k: 1 },
      { id: 'c', k: 1 },
    ]
    const sorted = [...rows].sort((x, y) => compareValues(x.k, y.k, 'number'))
    expect(sorted.map((r) => r.id)).toEqual(['a', 'b', 'c'])
  })
})
