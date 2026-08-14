/**
 * Repo purity guards — the front-cover gate.
 * Wired via `bun test tests/hygiene` (package.json test:hygiene / test).
 *
 * The repo is the product; the lab is the lab. eval/ and bench/ entered
 * the tree three times before anyone noticed (purge: 97899b8b) — this
 * gate makes a fourth time a CI failure, not a discovery.
 */
import { describe, expect, test } from 'bun:test'
import { execSync } from 'node:child_process'

const tracked = (): string[] =>
  execSync('git ls-files', { encoding: 'utf8' }).trim().split('\n')

/** Top-level entries that are product. Anything else tracked = a conversation first. */
const TOP_LEVEL_ALLOWLIST = new Set([
  '.cargo', '.deck', '.github', '.gitignore', '.lsp.json', '.npmrc',
  'AGENTS.md', 'CHANGELOG.md', 'Cargo.lock', 'Cargo.toml', 'LICENSE', 'README.md',
  'apps', 'bun.lock', 'crates', 'docs', 'hooks', 'package.json', 'packages',
  'rust-toolchain.toml', 'scripts', 'skills', 'tests', 'third_party',
  'vitest.config.ts',
])

/** Path patterns that are never product, at any depth. */
const FORBIDDEN_TRACKED = [
  /(^|\/)node_modules\//,
  /(^|\/)dist\//,
  /^runs?\//,
  /^eval\//,
  /^bench\//,
  /(^|\/)\.deck\/.+/,
  /(^|\/)\.probe\//,
  /(^|\/)test-results\//,
  /(^|\/)playwright-report\//,
  /\.log$/,
  /\.bundle\.js$/,
  /\.wasm$/,
]

/** Tracked blob size budget. Full icon packs must not be committed. */
const MAX_TRACKED_BYTES = 1_000_000

describe('repo purity', () => {
  test('every tracked top-level entry is allowlisted product', () => {
    const offenders = new Set<string>()
    for (const f of tracked()) {
      const top = f.split('/')[0]
      if (!TOP_LEVEL_ALLOWLIST.has(top)) offenders.add(top)
    }
    expect(
      [...offenders],
      `non-product top-level entries tracked: ${[...offenders].join(', ')} — the repo is the product; lab/eval/scratch lives elsewhere`,
    ).toEqual([])
  })

  test('no forbidden tracked content (output dirs, deps, logs, bundles)', () => {
    const offenders = tracked().filter((f) =>
      FORBIDDEN_TRACKED.some((re) => re.test(f)),
    )
    expect(offenders, `forbidden tracked paths: ${offenders.join(', ')}`).toEqual([])
  })

  test('tracked blobs stay under the size budget', () => {
    const out = execSync('git ls-files | xargs du -b 2>/dev/null || true', {
      encoding: 'utf8',
    })
    const offenders = out
      .trim()
      .split('\n')
      .map((l) => l.split('\t'))
      .filter(([size]) => Number(size) > MAX_TRACKED_BYTES)
      .map(([size, path]) => `${path} (${size})`)
    expect(offenders, `oversized tracked blobs: ${offenders.join(', ')}`).toEqual([])
  })

  test('no agent-narration comment markers in tracked sources', () => {
    // Comments carry facts, decisions, risks — not a transcript of the agent's
    // inner life or a restatement of the next line. Single git grep pass over
    // tracked sources (third_party keeps upstream style).
    const PATTERNS = [
      `'^\\s*//\\s*(Let me|Now I|First I|I will|I'\\''ll|I'\\''m going)'`,
      `-i '^\\s*//\\s*This (function|method|will|handles|ensures|verifies)'`,
      `-i '^\\s*//\\s*As noted (above|earlier)'`,
    ]
    const hits: string[] = []
    for (const args of PATTERNS) {
      try {
        const out = execSync(
          `git grep -n -E ${args} -- '*.rs' '*.ts' '*.tsx' '*.mjs' '*.js' ':!third_party' || true`,
          { encoding: 'utf8', maxBuffer: 8 << 20 },
        ).trim()
        if (out) hits.push(out)
      } catch {
        // no matches
      }
    }
    const offenders = hits.join('\n').split('\n').filter(Boolean)
    expect(
      offenders.slice(0, 20),
      `narration comments at: ${offenders.slice(0, 10).join(', ')} (${offenders.length} total)`,
    ).toEqual([])
  })
})
