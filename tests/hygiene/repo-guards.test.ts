/**
 * Repo hygiene guards (HYGIENE-0).
 * Converted from scripts/test-no-*.sh — real tests in the suite, not shell wrappers.
 * Wire: package.json `test` / `test:hygiene`.
 *
 * Coverage:
 * 1. documentElement.style writes under apps/** (apps-only by design; packages/
 *    controller sources are covered by sail-no-imperative-style lint; harness
 *    tests under packages/ may still touch :root for mechanism proofs).
 * 2. no residual xtask-deck / phantom gen-* tasks
 * 3. T-I6 paid product module refs in ops/studio core
 * 4. standing sanitization scan classes (operator paths, mission paths,
 *    job/mission process ids, dated owner-ruling monologue markers)
 */
import { describe, expect, test } from 'bun:test'
import { existsSync, readdirSync, readFileSync, statSync, writeFileSync, rmSync } from 'node:fs'
import { join, relative, resolve } from 'node:path'

const REPO = resolve(import.meta.dir, '../..')

const SKIP_DIRS = new Set([
  'node_modules',
  'target',
  'dist',
  'coverage',
  '.deck',
  '.artifacts',
  '.git',
  '.claude',
  '.grok',
  '.agents', // org scratch; not product surface
  '.sail-out', // residual pre-OUTPUT-1 emit dir (not product default)
  'out', // test/parity runner output (absolute paths, regenerable)
  'runs',
  'eval',
])

/** Generated / local-only files that may sit on disk but never ship. */
const SKIP_FILES = new Set([
  'CLAUDE.md',
  'AGENTS.md',
  'GEMINI.md',
  '.mcp.json',
  'skills-lock.json',
])

function* walkFiles(dir: string, base = dir): Generator<string> {
  let entries: string[]
  try {
    entries = readdirSync(dir)
  } catch {
    return
  }
  for (const entry of entries) {
    if (SKIP_DIRS.has(entry)) continue
    if (SKIP_FILES.has(entry)) continue
    const p = join(dir, entry)
    let st
    try {
      st = statSync(p)
    } catch {
      continue
    }
    if (st.isDirectory()) {
      yield* walkFiles(p, base)
    } else if (st.isFile()) {
      yield p
    }
  }
}

function readText(path: string): string | null {
  try {
    return readFileSync(path, 'utf8')
  } catch {
    return null
  }
}

function skipBinaryRel(rel: string): boolean {
  return /\.(png|webp|jpg|jpeg|svg|woff2?|lock|wasm)$/i.test(rel)
}

describe('no documentElement.style under apps/**', () => {
  // Scope: apps/** only (matches original shell intent). Writes only —
  // reads via getComputedStyle / .style.getPropertyValue in tests are OK.
  // packages/ is not walked here: framework theme hatch + component harness
  // tests may set :root tokens; controller product code is linted separately.
  const pattern =
    /document\.documentElement\.style\.(setProperty|removeProperty|cssText)|documentElement\.style\.(setProperty|removeProperty)|documentElement[^;\n]{0,80}\.style\.(setProperty|removeProperty)/

  test('self-test: ban pattern rejects fixture pollution', () => {
    const sample = "document.documentElement.style.setProperty('--x', '1')"
    expect(pattern.test(sample)).toBe(true)
    expect(pattern.test("document.documentElement.style.getPropertyValue('--x')")).toBe(false)
  })

  test('apps source is clean', () => {
    const apps = join(REPO, 'apps')
    if (!existsSync(apps)) {
      throw new Error('apps/ missing — hygiene guard cannot skip fixtures')
    }
    const hits: string[] = []
    for (const file of walkFiles(apps)) {
      if (file.endsWith('.gen.ts') || file.endsWith('.bundle.js')) continue
      const text = readText(file)
      if (!text) continue
      if (pattern.test(text)) {
        hits.push(relative(REPO, file))
      }
    }
    expect(hits).toEqual([])
  })
})

describe('no residual xtask-deck / phantom gen-* task refs', () => {
  // After HYGIENE-0, `cargo xtask` is real. Ban the retired crate name and
  // never-landed gen-* family (only gen-icons exists).
  const retiredName = /xtask-deck/
  const phantomGen =
    /(?:cargo\s+xtask|cargo\s+run\s+-p\s+xtask)\b[^\n]*\bgen-(tokens|types|skills|fixtures|ops|sdk)\b/

  test('no xtask-deck package name left in source/docs', () => {
    const hits: string[] = []
    for (const file of walkFiles(REPO)) {
      const rel = relative(REPO, file)
      if (rel === 'Cargo.lock') continue
      if (rel.startsWith('tests/hygiene/')) continue
      if (skipBinaryRel(rel)) continue
      const text = readText(file)
      if (!text) continue
      if (retiredName.test(text)) {
        hits.push(rel)
      }
    }
    expect(hits).toEqual([])
  })

  test('no phantom gen-* tasks beyond gen-icons', () => {
    const hits: string[] = []
    for (const file of walkFiles(REPO)) {
      const rel = relative(REPO, file)
      if (rel.startsWith('tests/hygiene/')) continue
      if (rel === 'Cargo.lock') continue
      if (skipBinaryRel(rel)) continue
      const text = readText(file)
      if (!text) continue
      if (phantomGen.test(text)) {
        hits.push(rel)
      }
    }
    expect(hits).toEqual([])
  })
})

describe('T-I6: no paid product module refs in ops/studio core', () => {
  const pattern =
    /ShipDeploy|ShipCMS|ShipWorkflow|ShipPro\b|ShipModule|ship-pro-pack|ship_paid|ENABLE_SHIP_|SHIP_PAID_|paid.?module.?flag|feature.?flag.*[Ss]hip.*module/i

  const targets = ['crates/sail/ops/src', 'apps/studio/src']

  test('scan targets exist or are explicitly absent', () => {
    const any = targets.some((t) => existsSync(join(REPO, t)))
    expect(any).toBe(true)
  })

  test('no paid product hardcodes in ops/studio core', () => {
    const hits: string[] = []
    for (const t of targets) {
      const root = join(REPO, t)
      if (!existsSync(root)) continue
      for (const file of walkFiles(root)) {
        if (file.endsWith('.md')) continue
        if (relative(REPO, file).includes('.__hygiene_')) continue
        const text = readText(file)
        if (!text) continue
        if (pattern.test(text)) {
          hits.push(relative(REPO, file))
        }
      }
    }
    expect(hits).toEqual([])
  })

  test('module loader stays product-agnostic when present', () => {
    const mod = join(REPO, 'crates/sail/ops/src/module.rs')
    if (!existsSync(mod)) return
    const text = readText(mod) ?? ''
    expect(/namespace:\s*"Ship|"Ship[A-Z]/.test(text)).toBe(false)
  })

  /** Same walk used by the production T-I6 guard (not a string-only tautology). */
  function scanProductRefs(): string[] {
    const hits: string[] = []
    for (const t of targets) {
      const root = join(REPO, t)
      if (!existsSync(root)) continue
      for (const file of walkFiles(root)) {
        if (file.endsWith('.md')) continue
        const text = readText(file)
        if (!text) continue
        if (pattern.test(text)) {
          hits.push(relative(REPO, file))
        }
      }
    }
    return hits
  }

  test('deliberate Ship-ref violation fails the guard (prove + clean)', () => {
    const scratch = join(REPO, 'apps/studio/src')
    if (!existsSync(scratch)) {
      throw new Error('apps/studio/src missing — cannot prove T-I6 walk guard')
    }
    // Name is gitignored (`**/.__hygiene_*`) so a hard-kill mid-test cannot
    // leave a staged product path.
    const probe = join(scratch, '.__hygiene_ship_ref_probe.ts')
    const probeRel = relative(REPO, probe)
    writeFileSync(probe, 'export const x = "ShipDeploy"\n')
    try {
      const dirty = scanProductRefs()
      expect(dirty).toContain(probeRel)
    } finally {
      try {
        rmSync(probe)
      } catch {
        /* ignore */
      }
    }
    const clean = scanProductRefs()
    expect(clean).not.toContain(probeRel)
    expect(clean).toEqual([])
  })
})

describe('sanitization standing scan (paths / mission / monologue markers)', () => {
  // High-signal classes from HYGIENE-0 §4. Complements T-I6 (paid modules).
  // Full prose monologue still needs occasional human review; these patterns
  // catch the classes that already regressed once.
  const classes: { name: string; pattern: RegExp }[] = [
    {
      name: 'operator home paths',
      // Real machine homes continue past the username (`/home/user/...` or
      // `/Users/user/...`). Route segments like `/home/page.sail` do not.
      pattern: /\/home\/[a-zA-Z][a-zA-Z0-9_-]+\/|\/Users\/[A-Za-z][A-Za-z0-9_-]+\//,
    },
    {
      name: 'org mission filesystem paths',
      pattern: /helm\/data\/missions|m-deck-standalone/,
    },
    {
      name: 'process job / mission ids',
      pattern: /\bjob\/[a-z][a-z0-9_-]+|\bm-sail-[a-z0-9_-]+/,
    },
    {
      name: 'task number diary',
      pattern: /\btask\s*#\d+/i,
    },
    {
      name: 'dated owner-ruling monologue',
      pattern: /Owner\s+[Rr]uling/,
    },
  ]

  function scanHits(pattern: RegExp): string[] {
    const hits: string[] = []
    for (const file of walkFiles(REPO)) {
      const rel = relative(REPO, file)
      if (rel.startsWith('tests/hygiene/')) continue
      if (rel === 'Cargo.lock') continue
      if (skipBinaryRel(rel)) continue
      if (rel.includes('.__hygiene_')) continue
      const text = readText(file)
      if (!text) continue
      if (pattern.test(text)) {
        hits.push(rel)
      }
    }
    return hits
  }

  for (const { name, pattern } of classes) {
    test(`no ${name} in product tree`, () => {
      expect(scanHits(pattern)).toEqual([])
    })
  }

  test('self-test: patterns fire on known-bad samples', () => {
    expect(classes[0]!.pattern.test('/home/madvib/dev/Ship')).toBe(true)
    expect(classes[0]!.pattern.test('route /home/page.sail')).toBe(false)
    expect(classes[0]!.pattern.test('path="/home/user/.ssh"')).toBe(true)
    expect(classes[1]!.pattern.test('helm/data/missions/m-deck-standalone/x')).toBe(true)
    expect(classes[2]!.pattern.test('// see job/mount-route-bug')).toBe(true)
    expect(classes[3]!.pattern.test('task #80:')).toBe(true)
    expect(classes[3]!.pattern.test('Task #34 regression')).toBe(true)
    expect(classes[4]!.pattern.test('Owner ruling 2026-07-18')).toBe(true)
  })
})
