// primitive-adherence.test.ts — the grader's contract, made a check.
//
// Run: `bun test eval/scripts/primitive-adherence.test.ts`
//
// The two required fixtures: a `.sui` that hand-rolls layout scores LOW, one
// that composes primitives + tokens scores HIGH — with a wide margin between
// them. Plus a handful of unit checks on the pure scorer so a regression in
// one detector is localised.

import { test, expect } from 'bun:test'
import { readFileSync } from 'node:fs'
import { join } from 'node:path'
import { scorePrimitiveAdherence, gradePrimitiveAdherence } from './primitive-adherence'

const FIXTURES = join(import.meta.dir, 'fixtures/primitive-adherence')
const read = (name: string) => readFileSync(join(FIXTURES, name), 'utf8')

test('hand-rolled layout fixture scores LOW', () => {
  const s = scorePrimitiveAdherence(read('adhoc.sui'))
  expect(s.adHocLayout).toBeGreaterThan(0)
  expect(s.literalValues).toBeGreaterThan(0)
  expect(s.primitiveUses).toBe(0)
  expect(s.score).toBeLessThan(0.3)
})

test('composed primitives + tokens fixture scores HIGH', () => {
  const s = scorePrimitiveAdherence(read('composed.sui'))
  expect(s.primitiveUses).toBeGreaterThan(0)
  expect(s.tokenRefs).toBeGreaterThan(0)
  expect(s.adHocLayout).toBe(0)
  expect(s.literalValues).toBe(0)
  expect(s.score).toBeGreaterThan(0.85)
})

test('composed scores strictly, decisively higher than hand-rolled', () => {
  const low = scorePrimitiveAdherence(read('adhoc.sui')).score
  const high = scorePrimitiveAdherence(read('composed.sui')).score
  expect(high - low).toBeGreaterThan(0.6)
})

test('gradePrimitiveAdherence reports per-file + an overall mean', () => {
  const report = gradePrimitiveAdherence([
    { file: 'adhoc.sui', text: read('adhoc.sui') },
    { file: 'composed.sui', text: read('composed.sui') },
  ])
  expect(report.perFile).toHaveLength(2)
  const mean = report.perFile.reduce((s, f) => s + f.score, 0) / 2
  expect(report.overall).toBeCloseTo(mean, 10)
})

// --- unit checks on the pure detectors -------------------------------------

test(':host display:flex is exempt (host root layout, not a hand-rolled stack)', () => {
  const src = '<style>:host { display: flex; flex-direction: column; }</style>'
  expect(scorePrimitiveAdherence(src).adHocLayout).toBe(0)
})

test('display:grid / display:flex off :host are penalised', () => {
  const src = '<style>.a { display: grid; } .b { display: inline-flex; }</style>'
  expect(scorePrimitiveAdherence(src).adHocLayout).toBe(2)
})

test('hex, px, and a raw font-family are each literal-value penalties', () => {
  const src = '<style>.a { color: #abc; padding: 12px; font-family: "Inter", sans-serif; }</style>'
  expect(scorePrimitiveAdherence(src).literalValues).toBe(3)
})

test('font-family that defers to a role token is NOT a literal', () => {
  const src = '<style>.a { font-family: var(--body-font); }</style>'
  expect(scorePrimitiveAdherence(src).literalValues).toBe(0)
})

test('a file with nothing to grade is vacuously clean (score 1)', () => {
  expect(scorePrimitiveAdherence('<p>hello</p>').score).toBe(1)
})

test('HTML comments never move the score', () => {
  const commented = '<!-- <sui-grid> and var(--space-4) mentioned here -->\n<p>hi</p>'
  const s = scorePrimitiveAdherence(commented)
  expect(s.primitiveUses).toBe(0)
  expect(s.tokenRefs).toBe(0)
})
