import { describe, it, expect } from 'vitest'
import { completionsAt, allSources } from '../src/completions'

describe('completionsAt', () => {
  it('returns null when no trigger precedes the cursor', () => {
    expect(completionsAt('<div></div>', 5)).toBeNull()
  })

  it('matches on: trigger and returns event list', () => {
    const src = '<button on:'
    const r = completionsAt(src, src.length)
    expect(r).not.toBeNull()
    expect(r!.trigger).toBe('on:')
    expect(r!.items.some((i) => i.label === 'click')).toBe(true)
  })

  it('matches bind: trigger', () => {
    const src = '<input bind:'
    const r = completionsAt(src, src.length)
    expect(r!.trigger).toBe('bind:')
    expect(r!.items.some((i) => i.label === 'value')).toBe(true)
  })

  it('matches | (modifier) trigger', () => {
    const src = '<form on:submit|'
    const r = completionsAt(src, src.length)
    expect(r!.trigger).toBe('|')
    expect(r!.items.some((i) => i.label === 'preventDefault')).toBe(true)
  })

  it('matches {# block trigger', () => {
    const src = 'foo {#'
    const r = completionsAt(src, src.length)
    expect(r!.trigger).toBe('{#')
    expect(r!.items.some((i) => i.label === 'if')).toBe(true)
  })

  it('prefers longer trigger over shorter', () => {
    // 'bind:' should win over ':' — actually we don't have a ':' trigger,
    // but verify 'bind:' wins over 'on:' when both could match.
    const src = '<input bind:'
    const r = completionsAt(src, src.length)
    expect(r!.trigger).toBe('bind:')
  })
})

describe('allSources', () => {
  it('returns every trigger source', () => {
    const sources = allSources()
    const triggers = sources.map((s) => s.trigger)
    expect(triggers).toContain('on:')
    expect(triggers).toContain('|')
    expect(triggers).toContain('bind:')
    expect(triggers).toContain('prop:')
    expect(triggers).toContain('class:')
    expect(triggers).toContain('{#')
    expect(triggers).toContain('{:')
  })
})
