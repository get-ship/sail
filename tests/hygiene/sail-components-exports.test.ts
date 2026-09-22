/**
 * L2-2 exports cleanup gates:
 * 1. Zero legacy `@ship/sail-<domain>` product import specs (domains are subpaths
 *    of `@ship/sail-components` only).
 * 2. No domain package.json under packages/components/sail/<domain>/.
 * 3. Umbrella exports: no short aliases; domain-prefixed tags; every export
 *    target exists on disk and is covered by package `files`.
 */
import { describe, expect, test } from 'bun:test'
import { existsSync, readdirSync, readFileSync, statSync } from 'node:fs'
import { join, matchesGlob, relative, resolve } from 'node:path'

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
  '.agents',
  '.sail-out',
  'out',
  'runs',
  'eval',
])

const SKIP_FILES = new Set(['CHANGELOG.md', 'CLAUDE.md', 'AGENTS.md', 'GEMINI.md'])

const LEGACY_SPEC =
  /@ship\/sail-(primitives|charts|code-editor|viewer|media|chat-elements|canvas|mermaid)\b/

/** Export key prefixes that own domain-prefixed tags. */
const DOMAIN_PREFIXES = [
  './primitives/',
  './charts/',
  './code/',
  './viewers/',
  './media/',
  './chat/',
  './canvas/',
  './mermaid/',
] as const

function* walkFiles(dir: string): Generator<string> {
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
    if (st.isDirectory()) yield* walkFiles(p)
    else if (st.isFile()) yield p
  }
}

function isTexty(path: string): boolean {
  return /\.(ts|tsx|js|mjs|cjs|json|jsonc|sail|md|rs|toml|txt|html|css|sh)$/i.test(path)
}

/** Collect package-relative export targets (`types` + `default` + bare string). */
function exportTargets(entry: unknown): string[] {
  if (typeof entry === 'string') return [entry]
  if (entry == null || typeof entry !== 'object') return []
  const o = entry as Record<string, unknown>
  const out: string[] = []
  for (const [k, v] of Object.entries(o)) {
    if (typeof v === 'string') {
      // Condition keys (types/default/import/…) or nested string targets.
      if (k === 'types' || k === 'default' || k === 'import' || k === 'require' || k === 'node' || k === 'browser') {
        out.push(v)
      }
    } else if (v && typeof v === 'object') {
      out.push(...exportTargets(v))
    }
  }
  return out
}

/** True when a package-relative path is included by any `files` glob. */
function coveredByFiles(relPath: string, filesGlobs: string[]): boolean {
  const norm = relPath.replace(/^\.\//, '')
  // npm always packs these even when `files` is set.
  if (norm === 'package.json' || norm === 'README.md' || norm === 'LICENSE') return true
  return filesGlobs.some((g) => {
    const pattern = g.replace(/^\.\//, '')
    return matchesGlob(norm, pattern)
  })
}

describe('@ship/sail-components exports cleanup', () => {
  test('no domain package.json under packages/components/sail/*/', () => {
    const sailRoot = join(REPO, 'packages/components/sail')
    const offenders: string[] = []
    for (const entry of readdirSync(sailRoot)) {
      const domainPkg = join(sailRoot, entry, 'package.json')
      if (existsSync(domainPkg)) offenders.push(relative(REPO, domainPkg))
    }
    expect(offenders, `domain package.json must die: ${offenders.join(', ')}`).toEqual([])
  })

  test('zero legacy @ship/sail-<domain> product specs', () => {
    const offenders: string[] = []
    for (const file of walkFiles(REPO)) {
      if (!isTexty(file)) continue
      const rel = relative(REPO, file)
      if (rel === 'bun.lock' || rel.endsWith('.tgz')) continue
      // Historical changelog only — not product import surface.
      if (rel === 'CHANGELOG.md') continue
      let text: string
      try {
        text = readFileSync(file, 'utf8')
      } catch {
        continue
      }
      if (!LEGACY_SPEC.test(text)) continue
      // Reset lastIndex for global-less test + collect line numbers
      const lines = text.split('\n')
      for (let i = 0; i < lines.length; i++) {
        if (LEGACY_SPEC.test(lines[i]!)) {
          offenders.push(`${rel}:${i + 1}: ${lines[i]!.trim().slice(0, 120)}`)
        }
      }
    }
    expect(
      offenders,
      `legacy domain package specs remain:\n${offenders.slice(0, 40).join('\n')}`,
    ).toEqual([])
  })

  test('umbrella exports: domains present, short aliases absent, tags domain-prefixed', () => {
    const pkgPath = join(REPO, 'packages/components/sail/package.json')
    const pkg = JSON.parse(readFileSync(pkgPath, 'utf8')) as {
      name: string
      exports: Record<string, unknown>
      sailComponents?: boolean
    }
    expect(pkg.name).toBe('@ship/sail-components')
    expect(pkg.sailComponents).toBe(true)

    const keys = Object.keys(pkg.exports)
    for (const domain of [
      './primitives',
      './charts',
      './code',
      './viewers',
      './media',
      './chat',
      './canvas',
      './mermaid',
    ]) {
      expect(keys, `missing domain export ${domain}`).toContain(domain)
    }

    // One canonical path — no short aliases (sample + any bare ./sail-*).
    const banned = [
      './engine',
      './wasm-bootstrap',
      './same-doc-mount',
      './sail-calendar',
      './sail-bar-chart',
      './sail-code-editor',
      './sail-mermaid',
    ]
    for (const b of banned) {
      expect(keys, `duplicate short alias must be deleted: ${b}`).not.toContain(b)
    }
    const bareSail = keys.filter((k) => /^\.\/sail-[a-z0-9-]+$/.test(k))
    expect(bareSail, `bare ./sail-* short aliases forbidden: ${bareSail.join(', ')}`).toEqual([])

    // Utility paths live under their domain.
    expect(keys).toContain('./media/engine')
    expect(keys).toContain('./viewers/wasm-bootstrap')
    expect(keys).toContain('./viewers/same-doc-mount')

    // Every tag key must be domain-prefixed (not a bare short path).
    const tags = keys.filter((k) => /\/sail-[a-z0-9-]+$/.test(k))
    expect(tags.length).toBeGreaterThanOrEqual(40)
    const unprefixed = tags.filter((k) => !DOMAIN_PREFIXES.some((p) => k.startsWith(p)))
    expect(
      unprefixed,
      `tag exports must use domain prefix (./primitives|charts|…/sail-*): ${unprefixed.join(', ')}`,
    ).toEqual([])
  })

  test('export targets exist on disk and are covered by package files', () => {
    const pkgDir = join(REPO, 'packages/components/sail')
    const pkgPath = join(pkgDir, 'package.json')
    const pkg = JSON.parse(readFileSync(pkgPath, 'utf8')) as {
      files: string[]
      exports: Record<string, unknown>
    }
    expect(Array.isArray(pkg.files) && pkg.files.length > 0).toBe(true)

    const missing: string[] = []
    const uncovered: string[] = []
    for (const [key, entry] of Object.entries(pkg.exports)) {
      if (key === './package.json') continue
      for (const target of exportTargets(entry)) {
        const rel = target.replace(/^\.\//, '')
        const abs = join(pkgDir, rel)
        if (!existsSync(abs)) {
          missing.push(`${key} → ${target}`)
          continue
        }
        if (!coveredByFiles(rel, pkg.files)) {
          uncovered.push(`${key} → ${target} (not matched by files)`)
        }
      }
    }
    expect(missing, `export targets missing on disk:\n${missing.join('\n')}`).toEqual([])
    expect(
      uncovered,
      `export targets not covered by package files (tarball break):\n${uncovered.join('\n')}`,
    ).toEqual([])
  })
})
