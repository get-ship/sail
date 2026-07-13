/**
 * CodeMirror theme using Ship design tokens.
 *
 * Colors reference CSS custom properties from tokens.css so the theme
 * adapts to light/dark mode automatically via the .dark class.
 */

import { EditorView } from '@codemirror/view'
import { HighlightStyle, syntaxHighlighting } from '@codemirror/language'
import { tags } from '@lezer/highlight'

export const suiEditorTheme = EditorView.theme({
  '&': {
    backgroundColor: 'var(--card)',
    color: 'var(--foreground)',
    fontSize: 'inherit',
    fontFamily: 'inherit',
    // The editor fills its mount box; the box's chrome (border/radius/size)
    // is the component template's sx. CodeMirror internals are styled HERE —
    // the FFI library's own theme system, not authored CSS.
    height: '100%',
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
    background: 'var(--tone-primary-bg)',
  },
  '.cm-activeLine': {
    backgroundColor: 'color-mix(in oklch, var(--muted) 30%, transparent)',
  },
  '.cm-gutters': {
    backgroundColor: 'var(--muted)',
    color: 'var(--muted-foreground)',
    border: 'none',
    borderRight: '1px solid var(--border)',
  },
  '.cm-activeLineGutter': {
    backgroundColor: 'color-mix(in oklch, var(--muted) 60%, transparent)',
    color: 'var(--foreground)',
  },
  '.cm-foldPlaceholder': {
    backgroundColor: 'var(--muted)',
    border: '1px solid var(--border)',
    color: 'var(--muted-foreground)',
  },
  '.cm-tooltip': {
    backgroundColor: 'var(--popover)',
    color: 'var(--popover-foreground)',
    border: '1px solid var(--border)',
    borderRadius: 'var(--radius-md)',
    boxShadow: 'var(--shadow-md)',
  },
  '.cm-tooltip-autocomplete': {
    '& > ul > li[aria-selected]': {
      backgroundColor: 'var(--muted)',
    },
  },
  '.cm-searchMatch': {
    backgroundColor: 'var(--tone-info-border)',
  },
  '.cm-searchMatch.cm-searchMatch-selected': {
    backgroundColor: 'var(--tone-primary-border)',
  },
  '.cm-placeholder': {
    color: 'var(--muted-foreground)',
    fontStyle: 'italic',
  },
  '&.cm-focused': { outline: 'none' },
  '.cm-scroller': {
    fontFamily: 'inherit',
    lineHeight: 'inherit',
    overflow: 'auto',
  },
})

// Syntax colors read --syntax-* tokens (tokens.css) rather than --primary
// / --accent so themes that intentionally subdue their brand palette
// (halo, paper) can still keep code legible at AA contrast.
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
  { tag: tags.variableName, color: 'var(--syntax-variable)' },
  { tag: tags.definition(tags.variableName), color: 'var(--syntax-variable)', fontWeight: '500' },
  { tag: tags.function(tags.variableName), color: 'var(--syntax-function)' },
  { tag: tags.typeName, color: 'var(--syntax-type)' },
  { tag: tags.className, color: 'var(--syntax-type)' },
  { tag: tags.propertyName, color: 'var(--syntax-variable)' },
  { tag: tags.definition(tags.propertyName), color: 'var(--syntax-variable)' },
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
