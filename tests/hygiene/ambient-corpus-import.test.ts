/**
 * SPEC-AMBIENT-SURFACE-1 §2–3 / SPIKE S5.2 + S5.3/E3 — corpus import law +
 * tutorial package die + deep packages/ facade law.
 *
 * Law:
 * 1. `@ship/sail-tutorial` package is gone (content folded into sail-site).
 * 2. App product sources have zero **value** / side-effect imports from `@ship/*`.
 *    Type-only imports (`import type`, `import { type X }`) remain free.
 * 3. Tutorial content still lives under sail-site (lessons or generated data).
 * 4. App product sources (outside `src/vendor/`) have zero deep `packages/`
 *    path imports — third-party / monorepo surface is wrapped in app-local
 *    vendor facades only (S5.2 REVISE + S5.3/E3).
 *
 * Zero allowlist for product apps. Tests/e2e/qa/scripts are out of scope here
 * (same path policy as platform-neutrality-corpus).
 *
 * Wire: `bun test tests/hygiene/ambient-corpus-import.test.ts`
 * Implementer must not skip or weaken the S5.2 gates. E3 may add the deep-import
 * case only.
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
 * Value / side-effect import lines from @ship/*. Type-only free.
 * Mirrors sail-framework-import-teaching intent (string-level corpus gate).
 */
function valueShipImportLines(src: string): string[] {
  const body = stripComments(src)
  const hits: string[] = []
  for (const line of body.split('\n')) {
    const t = line.trim()
    if (!t.includes('@ship/')) continue
    // whole-decl type import
    if (/^import\s+type\s/.test(t)) continue
    // side-effect or value import with @ship
    const m = t.match(
      /(?:import\s+(?!type\b)[\s\S]*?\sfrom\s+|import\s+)['"](@ship\/[^'"]+)['"]/,
    )
    if (!m) continue
    // import { type Foo, type Bar } from '@ship/…' — all type-only
    if (/^import\s*\{[^}]*\}\s*from/.test(t)) {
      const brace = t.match(/^import\s*\{([^}]*)\}/)
      if (brace) {
        const specs = brace[1]
          .split(',')
          .map((s) => s.trim())
          .filter(Boolean)
        if (specs.length > 0 && specs.every((s) => s.startsWith('type ') || s === 'type')) {
          continue
        }
      }
    }
    hits.push(t.slice(0, 160))
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

describe('S5.2 app corpus zero @ship value imports', () => {
  test('apps product sources have no value/side-effect @ship/* imports', () => {
    const appsRoot = join(REPO, 'apps')
    const offenders: string[] = []
    for (const abs of walkFiles(appsRoot)) {
      const rel = relative(REPO, abs).replace(/\\/g, '/')
      if (isTestOrToolPath(rel)) continue
      // generated language surface embeds examples; still product — must be clean
      let src: string
      try {
        src = readFileSync(abs, 'utf8')
      } catch {
        continue
      }
      const hits = valueShipImportLines(src)
      for (const h of hits) {
        offenders.push(`${rel}: ${h}`)
      }
    }
    expect(offenders).toEqual([])
  })
})

/**
 * Deep monorepo path imports into packages/ (value, type, or side-effect).
 * Mirrors sail-framework-import-teaching facade case (string-level corpus gate).
 * Scans joined source so multiline `import { … }\nfrom '…packages/…'` cannot
 * bypass the ratchet by putting the specifier on a non-import-leading line.
 * Vendor facades under apps/.../src/vendor/ are the legal packaging seam.
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

// Legal packaging seam only: apps/*/src/vendor/** (aligned with lint rule).
function isVendorFacadePath(rel: string): boolean {
  const p = rel.replace(/\\/g, '/')
  return p.includes('/src/vendor/')
}

describe('S5.3/E3 app corpus zero deep packages/ imports', () => {
  test('apps product sources (outside src/vendor) have no deep packages/ path imports', () => {
    const appsRoot = join(REPO, 'apps')
    const offenders: string[] = []
    for (const abs of walkFiles(appsRoot)) {
      const rel = relative(REPO, abs).replace(/\\/g, '/')
      if (isTestOrToolPath(rel)) continue
      if (isVendorFacadePath(rel)) continue
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

  test('vendor facades exist as the legal packaging seam', () => {
    // At least one app carries src/vendor (S5.2 REVISE shape).
    const siteVendor = join(REPO, 'apps/sail-site/src/vendor')
    const studioVendor = join(REPO, 'apps/studio/src/vendor')
    expect(existsSync(siteVendor) || existsSync(studioVendor)).toBe(true)
  })
})
