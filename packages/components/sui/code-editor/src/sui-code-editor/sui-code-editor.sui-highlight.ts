/**
 * SUI syntax overlay for CodeMirror 6.
 *
 * Builds on top of `lang-html` (which already gives us tag/attribute
 * coloring) and adds highlighting for SUI-specific constructs:
 *
 *   • Block directives — `{#if}`, `{#for x of list key={…}}`,
 *     `{#match}`, `{#async}`, `{#while-loading of x}`,
 *     `{#while-error err of x}` and their `:else if` / `{:else}` /
 *     `{:case}` / `{:default}` / `{:loading}` / `{:error …}` / `{:ok}`
 *     continuations and `{/if}` / `{/for}` / `{/match}` / `{/async}` /
 *     `{/while-loading}` / `{/while-error}` closers.
 *   • Interpolation braces — `{expr}` in text content (distinct from
 *     plain text and from the directive forms above).
 *   • Directive attribute prefixes — `bind:` / `on:` / `class:` /
 *     `style:` / `prop:` / `attr:` / `transition:` / `in:` / `out:`
 *     as distinct from a normal attribute name.
 *   • `on:event` modifier segments — `|prevent`, `|stop`,
 *     `|stopImmediatePropagation`, `|self`, `|trusted`, `|key:NAME`,
 *     `|debounce:N`, `|throttle:N`.
 *
 * Approach: a `ViewPlugin` that walks the HTML parse tree, scopes work
 * to text content (`Text` nodes) and attribute name/value nodes, and
 * emits `Decoration.mark` spans with SUI-specific CSS classes. The
 * theme block at the bottom maps those classes onto the same
 * `--syntax-*` design tokens the rest of the editor uses, so dark
 * / light / theme overrides carry through automatically.
 *
 * Grammar is the real one — see
 * `crates/sui/compiler/src/parse/template/tokenize.rs` (`classify_brace`)
 * and `crates/sui/compiler/src/parse/template/attrs.rs` (`compile_attr`
 * for the directive attribute prefixes, `parse_modifier` for the
 * `on:|…` modifier grammar).
 */

import { EditorView } from '@codemirror/view'
import {
  ViewPlugin, Decoration,
  type DecorationSet, type ViewUpdate, type PluginValue,
} from '@codemirror/view'
import { RangeSetBuilder } from '@codemirror/state'
import { syntaxTree } from '@codemirror/language'

// ─── CSS class names ─────────────────────────────────────────────────
// One class per logical token group; the theme below maps each onto a
// `--syntax-*` design token so light/dark themes flow through.
const CLS_DIRECTIVE_SIGIL = 'sui-sigil'       // # / : / /
const CLS_DIRECTIVE_KW = 'sui-directive-kw'   // if / for / match / else / while-loading …
const CLS_DIRECTIVE_EXPR = 'sui-directive-expr' // inner expression after the keyword
const CLS_INTERP = 'sui-interp'               // {expr} interpolation
const CLS_INTERP_BRACE = 'sui-interp-brace'   // the { and } themselves
const CLS_ATTR_PREFIX = 'sui-attr-prefix'     // bind: / on: / class: / …
const CLS_ATTR_NAME = 'sui-attr-name'         // the rest of an attribute name
const CLS_ATTR_MODIFIER = 'sui-attr-modifier' // |prevent / |debounce:N …

// ─── Grammar tables (mirrors `crates/sui/compiler`) ──────────────────
const OPEN_KEYWORDS = [
  'if', 'for', 'match', 'async', 'while-loading', 'while-error',
] as const

const CLOSE_KEYWORDS = [
  'if', 'for', 'match', 'async', 'while-loading', 'while-error',
] as const

const CONTINUATION_KEYWORDS = [
  'else if', 'else', 'case', 'default', 'loading', 'error', 'ok',
] as const

const ATTR_PREFIXES = [
  'bind', 'on', 'class', 'style', 'prop', 'attr',
  'transition', 'in', 'out',
] as const

const ATTR_MODIFIERS_NO_ARG = new Set([
  'prevent', 'preventdefault',
  'stop', 'stoppropagation',
  'stopimmediatepropagation',
  'self',
  'trusted',
])
const ATTR_MODIFIERS_ARG = new Set(['key', 'debounce', 'throttle'])

/**
 * Match an opening directive inner: `if`, `for x of list key={…}`,
 * `match x`, `async`, `while-loading of x`, `while-error err of x`.
 * Returns `{ keyword, exprStart, exprEnd }` or `null`.
 */
function matchOpen(inner: string): { keyword: string; exprStart: number; exprEnd: number } | null {
  // Longest-first so `while-loading` / `while-error` win over `if`/`for`.
  for (const kw of OPEN_KEYWORDS) {
    if (inner === kw) return { keyword: kw, exprStart: inner.length, exprEnd: inner.length }
    if (inner.startsWith(kw)) {
      const after = inner.slice(kw.length)
      if (after.length > 0 && /\s/.test(after[0]!)) {
        return { keyword: kw, exprStart: kw.length, exprEnd: inner.length }
      }
    }
  }
  return null
}

/**
 * Match a continuation inner: `else if expr`, `else`, `case expr`,
 * `default`, `loading`, `error expr`, `ok`. Returns the keyword text
 * and the (trimmed) expression slice, or `null`.
 */
function matchContinuation(inner: string): { keyword: string; exprStart: number; exprEnd: number } | null {
  const trimmed = inner.trim()
  for (const kw of CONTINUATION_KEYWORDS) {
    if (trimmed === kw) return { keyword: kw, exprStart: inner.length, exprEnd: inner.length }
    if (trimmed.startsWith(kw)) {
      const after = trimmed.slice(kw.length)
      if (after.length > 0 && /\s/.test(after[0]!)) {
        // Map back to original-position indices.
        const leading = inner.length - trimmed.length
        return {
          keyword: kw,
          exprStart: leading + kw.length,
          exprEnd: leading + trimmed.length,
        }
      }
    }
  }
  return null
}

/**
 * Match a closer inner: `/if`, `/for`, …
 */
function matchClose(inner: string): string | null {
  for (const kw of CLOSE_KEYWORDS) {
    if (inner === `/${kw}`) return kw
  }
  return null
}

/**
 * Scan a single text node (the source slice between two HTML tags) for
 * brace constructs. Emits decorations for opening directives,
 * continuations, closers, and bare interpolations. Returns `true` if
 * any decoration was added (used for early termination in the caller).
 */
function scanTextRange(
  source: string,
  base: number,
  builder: RangeSetBuilder<Decoration>,
): void {
  const len = source.length
  let i = 0
  while (i < len) {
    const ch = source[i]
    if (ch !== '{') { i++; continue }
    // Find matching close brace, respecting nested braces and quoted strings.
    const close = findMatchingBrace(source, i)
    if (close < 0) { i++; continue }
    const inner = source.slice(i + 1, close)
    // Try the more specific shapes first.
    const open = matchOpen(inner)
    if (open) {
      const absOpen = base + i
      const absClose = base + close + 1
      builder.add(absOpen, absOpen + 1, Decoration.mark({ class: CLS_DIRECTIVE_SIGIL }))
      builder.add(absOpen + 1, absOpen + 1 + ('#' + open.keyword).length,
        Decoration.mark({ class: `${CLS_DIRECTIVE_SIGIL} ${CLS_DIRECTIVE_KW}` }))
      // Skip the leading whitespace before the expression.
      let exprFrom = i + 1 + open.exprStart
      while (exprFrom < close && /\s/.test(source[exprFrom]!)) exprFrom++
      if (exprFrom < close) {
        builder.add(base + exprFrom, base + close, Decoration.mark({ class: CLS_DIRECTIVE_EXPR }))
      }
      builder.add(base + close, absClose, Decoration.mark({ class: CLS_DIRECTIVE_SIGIL }))
      i = close + 1
      continue
    }
    const cont = matchContinuation(inner)
    if (cont) {
      const absOpen = base + i
      const absClose = base + close + 1
      builder.add(absOpen, absOpen + 2, Decoration.mark({ class: CLS_DIRECTIVE_SIGIL }))
      builder.add(absOpen + 2, absOpen + 2 + cont.keyword.length,
        Decoration.mark({ class: `${CLS_DIRECTIVE_SIGIL} ${CLS_DIRECTIVE_KW}` }))
      let exprFrom = i + 1 + cont.exprStart
      while (exprFrom < close && /\s/.test(source[exprFrom]!)) exprFrom++
      if (exprFrom < close) {
        builder.add(base + exprFrom, base + close, Decoration.mark({ class: CLS_DIRECTIVE_EXPR }))
      }
      builder.add(base + close, absClose, Decoration.mark({ class: CLS_DIRECTIVE_SIGIL }))
      i = close + 1
      continue
    }
    const closeKw = matchClose(inner)
    if (closeKw) {
      const absOpen = base + i
      const absClose = base + close + 1
      builder.add(absOpen, absOpen + 2, Decoration.mark({ class: CLS_DIRECTIVE_SIGIL }))
      builder.add(absOpen + 2, absClose - 1, Decoration.mark({ class: CLS_DIRECTIVE_KW }))
      builder.add(absClose - 1, absClose, Decoration.mark({ class: CLS_DIRECTIVE_SIGIL }))
      i = close + 1
      continue
    }
    // Bare interpolation `{expr}` — color braces distinctly from the body.
    const absOpen = base + i
    const absClose = base + close + 1
    builder.add(absOpen, absOpen + 1, Decoration.mark({ class: CLS_INTERP_BRACE }))
    builder.add(absOpen + 1, base + close, Decoration.mark({ class: CLS_INTERP }))
    builder.add(base + close, absClose, Decoration.mark({ class: CLS_INTERP_BRACE }))
    i = close + 1
  }
}

/**
 * Return the index of the `}` that matches the `{` at `openIdx`, or
 * `-1` if none. Honours nested braces and single/double-quoted
 * strings (so `{a: "}"}` doesn't terminate at the inner quote).
 */
function findMatchingBrace(source: string, openIdx: number): number {
  let depth = 1
  let i = openIdx + 1
  let quote: string | null = null
  while (i < source.length) {
    const ch = source[i]!
    if (quote) {
      if (ch === '\\') { i += 2; continue }
      if (ch === quote) quote = null
      i++; continue
    }
    if (ch === '"' || ch === "'") { quote = ch; i++; continue }
    if (ch === '{') { depth++; i++; continue }
    if (ch === '}') {
      depth--
      if (depth === 0) return i
      i++; continue
    }
    i++
  }
  return -1
}

/**
 * Scan an `AttributeName` node for a SUI directive prefix. Highlights
 * `prefix:name` by separating the prefix from the rest, and tints the
 * `on:` modifier segments (`|prevent`, `|key:Enter` …) differently.
 */
function scanAttributeName(
  source: string,
  from: number,
  to: number,
  builder: RangeSetBuilder<Decoration>,
): void {
  const name = source.slice(from, to)
  const colonIdx = name.indexOf(':')
  if (colonIdx < 0) return
  const prefix = name.slice(0, colonIdx).toLowerCase()
  if (!(ATTR_PREFIXES as readonly string[]).includes(prefix)) return

  builder.add(from, from + colonIdx + 1, Decoration.mark({ class: CLS_ATTR_PREFIX }))
  builder.add(from + colonIdx + 1, to, Decoration.mark({ class: CLS_ATTR_NAME }))

  // `on:event` modifiers — segments after `|` that follow the event name.
  if (prefix === 'on') {
    let i = colonIdx + 1
    while (i < name.length) {
      const c = name[i]!
      if (c === '|') {
        // The modifier extends from `i` up to the next `|`, end of name, or
        // any attribute separator (whitespace, `=`, `>`, `/`). We're inside
        // `AttributeName` so we only need to stop at end-of-name here.
        const start = i
        i++
        let argStart = -1
        while (i < name.length && /[A-Za-z0-9]/.test(name[i]!)) i++
        const head = name.slice(start + 1, i).toLowerCase()
        if (i < name.length && name[i] === ':') {
          argStart = i
          i++
          while (i < name.length && /[A-Za-z0-9]/.test(name[i]!)) i++
        }
        if (ATTR_MODIFIERS_NO_ARG.has(head) ||
          (ATTR_MODIFIERS_ARG.has(head) && argStart >= 0)) {
          builder.add(from + start, from + i, Decoration.mark({ class: CLS_ATTR_MODIFIER }))
        }
      } else {
        i++
      }
    }
  }
}

/**
 * Scan an attribute value node for an interpolation expression. Only
 * braces-style values (`{…}`, `'…'` is never an interp) are treated.
 */
function scanAttributeValue(
  source: string,
  from: number,
  to: number,
  builder: RangeSetBuilder<Decoration>,
): void {
  const value = source.slice(from, to)
  if (!value) return
  // Strip the outer quotes (if any) — only `"{…}"` / `'{…}'` carry an
  // interpolation; `"done"` is a plain string.
  const trimmed = value.replace(/^["']|["']$/g, '')
  if (!trimmed.startsWith('{') || !trimmed.endsWith('}')) return
  // Find the matching close from position 0 (the opening `{`).
  const close = findMatchingBrace(trimmed, 0)
  if (close < 0) return
  const valueStart = from + (value.length - trimmed.length)
  builder.add(valueStart, valueStart + 1, Decoration.mark({ class: CLS_INTERP_BRACE }))
  builder.add(valueStart + 1, valueStart + close, Decoration.mark({ class: CLS_INTERP }))
  builder.add(valueStart + close, valueStart + close + 1, Decoration.mark({ class: CLS_INTERP_BRACE }))
}

class SuiHighlighter implements PluginValue {
  decorations: DecorationSet

  constructor(view: EditorView) {
    this.decorations = build(view)
  }

  update(update: ViewUpdate): void {
    if (update.docChanged || update.viewportChanged) {
      this.decorations = build(update.view)
    }
  }
}

function build(view: EditorView): DecorationSet {
  const builder = new RangeSetBuilder<Decoration>()
  const tree = syntaxTree(view.state)
  if (!tree.length) return builder.finish()

  // Walk the visible range so very large docs don't pay for offscreen
  // highlighting. Visible range = what CodeMirror is about to render.
  const visible = view.visibleRanges
  const docEnd = view.state.doc.length

  // Collect text + attribute intervals first so we can intersect with
  // the visible range cheaply.
  const intervals: Array<{ from: number; to: number; kind: 'text' | 'attrName' | 'attrValue' }> = []
  tree.iterate({
    enter(node) {
      const name = node.type.name
      if (name === 'Text') {
        intervals.push({ from: node.from, to: node.to, kind: 'text' })
      } else if (name === 'AttributeName') {
        intervals.push({ from: node.from, to: node.to, kind: 'attrName' })
      } else if (name === 'AttributeValue' || name === 'UnquotedAttributeValue') {
        intervals.push({ from: node.from, to: node.to, kind: 'attrValue' })
      }
    },
  })

  const source = view.state.doc.toString()
  const clip = (from: number, to: number) => [Math.max(0, from), Math.min(docEnd, to)] as const

  for (const vis of visible) {
    const [vFrom, vTo] = clip(vis.from, vis.to)
    for (const iv of intervals) {
      if (iv.to <= vFrom || iv.from >= vTo) continue
      const from = Math.max(iv.from, vFrom)
      const to = Math.min(iv.to, vTo)
      if (iv.kind === 'text') {
        scanTextRange(source.slice(from, to), from, builder)
      } else if (iv.kind === 'attrName') {
        scanAttributeName(source, from, to, builder)
      } else {
        scanAttributeValue(source, from, to, builder)
      }
    }
  }

  return builder.finish()
}

const suiHighlightPlugin = ViewPlugin.fromClass(SuiHighlighter, {
  decorations: (v) => v.decorations,
})

/**
 * Color rules for the SUI-specific token classes. They intentionally
 * sit on top of `lang-html`'s attribute coloring — for directive
 * attribute names, we override the inline style the language's
 * `HighlightStyle` writes for `t.attributeName`.
 */
const suiHighlightTheme = EditorView.theme({
  // Block directive sigils — `#`, `{:`, `{/`, plus the matching `}`.
  '& .sui-sigil': {
    color: 'var(--syntax-punctuation, var(--syntax-operator, var(--syntax-keyword)))',
    fontWeight: '600',
  },
  // `if`, `for`, `match`, `async`, `while-loading`, `else`, `case`, `default`,
  // `loading`, `error`, `ok`, etc.
  '& .sui-directive-kw': {
    color: 'var(--syntax-keyword)',
    fontWeight: '600',
  },
  // The expression following a directive keyword (`{#for t of todos key={…}}`).
  '& .sui-directive-expr': {
    color: 'var(--syntax-variable)',
  },
  // Bare `{expr}` interpolation body.
  '& .sui-interp': {
    color: 'var(--syntax-variable)',
  },
  // The `{` and `}` of an interpolation.
  '& .sui-interp-brace': {
    color: 'var(--syntax-punctuation, var(--syntax-operator, var(--muted-foreground)))',
    fontWeight: '600',
  },
  // Directive attribute prefix (`bind:`, `on:`, `class:`, …).
  '& .sui-attr-prefix': {
    color: 'var(--syntax-keyword)',
    fontWeight: '600',
  },
  // The part after the prefix (`value`, `checked`, `done`, `click`, …).
  '& .sui-attr-name': {
    color: 'var(--syntax-attr)',
  },
  // `|prevent`, `|debounce:N`, … on `on:`.
  '& .sui-attr-modifier': {
    color: 'var(--syntax-number)',
  },
})

/**
 * The SUI language extension: a text-decoration overlay that lights
 * up SUI's directives + interpolations on top of `lang-html`.
 */
export const suiHighlight = [suiHighlightPlugin, suiHighlightTheme]