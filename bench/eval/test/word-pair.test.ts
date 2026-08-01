import { describe, expect, it } from 'vitest'
import { allocateWordPair, isWordPairId } from '../src/naming/word-pair.ts'

describe('word-pair ids', () => {
  it('allocates adj-noun shape', () => {
    const id = allocateWordPair({ seed: 'test-1' })
    expect(isWordPairId(id)).toBe(true)
    expect(id).toMatch(/^[a-z]+-[a-z]+$/)
  })

  it('avoids taken pairs', () => {
    const taken = new Set<string>()
    const a = allocateWordPair({ seed: 'x', taken })
    const b = allocateWordPair({ seed: 'x', taken })
    expect(a).not.toBe(b)
    expect(taken.has(a)).toBe(true)
    expect(taken.has(b)).toBe(true)
  })

  it('rejects uuid / timestamp shapes', () => {
    expect(isWordPairId('550e8400-e29b-41d4-a716-446655440000')).toBe(false)
    expect(isWordPairId('20260801-120000-smoke')).toBe(false)
    expect(isWordPairId('calm-otter')).toBe(true)
    expect(isWordPairId('calm-otter-2')).toBe(true)
  })
})
