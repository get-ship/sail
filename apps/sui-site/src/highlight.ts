// A small, dependency-free syntax highlighter for the docs — covers the .sui /
// HTML / TS we show. It escapes all source text and only ever emits its own
// <span class="tok-*"> wrappers (no injection), which the codeblock styles via
// the --syntax-* tokens. A left-to-right single-pass tokenizer, so matches never
// overlap.
//
// (A faithful tree-sitter-sui pass is the eventual upgrade; this gets real,
// correct colour now.)

const KEYWORDS = new Set([
  'import', 'export', 'from', 'const', 'let', 'var', 'return', 'function',
  'default', 'if', 'else', 'for', 'of', 'in', 'new', 'await', 'async', 'type',
  'interface', 'class', 'extends', 'as',
])
// SUI / runtime vocabulary — highlighted as functions so the reactive verbs pop.
const VERBS = new Set([
  'signal', 'derived', 'effect', 'controller', 'scope', 'mount', 'untrack',
  'prop', 'data', 'client', 'npm', 'install',
])

function esc(s: string): string {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
}

// Ordered alternation. Comments + strings first so their contents aren't
// re-tokenized; bindings ({…}) before tags; words last.
const RE = new RegExp(
  [
    String.raw`(?<comment><!--[\s\S]*?-->|\/\/[^\n]*|\/\*[\s\S]*?\*\/|#[^\n]*)`,
    String.raw`(?<string>\`(?:\\.|[^\`\\])*\`|"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*')`,
    String.raw`(?<bind>\{[^{}]*\})`,
    String.raw`(?<tag><\/?[a-zA-Z][\w-]*|\/?>)`,
    String.raw`(?<num>\b\d+(?:\.\d+)?\b)`,
    String.raw`(?<word>[A-Za-z_$][\w$]*)`,
  ].join('|'),
  'g',
)

export function highlight(code: string): string {
  let out = ''
  let last = 0
  for (const m of code.matchAll(RE)) {
    const i = m.index ?? 0
    out += esc(code.slice(last, i))
    const g = m.groups ?? {}
    const text = m[0]
    if (g.comment != null) out += `<span class="tok-comment">${esc(text)}</span>`
    else if (g.string != null) out += `<span class="tok-string">${esc(text)}</span>`
    else if (g.bind != null) out += `<span class="tok-bind">${esc(text)}</span>`
    else if (g.tag != null) out += `<span class="tok-tag">${esc(text)}</span>`
    else if (g.num != null) out += `<span class="tok-number">${esc(text)}</span>`
    else if (g.word != null) {
      if (KEYWORDS.has(text)) out += `<span class="tok-keyword">${esc(text)}</span>`
      else if (VERBS.has(text)) out += `<span class="tok-fn">${esc(text)}</span>`
      else out += esc(text)
    } else out += esc(text)
    last = i + text.length
  }
  out += esc(code.slice(last))
  return out
}
