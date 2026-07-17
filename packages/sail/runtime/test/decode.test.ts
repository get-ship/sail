import { describe, expect, it } from 'vitest'
import {
  decodeString,
  decodeNumber,
  decodeBoolean,
  decodeJson,
  decodeEnum,
} from '../src/decode'

describe('decodeString', () => {
  it('passes through', () => expect(decodeString('hi')).toBe('hi'))
  it('null → empty', () => expect(decodeString(null)).toBe(''))
})

describe('decodeNumber', () => {
  it('parses integer', () => expect(decodeNumber('42')).toBe(42))
  it('parses float', () => expect(decodeNumber('3.14')).toBe(3.14))
  it('null → NaN', () => expect(decodeNumber(null)).toBeNaN())
  it('empty → NaN', () => expect(decodeNumber('')).toBeNaN())
  it('garbage → NaN', () => expect(decodeNumber('xyz')).toBeNaN())
})

describe('decodeBoolean', () => {
  it('null → false', () => expect(decodeBoolean(null)).toBe(false))
  it('"" → true (presence)', () => expect(decodeBoolean('')).toBe(true))
  it('"true" → true', () => expect(decodeBoolean('true')).toBe(true))
  it('"false" → false', () => expect(decodeBoolean('false')).toBe(false))
  it('"FALSE" → false (case-insensitive)', () => expect(decodeBoolean('FALSE')).toBe(false))
  it('arbitrary value → true', () => expect(decodeBoolean('yes')).toBe(true))
})

describe('decodeJson', () => {
  it('parses object', () => {
    expect(decodeJson<unknown>(null)('{"a":1}')).toEqual({ a: 1 })
  })
  it('parses array', () => {
    expect(decodeJson<unknown[]>([])('[1,2,3]')).toEqual([1, 2, 3])
  })
  it('null → fallback', () => {
    expect(decodeJson<unknown[]>([])(null)).toEqual([])
  })
  it('parse error → fallback', () => {
    const fb = { ok: true }
    expect(decodeJson(fb)('not json')).toBe(fb)
  })
})

describe('decodeEnum', () => {
  const dec = decodeEnum(['primary', 'ghost'] as const, 'primary')
  it('allowed value', () => expect(dec('ghost')).toBe('ghost'))
  it('disallowed → fallback', () => expect(dec('weird')).toBe('primary'))
  it('null → fallback', () => expect(dec(null)).toBe('primary'))
})
