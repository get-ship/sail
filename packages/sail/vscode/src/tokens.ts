/**
 * Map @ship/sail-highlight spans onto VS Code semantic-token encoding.
 * Pure — no vscode module — so bun test can check legend types.
 */
import {
  SCOPE_TO_LSP,
  SCOPES,
  isSailScope,
  type HighlightSpan,
  type SailScope,
} from '@ship/sail-highlight'

/** Unique LSP token types in SCOPES order (first seen). */
export const LEGEND_TOKEN_TYPES: string[] = []
for (const scope of SCOPES) {
  const t = SCOPE_TO_LSP[scope]
  if (!LEGEND_TOKEN_TYPES.includes(t)) LEGEND_TOKEN_TYPES.push(t)
}

export const LEGEND_TOKEN_MODIFIERS: string[] = []

export interface EncodedToken {
  line: number
  startChar: number
  length: number
  typeIndex: number
  tokenModifiers: number
}

export function tokenTypeIndex(scope: SailScope): number {
  return LEGEND_TOKEN_TYPES.indexOf(SCOPE_TO_LSP[scope])
}

export function lspTypeForScope(scope: SailScope): string {
  return SCOPE_TO_LSP[scope]
}

/** Convert JS-string offsets to 0-based line / UTF-16 column. */
export function indexToLineChar(src: string, index: number): { line: number; char: number } {
  let line = 0
  let char = 0
  const n = Math.min(Math.max(index, 0), src.length)
  for (let i = 0; i < n; i++) {
    if (src.charCodeAt(i) === 10) {
      line++
      char = 0
    } else {
      char++
    }
  }
  return { line, char }
}

/** One visual token per line — VS Code tokens must not cross newlines. */
export function splitSpanLines(
  src: string,
  start: number,
  end: number,
): Array<{ line: number; startChar: number; length: number }> {
  const lo = Math.max(0, Math.min(start, src.length))
  const hi = Math.max(lo, Math.min(end, src.length))
  if (hi <= lo) return []
  const out: Array<{ line: number; startChar: number; length: number }> = []
  const origin = indexToLineChar(src, lo)
  let line = origin.line
  let startChar = origin.char
  let sliceStart = lo
  for (let i = lo; i < hi; i++) {
    if (src.charCodeAt(i) === 10) {
      if (i > sliceStart) {
        out.push({ line, startChar, length: i - sliceStart })
      }
      line++
      startChar = 0
      sliceStart = i + 1
    }
  }
  if (hi > sliceStart) {
    out.push({ line, startChar, length: hi - sliceStart })
  }
  return out
}

export function buildSemanticTokens(src: string, spans: readonly HighlightSpan[]): EncodedToken[] {
  const tokens: EncodedToken[] = []
  for (const span of spans) {
    if (!isSailScope(span.scope)) continue
    const typeIndex = tokenTypeIndex(span.scope)
    if (typeIndex < 0) continue
    for (const slice of splitSpanLines(src, span.start, span.end)) {
      if (slice.length <= 0) continue
      tokens.push({
        line: slice.line,
        startChar: slice.startChar,
        length: slice.length,
        typeIndex,
        tokenModifiers: 0,
      })
    }
  }
  tokens.sort((a, b) => a.line - b.line || a.startChar - b.startChar)
  return tokens
}

/** LSP / VS Code delta encoding: [deltaLine, deltaStart, length, type, mods] × N */
export function encodeTokenData(tokens: readonly EncodedToken[]): number[] {
  const data: number[] = []
  let prevLine = 0
  let prevChar = 0
  for (const t of tokens) {
    const deltaLine = t.line - prevLine
    const deltaStart = deltaLine === 0 ? t.startChar - prevChar : t.startChar
    data.push(deltaLine, deltaStart, t.length, t.typeIndex, t.tokenModifiers)
    prevLine = t.line
    prevChar = t.startChar
  }
  return data
}
