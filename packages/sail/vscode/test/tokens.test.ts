import { describe, expect, test } from 'bun:test'
import { SCOPE_TO_LSP, type HighlightSpan } from '@ship/sail-highlight'
import {
  LEGEND_TOKEN_TYPES,
  buildSemanticTokens,
  encodeTokenData,
  tokenTypeIndex,
} from '../src/tokens'

describe('semantic token builder', () => {
  test('fake spans encode to the SCOPE_TO_LSP legend types', () => {
    const src = 'style card\n'
    const spans: HighlightSpan[] = [
      { start: 0, end: 5, scope: 'keyword' },
      { start: 6, end: 10, scope: 'function' },
    ]
    const tokens = buildSemanticTokens(src, spans)
    expect(tokens).toHaveLength(2)

    expect(LEGEND_TOKEN_TYPES[tokens[0]!.typeIndex]).toBe(SCOPE_TO_LSP.keyword)
    expect(LEGEND_TOKEN_TYPES[tokens[1]!.typeIndex]).toBe(SCOPE_TO_LSP.function)
    expect(tokens[0]!.typeIndex).toBe(tokenTypeIndex('keyword'))
    expect(tokens[1]!.typeIndex).toBe(tokenTypeIndex('function'))
    expect(tokens[0]).toMatchObject({ line: 0, startChar: 0, length: 5, tokenModifiers: 0 })
    expect(tokens[1]).toMatchObject({ line: 0, startChar: 6, length: 4, tokenModifiers: 0 })

    const data = encodeTokenData(tokens)
    expect(data).toEqual([
      0,
      0,
      5,
      tokens[0]!.typeIndex,
      0,
      0,
      6,
      4,
      tokens[1]!.typeIndex,
      0,
    ])
  })

  test('keyword.directive maps to macro; punctuation shares operator', () => {
    const src = '<scope>'
    const spans: HighlightSpan[] = [
      { start: 0, end: 1, scope: 'punctuation.bracket' },
      { start: 1, end: 6, scope: 'keyword.directive' },
      { start: 6, end: 7, scope: 'punctuation.bracket' },
    ]
    const tokens = buildSemanticTokens(src, spans)
    expect(LEGEND_TOKEN_TYPES[tokens[0]!.typeIndex]).toBe('operator')
    expect(LEGEND_TOKEN_TYPES[tokens[1]!.typeIndex]).toBe('macro')
    expect(tokens[0]!.typeIndex).toBe(tokens[2]!.typeIndex)
    expect(tokenTypeIndex('punctuation.bracket')).toBe(tokenTypeIndex('punctuation.delimiter'))
  })
})
