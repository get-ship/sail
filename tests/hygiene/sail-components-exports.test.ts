/**
 * L2-2 exports cleanup gates:
 * 1. Zero legacy `@ship/sail-<domain>` product import specs (domains are subpaths
 *    of `@ship/sail-components` only).
 * 2. No domain package.json under packages/components/sail/<domain>/.
 * 3. Umbrella exports: no duplicate short aliases; domain + tag paths present.
 */
import { describe, expect, test } from 'bun:test'
import { existsSync, readdirSync, readFileSync, statSync } from 'node:fs'
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
  '.agents',
  '.sail-out',
  'out',
  'runs',
  'eval',
])

const SKIP_FILES = new Set(['CHANGELOG.md', 'CLAUDE.md', 'AGENTS.md', 'GEMINI.md'])

const LEGACY_SPEC =
  /@ship\/sail-(primitives|charts|code-editor|viewer|media|chat-elements|canvas|mermaid)\b/

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

  test('umbrella exports: domains present, short aliases absent', () => {
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

    // One canonical path — no short aliases.
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

    // Utility paths live under their domain.
    expect(keys).toContain('./media/engine')
    expect(keys).toContain('./viewers/wasm-bootstrap')
    expect(keys).toContain('./viewers/same-doc-mount')

    // Per-tag under domain (sample + count).
    expect(keys).toContain('./primitives/sail-calendar')
    expect(keys).toContain('./charts/sail-bar-chart')
    const tags = keys.filter((k) => /\/sail-[a-z0-9-]+$/.test(k))
    expect(tags.length).toBeGreaterThanOrEqual(40)
  })
})
