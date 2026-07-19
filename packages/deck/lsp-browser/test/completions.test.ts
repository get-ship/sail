import { describe, it, expect, beforeAll } from 'vitest'
import { allSources, completionsAt } from '../src/completions'

beforeAll(async () => {
  // The module initializes WASM at the top level; waiting here ensures
  // vocabulary is loaded before any assertion runs.
  await new Promise((resolve) => setTimeout(resolve, 0))
})

describe('completionsAt', () => {
  it('returns null when no trigger precedes the cursor', () => {
    expect(completionsAt('<div></div>', 5)).toBeNull()
  })

  it('matches on: trigger and returns event list from the compiler', () => {
    const src = '<button on:'
    const r = completionsAt(src, src.length)
    expect(r).not.toBeNull()
    expect(r!.trigger).toBe('on:')
    expect(r!.items.some((i) => i.label === 'click')).toBe(true)
    // focusin is a delegatable event the old hand-coded list omitted.
    expect(r!.items.some((i) => i.label === 'focusin')).toBe(true)
  })

  it('matches bind: trigger from compiler bind targets', () => {
    const src = '<input bind:'
    const r = completionsAt(src, src.length)
    expect(r!.trigger).toBe('bind:')
    expect(r!.items.some((i) => i.label === 'value')).toBe(true)
  })

  it('matches | (modifier) trigger and includes aliases', () => {
    const src = '<form on:submit|'
    const r = completionsAt(src, src.length)
    expect(r!.trigger).toBe('|')
    expect(r!.items.some((i) => i.label === 'preventDefault')).toBe(true)
    // Drift: the old TS list omitted the `prevent` and `stop` aliases.
    expect(r!.items.some((i) => i.label === 'prevent')).toBe(true)
    expect(r!.items.some((i) => i.label === 'stop')).toBe(true)
  })

  it('matches {# block trigger from compiler block manifest', () => {
    const src = 'foo {#'
    const r = completionsAt(src, src.length)
    expect(r!.trigger).toBe('{#')
    expect(r!.items.some((i) => i.label === 'if')).toBe(true)
    // Drift: the old TS list omitted `{#match}`.
    expect(r!.items.some((i) => i.label === 'match')).toBe(true)
  })

  it('matches {: clause trigger and surfaces match clauses', () => {
    const src = '{#match x}{:'
    const r = completionsAt(src, src.length)
    expect(r!.trigger).toBe('{:')
    expect(r!.items.some((i) => i.label === 'else')).toBe(true)
    // Drift: the old TS list omitted `case` and `default` clauses.
    expect(r!.items.some((i) => i.label === 'case')).toBe(true)
    expect(r!.items.some((i) => i.label === 'default')).toBe(true)
  })

  it('prefers longer trigger over shorter', () => {
    const src = '<input bind:'
    const r = completionsAt(src, src.length)
    expect(r!.trigger).toBe('bind:')
  })
})

describe('allSources', () => {
  it('returns every trigger source backed by the compiler', () => {
    const sources = allSources()
    const triggers = sources.map((s) => s.trigger)
    expect(triggers).toContain('on:')
    expect(triggers).toContain('|')
    expect(triggers).toContain('bind:')
    expect(triggers).toContain('{#')
    expect(triggers).toContain('{:')
  })
})
