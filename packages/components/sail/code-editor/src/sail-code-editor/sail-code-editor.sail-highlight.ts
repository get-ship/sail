/**
 * SAIL syntax overlay for CodeMirror 6.
 *
 * Builds on top of `lang-html` (which already gives us tag/attribute
 * coloring) and adds highlighting for SAIL-specific constructs:
 *
 *   • Block directives — `{#if}`, `{#for x of list key={…}}`,
 *     `{#match}`, `{#while-loading of x}`,
 *     `{#while-error err of x}` and their `:else if` / `{:else}` /
 *     `{:case}` / `{:default}` continuations and `{/if}` / `{/for}` /
 *     `{/match}` / `{/while-loading}` / `{/while-error}` closers.
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
 * emits `Decoration.mark` spans with SAIL-specific CSS classes. The
 * theme block at the bottom maps those classes onto the same
 * `--syntax-*` design tokens the rest of the editor uses, so dark
 * / light / theme overrides carry through automatically.
 *
 * Grammar is the real one — see
 * `crates/sail/compiler/src/parse/template/tokenize.rs` (`classify_brace`)
 * and `crates/sail/compiler/src/parse/template/attrs.rs` (`compile_attr`
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
const CLS_DIRECTIVE_SIGIL = 'sail-sigil'       // # / : / /
const CLS_DIRECTIVE_KW = 'sail-directive-kw'   // if / for / match / else / while-loading …
const CLS_DIRECTIVE_EXPR = 'sail-directive-expr' // inner expression after the keyword
const CLS_INTERP = 'sail-interp'               // {expr} interpolation
const CLS_INTERP_BRACE = 'sail-interp-brace'   // the { and } themselves
const CLS_ATTR_PREFIX = 'sail-attr-prefix'     // bind: / on: / class: / …
const CLS_ATTR_NAME = 'sail-attr-name'         // the rest of an attribute name
const CLS_ATTR_MODIFIER = 'sail-attr-modifier' // |prevent / |debounce:N …
// TypeScript-ish tokens inside <controller> / <scope> script bodies.
const CLS_TS_KW = 'sail-ts-kw'
const CLS_TS_STR = 'sail-ts-str'
const CLS_TS_NUM = 'sail-ts-num'
const CLS_TS_CMT = 'sail-ts-cmt'
const CLS_TS_FN = 'sail-ts-fn'

/** Tags whose text body is authored TypeScript, not SAIL template text. */
const SCRIPT_TAGS = new Set(['controller', 'scope'])

const TS_KEYWORDS = new Set([
  'break', 'case', 'catch', 'class', 'const', 'continue', 'debugger', 'default',
  'delete', 'do', 'else', 'enum', 'export', 'extends', 'false', 'finally',
  'for', 'function', 'if', 'import', 'in', 'instanceof', 'let', 'new', 'null',
  'return', 'super', 'switch', 'this', 'throw', 'true', 'try', 'typeof',
  'undefined', 'var', 'void', 'while', 'with', 'yield', 'async', 'await',
  'from', 'as', 'of', 'type', 'interface', 'implements', 'readonly', 'satisfies',
  'infer', 'keyof', 'never', 'unknown', 'any', 'string', 'number', 'boolean',
  'symbol', 'bigint', 'object', 'declare', 'namespace', 'module', 'abstract',
  'static', 'public', 'private', 'protected', 'override', 'get', 'set',
])

// ─── Grammar tables (mirrors `crates/sail/compiler`) ──────────────────
const OPEN_KEYWORDS = [
  'if', 'for', 'match', 'while-loading', 'while-error',
] as const

const CLOSE_KEYWORDS = [
  'if', 'for', 'match', 'while-loading', 'while-error',
] as const

const CONTINUATION_KEYWORDS = [
  'else if', 'else', 'case', 'default',
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
 * Match a directive *body* (the text after the opening sigil). Returns
 * `{ keyword, bodyStart, bodyEnd }` indices into the original `inner`,
 * or `null` when the body doesn't start with one of the SAIL directive
 * keywords. `inner` is the full text between `{` and `}`, including the
 * leading `#`, `/`, or `:` — exactly as the Rust compiler's
 * `classify_brace` receives it (crates/sail/compiler/src/parse/template/
 * tokenize.rs).
 */
function matchOpenBody(inner: string): { keyword: string; bodyStart: number; bodyEnd: number } | null {
  if (inner[0] !== '#') return null
  const afterHash = inner.slice(1)
  // Longest-first so `while-loading` / `while-error` win over `if`/`for`.
  for (const kw of OPEN_KEYWORDS) {
    if (afterHash === kw) return { keyword: kw, bodyStart: 1, bodyEnd: inner.length }
    if (afterHash.startsWith(kw)) {
      const rest = afterHash.slice(kw.length)
      if (rest.length > 0 && /\s/.test(rest[0]!)) {
        return { keyword: kw, bodyStart: 1 + kw.length, bodyEnd: inner.length }
      }
    }
  }
  return null
}

/**
 * Match a continuation body (the text after a `:` sigil). Recognises
 * `else if expr`, `else`, `case expr`, `default`, `loading`, `error expr`,
 * `ok`. Returns the keyword text + body indices, or `null`.
 */
function matchContinuationBody(inner: string): { keyword: string; bodyStart: number; bodyEnd: number } | null {
  if (inner[0] !== ':') return null
  const afterColon = inner.slice(1)
  const trimmed = afterColon.trimStart()
  const leading = afterColon.length - trimmed.length
  for (const kw of CONTINUATION_KEYWORDS) {
    if (trimmed === kw) return { keyword: kw, bodyStart: inner.length, bodyEnd: inner.length }
    if (trimmed.startsWith(kw)) {
      const rest = trimmed.slice(kw.length)
      if (rest.length === 0) {
        return { keyword: kw, bodyStart: 1 + leading + kw.length, bodyEnd: inner.length }
      }
      if (/\s/.test(rest[0]!)) {
        return { keyword: kw, bodyStart: 1 + leading + kw.length, bodyEnd: inner.length }
      }
    }
  }
  return null
}

/**
 * Match a closer body (the text after a `/` sigil). Recognises `/if`,
 * `/for`, `/match`, `/while-loading`, `/while-error`. Returns
 * the matched keyword, or `null`.
 */
function matchCloseBody(inner: string): string | null {
  if (inner[0] !== '/') return null
  const afterSlash = inner.slice(1)
  for (const kw of CLOSE_KEYWORDS) {
    if (afterSlash === kw) return kw
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
    const open = matchOpenBody(inner)
    if (open) {
      const absOpen = base + i
      const absClose = base + close + 1
      // `{` is the brace, `#` is the sigil, then the keyword.
      builder.add(absOpen, absOpen + 1, Decoration.mark({ class: CLS_INTERP_BRACE }))
      builder.add(absOpen + 1, absOpen + 2, Decoration.mark({ class: CLS_DIRECTIVE_SIGIL }))
      builder.add(absOpen + 2, absOpen + 2 + open.keyword.length,
        Decoration.mark({ class: CLS_DIRECTIVE_KW }))
      // Skip the leading whitespace before the expression. `bodyStart`
      // is the index within `inner` (which starts at `i+1`), so the
      // source position is `i + 1 + bodyStart`.
      let exprFrom = i + 1 + open.bodyStart
      while (exprFrom < close && /\s/.test(source[exprFrom]!)) exprFrom++
      if (exprFrom < close) {
        builder.add(base + exprFrom, base + close, Decoration.mark({ class: CLS_DIRECTIVE_EXPR }))
      }
      builder.add(base + close, absClose, Decoration.mark({ class: CLS_INTERP_BRACE }))
      i = close + 1
      continue
    }
    const cont = matchContinuationBody(inner)
    if (cont) {
      const absOpen = base + i
      const absClose = base + close + 1
      // `{`, `:` sigil, keyword, expression, `}`.
      builder.add(absOpen, absOpen + 1, Decoration.mark({ class: CLS_INTERP_BRACE }))
      builder.add(absOpen + 1, absOpen + 2, Decoration.mark({ class: CLS_DIRECTIVE_SIGIL }))
      builder.add(absOpen + 2, absOpen + 2 + cont.keyword.length,
        Decoration.mark({ class: CLS_DIRECTIVE_KW }))
      // Same offset rule as the open branch above: `bodyStart` is an
      // index into `inner`, so add the brace offset to get the source.
      let exprFrom = i + 1 + cont.bodyStart
      while (exprFrom < close && /\s/.test(source[exprFrom]!)) exprFrom++
      if (exprFrom < close) {
        builder.add(base + exprFrom, base + close, Decoration.mark({ class: CLS_DIRECTIVE_EXPR }))
      }
      builder.add(base + close, absClose, Decoration.mark({ class: CLS_INTERP_BRACE }))
      i = close + 1
      continue
    }
    const closeKw = matchCloseBody(inner)
    if (closeKw) {
      const absOpen = base + i
      const absClose = base + close + 1
      // `{`, `/` sigil, keyword, `}`.
      builder.add(absOpen, absOpen + 1, Decoration.mark({ class: CLS_INTERP_BRACE }))
      builder.add(absOpen + 1, absOpen + 2, Decoration.mark({ class: CLS_DIRECTIVE_SIGIL }))
      builder.add(absOpen + 2, absClose - 1, Decoration.mark({ class: CLS_DIRECTIVE_KW }))
      builder.add(absClose - 1, absClose, Decoration.mark({ class: CLS_INTERP_BRACE }))
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
 * Scan an `AttributeName` node for a SAIL directive prefix. Highlights
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

class SailHighlighter implements PluginValue {
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

/**
 * Find `<controller>` / `<scope>` body ranges in the full source.
 * Bodies are TypeScript — must not get SAIL `{expr}` interpolation marks.
 */
function findScriptBodies(source: string): Array<{ from: number; to: number }> {
  const out: Array<{ from: number; to: number }> = []
  for (const tag of SCRIPT_TAGS) {
    const openRe = new RegExp(`<${tag}(\\s[^>]*)?>`, 'gi')
    let m: RegExpExecArray | null
    while ((m = openRe.exec(source)) !== null) {
      const bodyStart = m.index + m[0].length
      const close = `</${tag}>`
      const closeAt = source.toLowerCase().indexOf(close, bodyStart)
      if (closeAt < 0) continue
      if (closeAt > bodyStart) out.push({ from: bodyStart, to: closeAt })
      openRe.lastIndex = closeAt + close.length
    }
  }
  out.sort((a, b) => a.from - b.from)
  return out
}

function covers(ranges: Array<{ from: number; to: number }>, from: number, to: number): boolean {
  // Interval fully inside a script body.
  for (const r of ranges) {
    if (from >= r.from && to <= r.to) return true
  }
  return false
}

/**
 * Lightweight TS lexer for controller/scope bodies. Not a full parser —
 * enough keyword/string/comment/number/function coloring to stop the
 * monochrome "no highlighting" look when HTML mode treats the body as text.
 */
function scanTsRange(
  source: string,
  base: number,
  from: number,
  to: number,
  builder: RangeSetBuilder<Decoration>,
): void {
  let i = from
  while (i < to) {
    const ch = source[i]!
    // line comment
    if (ch === '/' && source[i + 1] === '/') {
      let j = i + 2
      while (j < to && source[j] !== '\n') j++
      builder.add(base + i, base + j, Decoration.mark({ class: CLS_TS_CMT }))
      i = j
      continue
    }
    // block comment
    if (ch === '/' && source[i + 1] === '*') {
      let j = i + 2
      while (j < to - 1 && !(source[j] === '*' && source[j + 1] === '/')) j++
      j = Math.min(to, j + 2)
      builder.add(base + i, base + j, Decoration.mark({ class: CLS_TS_CMT }))
      i = j
      continue
    }
    // string
    if (ch === '"' || ch === "'" || ch === '`') {
      const q = ch
      let j = i + 1
      while (j < to) {
        if (source[j] === '\\') { j += 2; continue }
        if (source[j] === q) { j++; break }
        // template ${…} — stop at ${ and resume after matching }
        if (q === '`' && source[j] === '$' && source[j + 1] === '{') {
          j += 2
          let depth = 1
          while (j < to && depth > 0) {
            if (source[j] === '{') depth++
            else if (source[j] === '}') depth--
            j++
          }
          continue
        }
        j++
      }
      builder.add(base + i, base + j, Decoration.mark({ class: CLS_TS_STR }))
      i = j
      continue
    }
    // number
    if (/[0-9]/.test(ch) || (ch === '.' && /[0-9]/.test(source[i + 1] ?? ''))) {
      let j = i + 1
      while (j < to && /[0-9_xXa-fA-FnNeE.+-]/.test(source[j]!)) j++
      builder.add(base + i, base + j, Decoration.mark({ class: CLS_TS_NUM }))
      i = j
      continue
    }
    // identifier / keyword / function-call
    if (/[A-Za-z_$]/.test(ch)) {
      let j = i + 1
      while (j < to && /[A-Za-z0-9_$]/.test(source[j]!)) j++
      const word = source.slice(i, j)
      let k = j
      while (k < to && /\s/.test(source[k]!)) k++
      if (source[k] === '(') {
        builder.add(base + i, base + j, Decoration.mark({ class: CLS_TS_FN }))
      } else if (TS_KEYWORDS.has(word)) {
        builder.add(base + i, base + j, Decoration.mark({ class: CLS_TS_KW }))
      }
      i = j
      continue
    }
    i++
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
  const source = view.state.doc.toString()
  const scriptBodies = findScriptBodies(source)

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

  const clip = (from: number, to: number) => [Math.max(0, from), Math.min(docEnd, to)] as const

  // Collect all marks then sort — script-body TS scan and SAIL scan must
  // not emit overlapping ranges out of order into RangeSetBuilder.
  type Mark = { from: number; to: number; cls: string }
  const marks: Mark[] = []
  const markBuilder = {
    add(from: number, to: number, dec: Decoration) {
      // Decoration.mark stores class in spec
      const spec = (dec as unknown as { spec?: { class?: string } }).spec
      const cls = spec?.class
      if (cls && from < to) marks.push({ from, to, cls })
    },
  } as unknown as RangeSetBuilder<Decoration>

  for (const vis of visible) {
    const [vFrom, vTo] = clip(vis.from, vis.to)
    // TS highlight for visible script bodies.
    for (const body of scriptBodies) {
      if (body.to <= vFrom || body.from >= vTo) continue
      const from = Math.max(body.from, vFrom)
      const to = Math.min(body.to, vTo)
      scanTsRange(source, 0, from, to, markBuilder)
    }
    for (const iv of intervals) {
      if (iv.to <= vFrom || iv.from >= vTo) continue
      const from = Math.max(iv.from, vFrom)
      const to = Math.min(iv.to, vTo)
      if (iv.kind === 'text') {
        // Skip SAIL brace overlay inside controller/scope bodies — those
        // braces are TS, not template interpolations.
        if (covers(scriptBodies, from, to)) continue
        // Partial overlap: still scan non-script subranges by clipping.
        let cursor = from
        const blockers = scriptBodies
          .filter((b) => b.to > from && b.from < to)
          .sort((a, b) => a.from - b.from)
        for (const b of blockers) {
          if (cursor < b.from) scanTextRange(source.slice(cursor, b.from), cursor, markBuilder)
          cursor = Math.max(cursor, b.to)
        }
        if (cursor < to) scanTextRange(source.slice(cursor, to), cursor, markBuilder)
      } else if (iv.kind === 'attrName') {
        scanAttributeName(source, from, to, markBuilder)
      } else {
        scanAttributeValue(source, from, to, markBuilder)
      }
    }
  }

  marks.sort((a, b) => a.from - b.from || a.to - b.to)
  // RangeSetBuilder rejects overlapping/out-of-order ranges — drop overlaps.
  let lastTo = -1
  for (const m of marks) {
    if (m.from < lastTo) continue
    if (m.from >= m.to) continue
    builder.add(m.from, m.to, Decoration.mark({ class: m.cls }))
    lastTo = m.to
  }

  return builder.finish()
}

const sailHighlightPlugin = ViewPlugin.fromClass(SailHighlighter, {
  decorations: (v) => v.decorations,
})

/**
 * Color rules for the SAIL-specific token classes. They intentionally
 * sit on top of `lang-html`'s attribute coloring — for directive
 * attribute names, we override the inline style the language's
 * `HighlightStyle` writes for `t.attributeName`.
 */
const sailHighlightTheme = EditorView.theme({
  // Block directive sigils — `#`, `{:`, `{/`, plus the matching `}`.
  '& .sail-sigil': {
    color: 'var(--syntax-punctuation, var(--syntax-operator, var(--syntax-keyword)))',
    fontWeight: '600',
  },
  // `if`, `for`, `match`, `while-loading`, `while-error`, `else`, `case`,
  // `default`, etc.
  '& .sail-directive-kw': {
    color: 'var(--syntax-keyword)',
    fontWeight: '600',
  },
  // The expression following a directive keyword (`{#for t of todos key={…}}`).
  '& .sail-directive-expr': {
    color: 'var(--syntax-variable)',
  },
  // Bare `{expr}` interpolation body.
  '& .sail-interp': {
    color: 'var(--syntax-variable)',
  },
  // The `{` and `}` of an interpolation.
  '& .sail-interp-brace': {
    color: 'var(--syntax-punctuation, var(--syntax-operator, var(--muted-foreground)))',
    fontWeight: '600',
  },
  // Directive attribute prefix (`bind:`, `on:`, `class:`, …).
  '& .sail-attr-prefix': {
    color: 'var(--syntax-keyword)',
    fontWeight: '600',
  },
  // The part after the prefix (`value`, `checked`, `done`, `click`, …).
  '& .sail-attr-name': {
    color: 'var(--syntax-attr)',
  },
  // `|prevent`, `|debounce:N`, … on `on:`.
  '& .sail-attr-modifier': {
    color: 'var(--syntax-number)',
  },
  // TypeScript inside <controller> / <scope>.
  '& .sail-ts-kw': {
    color: 'var(--syntax-keyword)',
    fontWeight: '500',
  },
  '& .sail-ts-str': {
    color: 'var(--syntax-string)',
  },
  '& .sail-ts-num': {
    color: 'var(--syntax-number)',
  },
  '& .sail-ts-cmt': {
    color: 'var(--syntax-comment)',
    fontStyle: 'italic',
  },
  '& .sail-ts-fn': {
    color: 'var(--syntax-function)',
  },
})

/**
 * The SAIL language extension: a text-decoration overlay that lights
 * up SAIL's directives + interpolations on top of `lang-html`.
 */
export const sailHighlight = [sailHighlightPlugin, sailHighlightTheme]