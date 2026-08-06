/**
 * TEMPORARY migration scrub harness (SPEC-DEEMBALM-1).
 *
 * Proves the product corpus has zero authored occurrences of forms that
 * were scrubbed out of the language. Segregated from behavioral suites —
 * run via `bun test tests/scrub` only. Delete this harness when the
 * migration window ends (see AGENTS.md retirement-is-a-scrub).
 */
import { describe, expect, test } from 'bun:test'
import { execSync } from 'node:child_process'
import { readFileSync } from 'node:fs'
import { join, relative, resolve } from 'node:path'

const REPO = resolve(import.meta.dir, '../..')

/** Paths that may mention ghost forms in tests/docs about refusal itself. */
const ALLOW_PATH_RE = [
  /\/tests\/scrub\//,
  /\/AGENTS\.md$/,
  /language-surface\.gen\.ts$/,
  // Compiler/lint internals + refusal contract tests may name dead forms.
  /\/crates\/sail\/(compiler|lint)\//,
  /\/packages\/sail\/compiler-wasm\//,
  /\/packages\/deck\/lsp-browser\//,
  /\/skills\//,
  /CHANGELOG\.md$/,
  /README\.md$/,
]

/** Author-surface globs we scrub. */
const AUTHOR_GLOBS = [
  'apps/**/*.sail',
  'packages/**/*.sail',
  'crates/**/*.sail',
]

/** Patterns that must not appear in product .sail author sources. */
const FORBIDDEN_IN_SAIL: Array<{ name: string; re: RegExp }> = [
  { name: 'fx:', re: /\bfx:[a-zA-Z]/ },
  { name: 'authored motion:', re: /\bmotion:[a-zA-Z]/ },
  { name: 'transition:', re: /\btransition:[a-zA-Z]/ },
  { name: 'bind:this', re: /\bbind:this\b/ },
  { name: 'residual cond=', re: /<(?:if|else-if)\s+cond=/ },
  { name: 'residual of=', re: /<(?:for|case|while-loading|while-error)[^>]*\sof=/ },
  { name: 'residual match on=', re: /<match\s+on=/ },
  // CHANNELS Phase V — Style dual-debt locks (product .sail author surface only).
  // Measured 0 hits before locking. Teaching site TS snippets are out of scope.
  { name: 'retired sx= host', re: /\ssx\s*=/ },
  // Style product as attribute host (`style={…}`); raw CSS `style="…"` is allowed.
  { name: 'retired style={ product attr', re: /\sstyle\s*=\s*\{/ },
  // Flat method chain as block/const product (`style.pad(…)`). Requires `(` so
  // prose like <code>style.enter</code> / media names `style.css…` do not trip.
  {
    name: 'retired flat style.method( product',
    re: /(?:^|[\n>])\s*style\.[a-zA-Z][a-zA-Z0-9_-]*\s*\(/m,
  },
  {
    name: 'retired const = style.method( product',
    re: /\bconst\s+[a-zA-Z_][\w]*\s*=\s*style\.[a-zA-Z]/,
  },
  // CHANNELS Phase V — bare layout participation attrs (box channel only).
  // Axis sugar gap/align/justify on row|column|grid KEPT. col-span/row-span KEPT.
  // Measured 0 hits on product corpus before locking.
  {
    name: 'bare grow participation attr',
    re: /<[A-Za-z][\w:-]*[^>]*\sgrow(?:\s*=|\s|\/|>)/,
  },
  {
    name: 'bare shrink participation attr',
    re: /<[A-Za-z][\w:-]*[^>]*\sshrink(?:\s*=|\s|\/|>)/,
  },
  {
    name: 'bare w= participation attr',
    re: /<[A-Za-z][\w:-]*[^>]*\sw\s*=/,
  },
  {
    name: 'bare min-w= participation attr',
    re: /<[A-Za-z][\w:-]*[^>]*\smin-w\s*=/,
  },
  {
    name: 'bare max-w= participation attr',
    re: /<[A-Za-z][\w:-]*[^>]*\smax-w\s*=/,
  },
]

function gitLs(glob: string): string[] {
  try {
    const out = execSync(`git ls-files -- '${glob}'`, {
      cwd: REPO,
      encoding: 'utf8',
    }).trim()
    if (!out) return []
    return out.split('\n').filter(Boolean)
  } catch {
    return []
  }
}

function allowed(path: string): boolean {
  const rel = path.startsWith(REPO) ? relative(REPO, path) : path
  return ALLOW_PATH_RE.some((re) => re.test('/' + rel) || re.test(rel))
}

describe('scrub: retired author forms absent from corpus', () => {
  const sailFiles = AUTHOR_GLOBS.flatMap(gitLs)

  test('has author .sail files to scan', () => {
    expect(sailFiles.length).toBeGreaterThan(20)
  })

  for (const { name, re } of FORBIDDEN_IN_SAIL) {
    test(`no ${name} in product .sail`, () => {
      const hits: string[] = []
      for (const rel of sailFiles) {
        if (allowed(rel)) continue
        const abs = join(REPO, rel)
        let text: string
        try {
          text = readFileSync(abs, 'utf8')
        } catch {
          continue
        }
        // Strip HTML comments + // line comments so doc/example prose does not fail scrub.
        const stripped = text
          .replace(/<!--[\s\S]*?-->/g, '')
          .replace(/\/\/[^\n]*/g, '')
        if (re.test(stripped)) {
          hits.push(rel)
        }
      }
      expect(hits, `found ${name} in:\n${hits.join('\n')}`).toEqual([])
    })
  }
})
