/**
 * SPEC-AMBIENT-SURFACE-1 §2–3 / SPIKE S5.2 + S5.3/E3 / G2b — corpus import law +
 * tutorial package die + deep packages/ ban + L2 package consumption.
 *
 * Law:
 * 1. `@ship/sail-tutorial` package is gone (content folded into sail-site).
 * 2. Controllers stay ambient for framework value imports (lint hard-wall).
 *    Plain app modules consume monorepo surface as `@ship/*` packages (G2b L2).
 * 3. Tutorial content still lives under sail-site (lessons or generated data).
 * 4. App product sources have zero deep `packages/` path imports — use
 *    `@ship/<name>` package exports. Vendor facades are dead (G2b).
 *
 * Zero allowlist for product apps. Tests/e2e/qa/scripts are out of scope here
 * (same path policy as platform-neutrality-corpus).
 *
 * Wire: `bun test tests/hygiene/ambient-corpus-import.test.ts`
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
  '__snapshots__',
])

const PRODUCT_EXT = /\.(ts|tsx|mts|cts|sail)$/

function* walkFiles(dir: string): Generator<string> {
  let entries: string[]
  try {
    entries = readdirSync(dir)
  } catch {
    return
  }
  for (const entry of entries) {
    if (SKIP_DIRS.has(entry)) continue
    const p = join(dir, entry)
    let st
    try {
      st = statSync(p)
    } catch {
      continue
    }
    if (st.isDirectory()) {
      yield* walkFiles(p)
    } else if (st.isFile() && PRODUCT_EXT.test(entry)) {
      yield p
    }
  }
}

function isTestOrToolPath(rel: string): boolean {
  const p = rel.replace(/\\/g, '/')
  return (
    p.includes('/test/') ||
    p.includes('/e2e/') ||
    p.includes('/qa/') ||
    p.includes('/scripts/') ||
    p.includes('.test.') ||
    p.includes('.spec.') ||
    p.includes('/__tests__/')
  )
}

/** Strip // and /* *\/ comments roughly so string samples don't false-positive. */
function stripComments(src: string): string {
  return src
    .replace(/\/\*[\s\S]*?\*\//g, '')
    .replace(/(^|[^:])\/\/[^\n]*/g, '$1')
}

/**
 * Deep monorepo path imports into packages/ (value, type, or side-effect).
 * Monorepo-only packaging dual-path gate (product lint deliberately does not own this).
 * Scans joined source so multiline `import { … }\nfrom '…packages/…'` cannot
 * bypass the ratchet by putting the specifier on a non-import-leading line.
 */
function deepPackagesImportLines(src: string): string[] {
  const body = stripComments(src)
  const hits: string[] = []
  // Multiline-safe: match import/export … from '…packages/…' across newlines.
  const re =
    /(?:import\s+(?:type\s+)?[\s\S]*?\sfrom\s+|export\s+(?:type\s+)?[\s\S]*?\sfrom\s+|import\s+|export\s+\*\s+from\s+)['"]([^'"]*packages\/[^'"]+)['"]/g
  let m: RegExpExecArray | null
  while ((m = re.exec(body)) !== null) {
    hits.push(m[0].replace(/\s+/g, ' ').slice(0, 160))
  }
  return hits
}

describe('S5.2 tutorial package dies', () => {
  test('packages/sail/tutorial package directory is gone', () => {
    const pkgJson = join(REPO, 'packages/sail/tutorial/package.json')
    expect(existsSync(pkgJson)).toBe(false)
  })

  test('root workspaces do not list packages/sail/tutorial', () => {
    const root = JSON.parse(readFileSync(join(REPO, 'package.json'), 'utf8')) as {
      workspaces?: { packages?: string[] } | string[]
    }
    const list = Array.isArray(root.workspaces)
      ? root.workspaces
      : (root.workspaces?.packages ?? [])
    expect(list).not.toContain('packages/sail/tutorial')
  })

  test('apps do not depend on @ship/sail-tutorial', () => {
    const hits: string[] = []
    for (const app of ['apps/sail-site', 'apps/studio']) {
      const pj = join(REPO, app, 'package.json')
      if (!existsSync(pj)) continue
      const text = readFileSync(pj, 'utf8')
      if (text.includes('@ship/sail-tutorial')) hits.push(app)
    }
    expect(hits).toEqual([])
  })

  test('tutorial content folded into sail-site (lessons or committed tutorial-data)', () => {
    const lessons = join(REPO, 'apps/sail-site/content/tutorial/lessons')
    const prose = join(REPO, 'apps/sail-site/content/tutorial/prose')
    const data = join(REPO, 'apps/sail-site/src/compositions/tutorial-data.ts')
    const hasLessons = existsSync(lessons) && existsSync(prose)
    const hasData = existsSync(data)
    expect(hasLessons || hasData).toBe(true)
  })
})

describe('G2b app corpus L2 package consumption (no vendor facades)', () => {
  test('apps have no src/vendor packaging seam', () => {
    const siteVendor = join(REPO, 'apps/sail-site/src/vendor')
    const studioVendor = join(REPO, 'apps/studio/src/vendor')
    expect(existsSync(siteVendor)).toBe(false)
    expect(existsSync(studioVendor)).toBe(false)
  })

  test('apps product sources have no deep packages/ path imports', () => {
    const appsRoot = join(REPO, 'apps')
    const offenders: string[] = []
    for (const abs of walkFiles(appsRoot)) {
      const rel = relative(REPO, abs).replace(/\\/g, '/')
      if (isTestOrToolPath(rel)) continue
      let src: string
      try {
        src = readFileSync(abs, 'utf8')
      } catch {
        continue
      }
      const hits = deepPackagesImportLines(src)
      for (const h of hits) {
        offenders.push(`${rel}: ${h}`)
      }
    }
    expect(offenders).toEqual([])
  })
})
