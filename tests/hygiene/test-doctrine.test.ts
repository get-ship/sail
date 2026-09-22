// Test-doctrine ratchet — seven regex-level detectors guard how this repo
// tests style: tombstones (retired-surface assertions kept alive), parked
// tests (`return; // patched`, pending ignores), golden pins, skipped suites,
// emitted atoms leaking into test files, site-path strings inside crate
// tests, and tautological var(--) assertions.
//
// Current-state counts are grandfathered in test-doctrine-baseline.json;
// counts may only fall (ratchet). Regenerate after legitimate removals with
// TEST_DOCTRINE_REGENERATE=1. Scope a branch check with
// TEST_DOCTRINE_MODE=changed (diff origin/feat/next...HEAD, fallback HEAD~1).
// Wire: `bun test tests/hygiene/test-doctrine.test.ts`
import { describe, expect, test } from 'bun:test'
import { existsSync, readFileSync, readdirSync, statSync, writeFileSync } from 'node:fs'
import { join, relative } from 'node:path'

const ROOT = join(import.meta.dir, '..', '..')
const BASELINE_PATH = join(import.meta.dir, 'test-doctrine-baseline.json')
const GATE_REL = 'tests/hygiene/test-doctrine.test.ts'

// ── walkers ──────────────────────────────────────────────────────────────

interface ScannedFile {
  rel: string
  text: string
}

const EXCLUDED_DIRS = new Set(['node_modules', 'dist', 'target'])

function walk(dir: string, exts: string[], pred?: (rel: string) => boolean): string[] {
  const out: string[] = []
  if (!existsSync(dir)) return out
  for (const name of readdirSync(dir)) {
    const p = join(dir, name)
    if (statSync(p).isDirectory()) {
      if (EXCLUDED_DIRS.has(name)) continue
      out.push(...walk(p, exts, pred))
    } else if (exts.some((e) => name.endsWith(e))) {
      const rel = relative(ROOT, p).split('\\').join('/')
      if (!pred || pred(rel)) out.push(rel)
    }
  }
  return out
}

function scan(): ScannedFile[] {
  const rels = new Set<string>([
    ...walk(join(ROOT, 'apps/sail-site/src'), ['.test.ts']),
    ...walk(join(ROOT, 'apps/studio'), ['.test.ts', '.spec.ts']),
    ...walk(join(ROOT, 'packages'), ['.test.ts', '.spec.ts']),
    ...walk(join(ROOT, 'tests'), ['.test.ts'], (rel) => rel !== GATE_REL),
    ...walk(join(ROOT, 'crates/sail/compiler/src'), ['.rs']),
    ...walk(join(ROOT, 'crates/sail/spinnaker/src'), ['.rs']),
    ...walk(
      join(ROOT, 'crates'),
      ['.rs'],
      (rel) => rel.includes('/tests/') && !rel.startsWith('crates/sail/render/') && !rel.startsWith('crates/sail/compiler/src/') && !rel.startsWith('crates/sail/spinnaker/src/'),
    ),
  ])
  return [...rels]
    .sort()
    .map((rel) => ({ rel, text: readFileSync(join(ROOT, rel), 'utf8') }))
}

// ── detectors ────────────────────────────────────────────────────────────

interface Hit {
  file: string
  line: number
  code: string
  text: string
}

export const DETECTOR_CODES = [
  'tombstone',
  'parked_test',
  'pin',
  'stay_green',
  'emit',
  'site_fixture',
  'tautology',
] as const

const RETIRED_NAMES = /sx=|while-|STYLE-BINDING/
const RETIRED_COFILE = /sx=|while-|STYLE-BINDING|component\.[a-z]/
const DOCTRINE_CLOSE = '// doctrine-close'

function lineHits(f: ScannedFile, re: RegExp, code: string, exempt?: (line: string) => boolean): Hit[] {
  const hits: Hit[] = []
  for (const [i, line] of f.text.split('\n').entries()) {
    if (re.test(line) && !(exempt && exempt(line))) {
      hits.push({ file: f.rel, line: i + 1, code, text: line })
    }
  }
  return hits
}

function ignoreBodyTombstones(f: ScannedFile): Hit[] {
  const hits: Hit[] = []
  const lines = f.text.split('\n')
  for (const [i, line] of lines.entries()) {
    if (!/^\s*#\[ignore\]/.test(line)) continue
    let j = i + 1
    const body: string[] = [line]
    while (j < lines.length && j - i < 80 && !/^\s*#\[/.test(lines[j]!)) body.push(lines[j++]!)
    if (RETIRED_NAMES.test(body.join('\n'))) {
      hits.push({ file: f.rel, line: i + 1, code: 'tombstone', text: line })
    }
  }
  return hits
}

function tautologies(f: ScannedFile): Hit[] {
  const hits: Hit[] = []
  const expectRe =
    /expect\(((?:[^()]|\([^()]*\))*)\)\s*\.\s*(?:toContain|toBe|toEqual|toMatch|toInclude)\(\s*([`'"])([\s\S]*?)\2/g
  const lines = f.text.split('\n')
  for (const m of f.text.matchAll(expectRe)) {
    const argVars = [...m[1]!.matchAll(/var\((--[a-zA-Z][a-zA-Z0-9-]*)\)/g)].map((v) => v[1]!)
    if (!argVars.some((v) => m[3]!.includes(`var(${v})`))) continue
    const line = f.text.slice(0, m.index).split('\n').length
    hits.push({ file: f.rel, line, code: 'tautology', text: lines[line - 1]! })
  }
  return hits
}

function detect(f: ScannedFile): Hit[] {
  const hits: Hit[] = []
  const isTs = /\.(test|spec)\.ts$/.test(f.rel)
  const isRs = f.rel.endsWith('.rs')
  const isCratesTests = isRs && f.rel.includes('/tests/')

  if (RETIRED_COFILE.test(f.text)) hits.push(...lineHits(f, /not\.toContain\(/, 'tombstone'))
  hits.push(...ignoreBodyTombstones(f))

  hits.push(
    ...lineHits(
      f,
      /return;\s*\/\/\s*patched|#\[\s*ignore[^\]]*"[^"]*pending/,
      'parked_test',
      (line) => line.includes(DOCTRINE_CLOSE),
    ),
  )

  hits.push(...lineHits(f, /SAIL_GOLDEN_UPDATE|UPDATE_SNAPSHOTS/, 'pin'))
  if (isCratesTests) hits.push(...lineHits(f, /\.snap|golden/i, 'pin'))

  if (isTs) {
    hits.push(
      ...lineHits(f, /\bit\.skip\(|describe\.skip\(|test\.skip\(/, 'stay_green', (line) =>
        line.includes(DOCTRINE_CLOSE),
      ),
    )
    hits.push(...lineHits(f, /writeFileSync[^\n]*(golden|snapshot)/i, 'stay_green'))
  }

  if (isTs && /^(apps|packages)\//.test(f.rel)) {
    hits.push(...lineHits(f, /\bs-[a-z]+-[a-z]|data-sail-host/, 'emit'))
  }

  if (isRs) hits.push(...lineHits(f, /apps\/sail-site\/|apps\/studio\//, 'site_fixture'))

  if (isTs) hits.push(...tautologies(f))
  return hits
}

// ── baseline + modes ─────────────────────────────────────────────────────

type Baseline = Record<string, Record<string, number>>

function tally(hits: Hit[]): Baseline {
  const out: Baseline = {}
  for (const h of hits) {
    ;(out[h.file] ??= {})[h.code] = (out[h.file]![h.code] ?? 0) + 1
  }
  return out
}

function changedFiles(): string[] {
  const diff = (rev: string): string[] | null => {
    const proc = Bun.spawnSync(['git', 'diff', '--name-only', rev], { cwd: ROOT })
    if (proc.exitCode !== 0) return null
    return proc.stdout
      .toString()
      .split('\n')
      .map((s) => s.trim())
      .filter(Boolean)
  }
  return diff('origin/feat/next...HEAD') ?? diff('HEAD~1') ?? []
}

// ── gate ─────────────────────────────────────────────────────────────────

describe('test-doctrine ratchet', () => {
  const hits = scan().flatMap(detect)

  test('tree stays at or under baseline', () => {
    if (process.env.TEST_DOCTRINE_REGENERATE === '1') {
      const perCode: Record<string, number> = {}
      for (const h of hits) perCode[h.code] = (perCode[h.code] ?? 0) + 1
      writeFileSync(BASELINE_PATH, JSON.stringify(tally(hits), null, 2) + '\n')
      console.log(`[test-doctrine] baseline regenerated: ${hits.length} hits`, perCode)
      return
    }
    const baseline = JSON.parse(readFileSync(BASELINE_PATH, 'utf8')) as Baseline
    const changed = process.env.TEST_DOCTRINE_MODE === 'changed' ? new Set(changedFiles()) : null
    const scoped = changed ? hits.filter((h) => changed.has(h.file)) : hits
    const current = tally(scoped)
    const tripped = new Set<string>()
    for (const [file, codes] of Object.entries(current)) {
      for (const [code, n] of Object.entries(codes)) {
        if (n > (baseline[file]?.[code] ?? 0)) tripped.add(`${file}\u0000${code}`)
      }
    }
    const rows = scoped
      .filter((h) => tripped.has(`${h.file}\u0000${h.code}`))
      .map((h) => `${h.file}:${h.line} [${h.code}] ${h.text.trim()}`)
    expect(rows, `test-doctrine ratchet tripped — new hits above baseline:\n${rows.join('\n')}`).toEqual([])
  })

  // ── self-test ──────────────────────────────────────────────────────────

  const FIXTURES = join(import.meta.dir, 'fixtures/doctrine')

  function detectVirtual(name: string): Hit[] {
    return detect({
      rel: `apps/sail-site/src/${name.replace(/\.ts$/, '.test.ts')}`,
      text: readFileSync(join(FIXTURES, name), 'utf8'),
    })
  }

  test('violating fixture trips every ts-side detector', () => {
    const fired = new Set(detectVirtual('violating-sample.ts').map((h) => h.code))
    for (const code of ['tombstone', 'parked_test', 'pin', 'stay_green', 'emit', 'tautology']) {
      expect(fired.has(code), `violating-sample must trip ${code}`).toBe(true)
    }
  })

  test('clean fixture fires nothing', () => {
    expect(detectVirtual('clean-sample.ts')).toEqual([])
  })

  test('synthetic rust probes fire rs-side detectors', () => {
    const probe = [
      '#[cfg(test)]',
      'mod probe {',
      '    #[test]',
      '    fn retired_surface_still_referenced() {',
      '        let html = render("<div sx=\\"color:red\\">");',
      '        assert!(html.contains("sx="));',
      '    }',
      '',
      '    #[ignore] // sx= tombstone parked for the next sweep',
      '    #[test]',
      '    fn parked_pending_retire() {',
      '        assert_eq!(leak, "apps/studio/src/app.ts");',
      '    }',
      '',
      '    #[ignore = "pending rework"]',
      '    #[test]',
      '    fn later() {',
      '        assert_eq!(1, 1);',
      '    }',
      '}',
    ].join('\n')
    const fired = detect({ rel: 'crates/sail/probe/tests/probe.rs', text: probe }).map((h) => h.code)
    expect(fired).toContain('tombstone')
    expect(fired).toContain('site_fixture')
    expect(fired).toContain('parked_test')
  })
})
