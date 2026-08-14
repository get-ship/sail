/** Repo law: no documentElement.style writes in apps. */
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
  '.static-inventory.json',
  '.static-crawl.json',
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

describe('project Pure gate scripts stay wired', () => {
  // Scripts exist so un-wiring is loud. Do not invoke live `typecheck:pure`
  // from required `test` / type-gate until studio + site corpora are green.
  // L2 `packages/components/sail` is not an app Pure surface.
  const requiredScripts = ['typecheck:studio-pure', 'typecheck:site-pure', 'typecheck:pure']

  test('root package.json exposes app Pure typecheck scripts', () => {
    const pkg = JSON.parse(readText(join(REPO, 'package.json')) ?? '{}') as {
      scripts?: Record<string, string>
    }
    const scripts = pkg.scripts ?? {}
    for (const name of requiredScripts) {
      expect(scripts[name], `missing script ${name}`).toBeDefined()
    }
    expect(scripts['typecheck:studio-pure']).toContain('tsconfig.pure.json')
    expect(scripts['typecheck:site-pure']).toContain('tsconfig.pure.json')
    expect(scripts['typecheck:pure']).toContain('typecheck:studio-pure')
    expect(scripts['typecheck:pure']).toContain('typecheck:site-pure')
    expect(scripts['typecheck:pure'] ?? '').not.toContain('components-pure')
    expect(scripts.test ?? '', 'required test must not invoke live typecheck:pure').not.toContain(
      'typecheck:pure',
    )
  })

  test('CI type-gate keeps corpus builds and does not invoke live typecheck:pure', () => {
    const yml = readText(join(REPO, '.github/workflows/hygiene.yml')) ?? ''
    expect(yml).toContain('App corpus builds')
    expect(yml).toContain('build apps/sail-site/src')
    expect(yml).toContain('build apps/studio/src')
    expect(yml).not.toContain('typecheck:pure')
    expect(yml).not.toContain('typecheck:components-pure')
  })
})
