// Sync highlighter for `<code-block>` / `use:syntaxHighlight`.
//
// TS / shell (and sail-shaped first paint) stay a small regex scanner so
// the runtime never ships a wasm grammar. Apps that have the highlighter
// (docs, studio) call `registerSailHighlighter` after init; sail-shaped
// source then paints from that function. Escapes all source text and only
// ever emits its own <span class="tok-*"> wrappers.
//
// Context model for the regex path: a snippet is either MARKUP (starts
// with a tag or `{#…}`) or CODE (TS / bash). Markup text is prose —
// words there are never keywords. `{…}` in markup is a binding, matched
// with nested-brace/quote awareness. Inside `<controller>` / `<scope>` /
// `<script>` the scanner drops into CODE mode until the closing tag.

const KEYWORDS = new Set([
  'import', 'export', 'from', 'const', 'let', 'var', 'return', 'function',
  'default', 'if', 'else', 'for', 'of', 'in', 'new', 'await', 'async', 'type',
  'interface', 'class', 'extends', 'as',
])
// SAIL / runtime vocabulary — highlighted as functions so the reactive verbs pop.
const VERBS = new Set([
  'signal', 'derived', 'effect', 'controller', 'scope', 'mount', 'untrack',
  'prop', 'data', 'client', 'npm', 'install',
])

// Tags whose content is code, not markup. Mirrors the .sail file sections.
const CODE_TAGS = new Set(['controller', 'scope', 'script'])

function esc(s: string): string {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
}

function span(cls: string, text: string): string {
  return `<span class="tok-${cls}">${esc(text)}</span>`
}

/**
 * Index of the `}` matching the `{` at `openIdx`, or -1. Honours nested
 * braces and quoted strings so attrs like `key={f.id}` and
 * `{playing() ? 'a' : 'b'}` close at the right brace.
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
      i++
      continue
    }
    if (ch === '"' || ch === "'" || ch === '`') { quote = ch; i++; continue }
    if (ch === '{') { depth++; i++; continue }
    if (ch === '}') {
      depth--
      if (depth === 0) return i
      i++
      continue
    }
    i++
  }
  return -1
}

const IDENT_START = /[A-Za-z_$]/
const IDENT = /[\w$]/

/** True when only whitespace sits between `idx` and the start of its line. */
function atLineStart(src: string, idx: number): boolean {
  for (let k = idx - 1; k >= 0; k--) {
    const c = src[k]!
    if (c === '\n') return true
    if (!/\s/.test(c)) return false
  }
  return true
}

/**
 * Highlight a run of code (TS or shell). Braces, parens and arrows are plain
 * punctuation here — a `{` never starts a binding inside code, which is what
 * keeps `effect(() => { … })` bodies intact.
 *
 * `allowMarkupHandoff`: .sail's top-level shape is a code prelude followed by
 * template markup, so a tag at the start of a line hands the rest of the
 * source to the markup scanner. Generics (`signal<Flight[]>`) never sit at
 * line start, so they stay code. Off inside `<controller>`/`<scope>` slices.
 */
function highlightCode(src: string, allowMarkupHandoff = false): string {
  let out = ''
  let plain = ''
  const flush = (): void => { out += esc(plain); plain = '' }
  const len = src.length
  let i = 0
  while (i < len) {
    const ch = src[i]!
    if (allowMarkupHandoff && ch === '<' && /[A-Za-z/]/.test(src[i + 1] ?? '') &&
      atLineStart(src, i)) {
      flush()
      out += highlightMarkup(src.slice(i))
      return out
    }
    if (ch === '/' && src[i + 1] === '/') {
      const end = src.indexOf('\n', i)
      const stop = end < 0 ? len : end
      flush(); out += span('comment', src.slice(i, stop)); i = stop
      continue
    }
    if (ch === '/' && src[i + 1] === '*') {
      const end = src.indexOf('*/', i + 2)
      const stop = end < 0 ? len : end + 2
      flush(); out += span('comment', src.slice(i, stop)); i = stop
      continue
    }
    // Shell-style comment — only at line start / after whitespace, so TS
    // private fields (`#x`) and mid-token hashes stay untouched.
    if (ch === '#' && (i === 0 || /\s/.test(src[i - 1]!))) {
      const end = src.indexOf('\n', i)
      const stop = end < 0 ? len : end
      flush(); out += span('comment', src.slice(i, stop)); i = stop
      continue
    }
    if (ch === '"' || ch === "'" || ch === '`') {
      let j = i + 1
      while (j < len) {
        if (src[j] === '\\') { j += 2; continue }
        if (src[j] === ch) { j++; break }
        j++
      }
      flush(); out += span('string', src.slice(i, j)); i = j
      continue
    }
    if (/\d/.test(ch)) {
      let j = i + 1
      while (j < len && /\d/.test(src[j]!)) j++
      if (src[j] === '.' && /\d/.test(src[j + 1] ?? '')) {
        j += 2
        while (j < len && /\d/.test(src[j]!)) j++
      }
      flush(); out += span('number', src.slice(i, j)); i = j
      continue
    }
    if (IDENT_START.test(ch)) {
      let j = i + 1
      while (j < len && IDENT.test(src[j]!)) j++
      const word = src.slice(i, j)
      flush()
      if (KEYWORDS.has(word)) out += span('keyword', word)
      else if (VERBS.has(word)) out += span('fn', word)
      else out += esc(word)
      i = j
      continue
    }
    plain += ch
    i++
  }
  flush()
  return out
}

/**
 * Highlight a markup document (.sail template / HTML). Text content is prose
 * — never keyword-matched. `{…}` is a binding (nested-brace aware). Content
 * of controller/scope/script tags is handed to `highlightCode`.
 */
function highlightMarkup(src: string): string {
  let out = ''
  const len = src.length
  let i = 0
  while (i < len) {
    const ch = src[i]!
    if (src.startsWith('<!--', i)) {
      const end = src.indexOf('-->', i + 4)
      const stop = end < 0 ? len : end + 3
      out += span('comment', src.slice(i, stop)); i = stop
      continue
    }
    if (ch === '<' && /[A-Za-z/]/.test(src[i + 1] ?? '')) {
      const closing = src[i + 1] === '/'
      const nameStart = closing ? i + 2 : i + 1
      let j = nameStart
      while (j < len && /[\w-]/.test(src[j]!)) j++
      if (j === nameStart) { out += esc(ch); i++; continue }
      const tagName = src.slice(nameStart, j)
      out += span('tag', src.slice(i, j))
      i = j
      // Attribute region, until `>` / `/>`.
      let selfClosing = false
      while (i < len) {
        const c = src[i]!
        if (c === '>') { out += span('tag', '>'); i++; break }
        if (c === '/' && src[i + 1] === '>') {
          out += span('tag', '/>'); i += 2; selfClosing = true; break
        }
        if (c === '"' || c === "'") {
          let k = i + 1
          while (k < len) {
            if (src[k] === '\\') { k += 2; continue }
            if (src[k] === c) { k++; break }
            k++
          }
          out += span('string', src.slice(i, k)); i = k
          continue
        }
        if (c === '{') {
          const close = findMatchingBrace(src, i)
          if (close >= 0) {
            out += span('bind', src.slice(i, close + 1)); i = close + 1
            continue
          }
        }
        if (/[\s=]/.test(c)) { out += esc(c); i++; continue }
        // Attribute name (incl. directive prefixes like `on:click`).
        let k = i + 1
        while (k < len && !/[\s=/>'"{}]/.test(src[k]!)) k++
        out += span('attr', src.slice(i, k)); i = k
      }
      // Code sections: everything up to the closing tag is code.
      if (!closing && !selfClosing && CODE_TAGS.has(tagName.toLowerCase())) {
        const closeIdx = src.indexOf(`</${tagName}`, i)
        const stop = closeIdx < 0 ? len : closeIdx
        out += highlightCode(src.slice(i, stop))
        i = stop
      }
      continue
    }
    if (ch === '{') {
      const close = findMatchingBrace(src, i)
      if (close >= 0) {
        out += span('bind', src.slice(i, close + 1))
        i = close + 1
        continue
      }
    }
    // Plain text run — prose, never keyword-matched.
    let j = i + 1
    while (j < len && src[j] !== '<' && src[j] !== '{') j++
    out += esc(src.slice(i, j))
    i = j
  }
  return out
}

export type SailHtmlHighlighter = (src: string) => string

let sailHtml: SailHtmlHighlighter | null = null
const listeners = new Set<() => void>()

function isSailShaped(code: string): boolean {
  const t = code.trimStart()
  return t.startsWith('<') || t.startsWith('{#')
}

/** Opt-in `.sail` painter (docs / studio). Hello-world does not register. */
export function registerSailHighlighter(fn: SailHtmlHighlighter | null): void {
  sailHtml = fn
  for (const l of listeners) l()
}

export function subscribeSailHighlighter(fn: () => void): () => void {
  listeners.add(fn)
  return () => {
    listeners.delete(fn)
  }
}

export function highlight(code: string): string {
  if (isSailShaped(code) && sailHtml) {
    try {
      return sailHtml(code)
    } catch {
      /* regex fallback */
    }
  }
  return isSailShaped(code) ? highlightMarkup(code) : highlightCode(code, true)
}
