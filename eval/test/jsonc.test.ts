import { describe, expect, it } from 'vitest'
import { parseJsonc, stripJsonc } from '../src/jsonc.ts'

describe('jsonc', () => {
  it('strips line and block comments', () => {
    const raw = `{
      // line comment
      "a": 1, /* block */
      "b": "http://example.com"
    }`
    expect(parseJsonc<{ a: number; b: string }>(raw)).toEqual({
      a: 1,
      b: 'http://example.com',
    })
  })

  it('strips trailing commas', () => {
    const raw = `{ "x": [1, 2,], "y": { "z": true, }, }`
    expect(parseJsonc(raw)).toEqual({ x: [1, 2], y: { z: true } })
  })

  it('does not strip comment-like text inside strings', () => {
    const raw = `{ "msg": "use // carefully /* ok */" }`
    expect(parseJsonc<{ msg: string }>(raw).msg).toBe('use // carefully /* ok */')
  })

  it('throws on invalid JSON after strip', () => {
    expect(() => parseJsonc('{ not json')).toThrow()
  })

  it('stripJsonc preserves newlines from line comments', () => {
    const s = stripJsonc('a//c\nb')
    expect(s.includes('b')).toBe(true)
  })
})
