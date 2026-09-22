/**
 * SPEC-PLATFORM-NEUTRALITY-1 §4 / SPIKE S2.2 — corpus residual inventory.
 *
 * Pure type envs make document/window/localStorage unnameable in app
 * compositions/controllers/stores, and product code must not read DOM
 * dataset buses. These gates fail while residual product sources remain.
 *
 * Zero allowlist: scrub the corpus; do not carve exceptions here.
 * Wire: `bun test tests/hygiene/platform-neutrality-corpus.test.ts`
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

/** Product TypeScript under app src (not generated dumps, not tests). */
const PRODUCT_TS = /\.(ts|tsx|mts|cts)$/

function* walkFiles(dir: string): Generator<string> {
  let entries: string[]
  try {
    entries = readdirSync(dir)
  } catch {
    return
  }
  for (const entry of entries) {
    if (SKIP_DIRS.has(entry)) continue
    if (entry.startsWith('.')) continue
    const p = join(dir, entry)
    let st
    try {
      st = statSync(p)
    } catch {
      continue
    }
    if (st.isDirectory()) {
      yield* walkFiles(p)
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

function isProductTs(rel: string, abs: string): boolean {
  if (!PRODUCT_TS.test(abs)) return false
  if (abs.endsWith('.gen.ts') || abs.endsWith('.gen.tsx')) return false
  if (abs.endsWith('.d.ts')) return false
  if (rel.includes('/test/') || rel.includes('/tests/') || rel.includes('/__tests__/')) {
    return false
  }
  if (/\.(test|spec)\.(ts|tsx|mts|cts)$/.test(abs)) return false
  return true
}

/** Ambient god-globals banned under Pure product compositions / product TS. */
const AMBIENT =
  /\bdocument\.|\bwindow\.|\blocalStorage\b|\bsessionStorage\b/

/**
 * Residual dataset bus reads (substrate smuggle / wrong for non-data-* attrs).
 * Matches `.dataset.foo` and `dataset['foo']` forms only.
 */
const DATASET_READ = /\.dataset\.|\.dataset\[|dataset\[/

function scanHits(
  rootRel: string,
  pattern: RegExp,
  opts: { productTsOnly?: boolean } = {},
): string[] {
  const root = join(REPO, rootRel)
  if (!existsSync(root)) {
    throw new Error(`${rootRel} missing — corpus inventory cannot skip fixtures`)
  }
  const hits: string[] = []
  for (const file of walkFiles(root)) {
    const rel = relative(REPO, file)
    if (opts.productTsOnly && !isProductTs(rel, file)) continue
    const text = readText(file)
    if (!text) continue
    const lines = text.split('\n')
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i]!
      if (pattern.test(line)) {
        hits.push(`${rel}:${i + 1}: ${line.trim()}`)
      }
    }
  }
  return hits
}

describe('S2.2 platform-neutrality corpus: no ambient DOM in sail-site product TS', () => {
  const compositions = 'apps/sail-site/src/compositions'
  const siteSrc = 'apps/sail-site/src'

  test('self-test: ambient pattern matches document/window/localStorage forms', () => {
    expect(AMBIENT.test('document.createElement("div")')).toBe(true)
    expect(AMBIENT.test('window.addEventListener("popstate", fn)')).toBe(true)
    expect(AMBIENT.test('localStorage.getItem("k")')).toBe(true)
    expect(AMBIENT.test('sessionStorage.setItem("k", "v")')).toBe(true)
    expect(AMBIENT.test('typeof document === "undefined"')).toBe(false)
    expect(AMBIENT.test('// no document reload')).toBe(false)
  })

  test('apps/sail-site/src/compositions has zero document./window./localStorage', () => {
    const hits = scanHits(compositions, AMBIENT, { productTsOnly: true })
    expect(
      hits,
      `ambient residual in sail-site compositions (scrub for Pure type-env):\n${hits.slice(0, 40).join('\n')}${hits.length > 40 ? `\n… +${hits.length - 40} more` : ''}`,
    ).toEqual([])
  })

  test('apps/sail-site/src product TS has zero document./window./localStorage', () => {
    // Broader product TS surface (compositions + other pure modules under src).
    // Generated *.gen.ts and tests are excluded.
    const hits = scanHits(siteSrc, AMBIENT, { productTsOnly: true })
    expect(
      hits,
      `ambient residual in sail-site product TS:\n${hits.slice(0, 40).join('\n')}${hits.length > 40 ? `\n… +${hits.length - 40} more` : ''}`,
    ).toEqual([])
  })
})

describe('S2.2 platform-neutrality corpus: no dataset bus reads in studio product', () => {
  const studioSrc = 'apps/studio/src'

  test('self-test: dataset pattern matches reads, not prose without .dataset.', () => {
    expect(DATASET_READ.test('btn?.dataset.ruleToggle')).toBe(true)
    expect(DATASET_READ.test('el.dataset["mode"]')).toBe(true)
    expect(DATASET_READ.test('const x = el.dataset.mode')).toBe(true)
    // Teaching comments without member access form are not product reads.
    expect(DATASET_READ.test('// dataset.l1 never sees plain attrs')).toBe(false)
    expect(DATASET_READ.test('expect(vm).not.toMatch(/dataset\\.themeId/)')).toBe(false)
  })

  test('apps/studio/src product code has zero dataset. reads', () => {
    const hits = scanHits(studioSrc, DATASET_READ, { productTsOnly: true })
    expect(
      hits,
      `dataset residual in studio product src (use closed-over handlers / API):\n${hits.slice(0, 40).join('\n')}${hits.length > 40 ? `\n… +${hits.length - 40} more` : ''}`,
    ).toEqual([])
  })
})
