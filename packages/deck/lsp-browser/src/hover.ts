// Browser-side hover harness for .sail templates.
//
// Static dictionary lookups for directives / modifiers / block keywords,
// sourced from the compiler's own vocabulary tables via WASM. This replaces
// every hand-maintained copy in this file with the same data the parser
// renders in its unknown-directive errors and the docs tables derive from.

import initWasm, { completion_vocabulary } from '@ship/sail-compiler-wasm'

export interface SailHoverInfo {
  /** Markdown — rendered in the editor's hover tooltip. */
  contents: string
  /** Optional 1-based range to highlight in the editor when the
   *  tooltip is open. */
  range?: { line: number; column: number; endColumn?: number }
}

interface WasmVocabulary {
  modifiers: Array<{
    canonical: string
    alias?: string
    arg?: string
    summary: string
  }>
  blocks: Array<{ open: string; clauses: string[]; summary: string }>
  directives: Array<{ syntax: string; summary: string }>
  foreign_idioms: Array<{ syntax: string; instead: string }>
}

const WASM_URL = '/sail_compiler_bg.wasm'

let vocab: WasmVocabulary | null = null
let initDone: Promise<void> | null = null

function ensureInit(): Promise<void> {
  if (!initDone) {
    initDone = (async () => {
      await initWasm(WASM_URL)
      vocab = completion_vocabulary() as WasmVocabulary
    })()
  }
  return initDone
}

// Lazy — do NOT top-level await (breaks framework singleton bake + importmap).

const TAG_ATTR_RE = /([\w:-]+)(?=\s*=|\s|$)/g

function blockName(open: string): string {
  // Element form: `<if {…}>` / brace form: `{#if}`
  const s = open.replace(/^\{#/, '').replace(/^</, '')
  return s.split(/[\s{>/]/)[0] ?? s
}

function clauseName(clause: string): string {
  // Arms: `{else => …}` / `{loading => …}` — take the head keyword.
  const s = clause.replace(/^\{:?/, '').trim()
  return s.split(/[\s=>]/)[0] ?? s
}

function directivePrefix(syntax: string): string {
  // Attribute directives: `on:<event>...`, `bind:value / ...`, plain fields,
  // `sx="..."`, `slot="name"`, `<slot name="...">`, etc.
  const attr = syntax.match(/^<?[\w:-]+/)
  if (attr) return attr[0]
  // Fallback for shapes like `(--name)` inside sx values.
  return syntax.split(/\s/)[0]!
}

/**
 * Resolve a hover query at `offset` against the source text. Returns
 * static doc if the cursor is over a recognized directive, modifier,
 * or block keyword; null otherwise.
 */
export function hoverAt(source: string, offset: number): SailHoverInfo | null {
  // Sync path for editor tooltips — vocab may still be loading; return null
  // until ready rather than throwing (would break hoverTooltip).
  if (!vocab) {
    void ensureInit()
    return null
  }
  if (offset < 0 || offset > source.length) return null

  // Walk back over identifier chars only (and `:` so `on:click` and
  // `key:Enter` survive as one token). Sigils like `|` or `{#` stop
  // the walk so we can check them as the *preceding* context below.
  let start = offset
  while (start > 0 && /[\w:-]/.test(source[start - 1]!)) start--
  let end = offset
  while (end < source.length && /[\w:-]/.test(source[end]!)) end++

  const token = source.slice(start, end)
  if (!token) return null

  // 1. Directive prefix (on:, bind:, prop:, class:, style:). The
  //    token here will already include the trailing `:` (`on:`,
  //    `on:click`, etc.) so prefix-match is enough.
  for (const d of vocab.directives) {
    const prefix = directivePrefix(d.syntax)
    if (token === prefix || token.startsWith(prefix)) {
      return renderEntry(d.syntax, d.summary)
    }
  }

  // 1b. Foreign idioms (class:, style:, …) — teach the sail way.
  for (const d of vocab.foreign_idioms) {
    const prefix = d.syntax.replace(':', ':').split(' ')[0]!
    if (token === prefix || token.startsWith(prefix)) {
      return renderEntry(d.syntax, `**Foreign idiom.** ${d.instead}`)
    }
  }

  // 2. Modifier — preceded by `|`. e.g. `on:submit|preventDefault`,
  //    cursor in `preventDefault`, source[start-1] === '|'.
  if (start > 0 && source[start - 1] === '|') {
    const base = token.includes(':') ? token.slice(0, token.indexOf(':')) : token
    for (const m of vocab.modifiers) {
      if (m.canonical === base || m.alias === base) {
        const sig = m.arg ? `|${m.canonical}:<${m.arg}>` : `|${m.canonical}`
        return renderEntry(sig, m.summary)
      }
    }
  }

  // 3. Block keyword — element open `<match`, brace `{#if}`, arm `{else`.
  if (start >= 1 && source[start - 1] === '<') {
    for (const b of vocab.blocks) {
      if (blockName(b.open) === token) {
        const clauses = b.clauses.join(' ')
        const sig = `${b.open}${clauses ? ` … ${clauses}` : ''}`
        return renderEntry(sig, b.summary)
      }
    }
  }
  if (start >= 2) {
    const sigil = source.slice(start - 2, start)
    if (sigil === '{#' || sigil === '{:') {
      for (const b of vocab.blocks) {
        if (sigil === '{#' && blockName(b.open) === token) {
          const clauses = b.clauses.join(' ')
          const sig = `${b.open}${clauses ? ` … ${clauses}` : ''}`
          return renderEntry(sig, b.summary)
        }
        if (sigil === '{:' && b.clauses.some((c) => clauseName(c) === token)) {
          const sig = `${sigil}${token}`
          return renderEntry(sig, b.summary)
        }
      }
    }
  }
  // Arm head inside `{loading => …}` / `{else => …}` (no preceding `#`/`: `).
  if (start >= 1 && source[start - 1] === '{') {
    for (const b of vocab.blocks) {
      if (b.clauses.some((c) => clauseName(c) === token || c.includes(token))) {
        return renderEntry(b.open, b.summary)
      }
    }
  }

  return null
}

function renderEntry(signature: string, doc: string): SailHoverInfo {
  return { contents: `${signature}\n\n${doc}` }
}
