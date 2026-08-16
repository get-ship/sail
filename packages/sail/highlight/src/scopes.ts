/**
 * Closed map: tree-sitter capture (no `@`) → LSP semantic token type
 * and CodeMirror CSS class.
 *
 * A capture in highlights.scm that is missing here fails
 * `packages/sail/grammar/tests/highlight.rs` (SCOPES) and
 * `test/scopes-drift.test.ts`.
 */

export const SCOPES = [
  'tag',
  'attribute',
  'string',
  'keyword',
  'keyword.directive',
  'function',
  'variable',
  'comment',
  'punctuation.bracket',
  'punctuation.delimiter',
] as const

export type SailScope = (typeof SCOPES)[number]

/** LSP SemanticTokenTypes names (vscode / helix). */
export const SCOPE_TO_LSP: Record<SailScope, string> = {
  tag: 'type',
  attribute: 'property',
  string: 'string',
  keyword: 'keyword',
  'keyword.directive': 'macro',
  function: 'function',
  variable: 'variable',
  comment: 'comment',
  'punctuation.bracket': 'operator',
  'punctuation.delimiter': 'operator',
}

/** Classes consumed by sail-code-editor theme (`--syntax-*`). */
export const SCOPE_TO_CM_CLASS: Record<SailScope, string> = {
  tag: 'sail-tag',
  attribute: 'sail-attr-name',
  string: 'sail-string',
  keyword: 'sail-kw',
  'keyword.directive': 'sail-attr-prefix',
  function: 'sail-ts-fn',
  variable: 'sail-interp',
  comment: 'sail-ts-cmt',
  'punctuation.bracket': 'sail-interp-brace',
  'punctuation.delimiter': 'sail-attr-modifier',
}

export function isSailScope(name: string): name is SailScope {
  return (SCOPES as readonly string[]).includes(name)
}
