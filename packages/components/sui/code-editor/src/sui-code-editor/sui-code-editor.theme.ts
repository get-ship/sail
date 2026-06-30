/**
 * CodeMirror theme using SUI design tokens.
 *
 * Colors reference CSS custom properties from tokens.css so the theme
 * adapts to the sui document's [data-mode="dark"] / default light axis
 * automatically — no separate editor stylesheet.
 */

import { EditorView } from '@codemirror/view'
import { HighlightStyle, syntaxHighlighting } from '@codemirror/language'
import { tags } from '@lezer/highlight'

export const suiEditorTheme = EditorView.theme({
  '&': {
    backgroundColor: 'var(--surface-1)',
    color: 'var(--foreground)',
    fontSize: 'inherit',
    fontFamily: 'inherit',
  },
  '.cm-content': {
    caretColor: 'var(--foreground)',
    fontFamily: 'inherit',
    padding: '8px 0',
  },
  '.cm-cursor, .cm-dropCursor': {
    borderLeftColor: 'var(--foreground)',
    borderLeftWidth: '2px',
  },
  '&.cm-focused .cm-selectionBackground, .cm-selectionBackground': {
    background: 'color-mix(in oklch, var(--primary) 24%, transparent)',
  },
  '.cm-activeLine': {
    backgroundColor: 'color-mix(in oklch, var(--surface-2) 70%, transparent)',
  },
  '.cm-gutters': {
    backgroundColor: 'var(--surface-2)',
    color: 'var(--muted-foreground)',
    border: 'none',
    borderRight: '1px solid var(--border)',
  },
  '.cm-activeLineGutter': {
    backgroundColor: 'var(--surface-2)',
    color: 'var(--foreground)',
  },
  '.cm-foldPlaceholder': {
    backgroundColor: 'var(--surface-2)',
    border: '1px solid var(--border)',
    color: 'var(--muted-foreground)',
  },
  '.cm-tooltip': {
    backgroundColor: 'var(--surface-1)',
    color: 'var(--foreground)',
    border: '1px solid var(--border)',
    borderRadius: 'var(--radius-md)',
    boxShadow: 'var(--shadow-sm, 0 1px 2px rgba(0,0,0,0.08))',
  },
  '.cm-tooltip-autocomplete': {
    '& > ul > li[aria-selected]': {
      backgroundColor: 'var(--surface-2)',
    },
  },
  '.cm-searchMatch': {
    backgroundColor: 'color-mix(in oklch, var(--primary) 30%, transparent)',
  },
  '.cm-searchMatch.cm-searchMatch-selected': {
    backgroundColor: 'color-mix(in oklch, var(--primary) 50%, transparent)',
  },
  '.cm-placeholder': {
    color: 'var(--muted-foreground)',
    fontStyle: 'italic',
  },
  '&.cm-focused': { outline: 'none' },
  '.cm-scroller': {
    fontFamily: 'inherit',
    lineHeight: 'inherit',
  },
})

// Syntax colors read --syntax-* tokens (tokens.css) so themes that intentionally
// subdue their brand palette keep code legible at AA contrast.
const suiHighlightStyle = HighlightStyle.define([
  { tag: tags.keyword, color: 'var(--syntax-keyword)' },
  { tag: tags.operator, color: 'var(--syntax-keyword)' },
  { tag: tags.controlKeyword, color: 'var(--syntax-keyword)', fontWeight: '500' },
  { tag: tags.definitionKeyword, color: 'var(--syntax-keyword)' },
  { tag: tags.moduleKeyword, color: 'var(--syntax-keyword)' },
  { tag: tags.string, color: 'var(--syntax-string)' },
  { tag: tags.regexp, color: 'var(--syntax-string)' },
  { tag: tags.special(tags.string), color: 'var(--syntax-string)' },
  { tag: tags.number, color: 'var(--syntax-number)' },
  { tag: tags.bool, color: 'var(--syntax-number)' },
  { tag: tags.null, color: 'var(--syntax-number)' },
  { tag: tags.comment, color: 'var(--syntax-comment)', fontStyle: 'italic' },
  { tag: tags.blockComment, color: 'var(--syntax-comment)', fontStyle: 'italic' },
  { tag: tags.lineComment, color: 'var(--syntax-comment)', fontStyle: 'italic' },
  { tag: tags.variableName, color: 'var(--foreground)' },
  { tag: tags.definition(tags.variableName), color: 'var(--foreground)', fontWeight: '500' },
  { tag: tags.function(tags.variableName), color: 'var(--syntax-function)' },
  { tag: tags.typeName, color: 'var(--syntax-function)' },
  { tag: tags.className, color: 'var(--syntax-function)' },
  { tag: tags.propertyName, color: 'var(--foreground)' },
  { tag: tags.definition(tags.propertyName), color: 'var(--foreground)' },
  { tag: tags.atom, color: 'var(--syntax-number)' },
  { tag: tags.meta, color: 'var(--syntax-comment)' },
  { tag: tags.tagName, color: 'var(--syntax-tag)' },
  { tag: tags.attributeName, color: 'var(--syntax-attr)' },
  { tag: tags.attributeValue, color: 'var(--syntax-string)' },
  { tag: tags.heading, color: 'var(--syntax-keyword)', fontWeight: '600' },
  { tag: tags.emphasis, fontStyle: 'italic' },
  { tag: tags.strong, fontWeight: '600' },
  { tag: tags.link, color: 'var(--syntax-number)', textDecoration: 'underline' },
  { tag: tags.invalid, color: 'var(--destructive)' },
])

export const suiHighlighting = syntaxHighlighting(suiHighlightStyle)
