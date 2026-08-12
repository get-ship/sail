/**
 * G2d / dev-dep boundary — Node and Bun are the build toolchain, never a
 * product runtime dependency.
 *
 * Law:
 * - Product `src` trees under apps/ and packages/ must not import `node:`,
 *   `bun:`, or free `process` (Node ambient). Those belong in test/,
 *   scripts/, and tooling only.
 * - `@ship/sail-runtime` and `@ship/sail-dom` shipped sources are browser-only
 *   (zero node/bun/process). DOM compile harness lives under `dom/test/`.
 *
 * Error message shape (external audience): what / allowed / fix.
 * Wire: `bun test tests/hygiene/node-import-boundary.test.ts`
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
    if (st.isDirectory()) yield* walkFiles(p)
    else if (st.isFile()) yield p
  }
}

function readText(path: string): string | null {
  try {
    return readFileSync(path, 'utf8')
  } catch {
    return null
  }
}

function isProductSrcTs(rel: string, abs: string): boolean {
  if (!PRODUCT_TS.test(abs)) return false
  if (abs.endsWith('.gen.ts') || abs.endsWith('.gen.tsx')) return false
  if (abs.endsWith('.d.ts')) return false
  // Only product `src/**` — not test/, scripts/, e2e/
  if (!rel.includes('/src/')) return false
  if (rel.includes('/test/') || rel.includes('/tests/') || rel.includes('/__tests__/')) {
    return false
  }
  if (/\.(test|spec)\.(ts|tsx|mts|cts)$/.test(abs)) return false
  return true
}

/** `node:fs`, dynamic `import('node:…')`, `from "bun:test"`, etc. */
const NODE_OR_BUN_SPEC =
  /(?:from\s+|import\s*\(\s*|require\s*\(\s*)['"](?:node:|bun:)[^'"]+['"]/

/**
 * Free Node `process` ambient — not CSS `process:` or prose.
 * Matches `process.env`, `process.platform`, bare `process.` member access.
 */
const PROCESS_MEMBER = /(?<![\w$])process\.(env|platform|cwd|argv|versions|stdout|stderr|stdin|exit|dlopen)\b/

function scanHits(
  rootRel: string,
  pattern: RegExp,
  opts: { productSrcOnly?: boolean } = {},
): string[] {
  const root = join(REPO, rootRel)
  if (!existsSync(root)) {
    throw new Error(`${rootRel} missing — node-import boundary cannot skip fixtures`)
  }
  const hits: string[] = []
  for (const file of walkFiles(root)) {
    const rel = relative(REPO, file)
    if (opts.productSrcOnly && !isProductSrcTs(rel, file)) continue
    if (!PRODUCT_TS.test(file) && !file.endsWith('.mts') && !file.endsWith('.cts')) continue
    if (file.endsWith('.d.ts') || file.endsWith('.gen.ts')) continue
    const text = readText(file)
    if (!text) continue
    const lines = text.split('\n')
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i]!
      // Skip pure comment lines (boundary docs may mention node:).
      const trimmed = line.trim()
      if (trimmed.startsWith('//') || trimmed.startsWith('*') || trimmed.startsWith('/*')) {
        continue
      }
      if (pattern.test(line)) {
        hits.push(`${rel}:${i + 1}: ${trimmed}`)
      }
    }
  }
  return hits
}

function boundaryMessage(scope: string, hits: string[]): string {
  return [
    `Node/Bun imports are not allowed in ${scope}.`,
    `Allowed: test/, scripts/, and build tooling only — never product src/ or shipped runtime.`,
    `Fix: move the Node-only module under test/ (or scripts/) and keep Pure/app src free of node:*, bun:*, and process.*.`,
    hits.length ? `Hits:\n${hits.slice(0, 40).join('\n')}${hits.length > 40 ? `\n… +${hits.length - 40} more` : ''}` : '',
  ]
    .filter(Boolean)
    .join('\n')
}

describe('G2d node-import boundary: product src/**', () => {
  test('self-test: patterns match node:/bun:/process forms', () => {
    expect(NODE_OR_BUN_SPEC.test("import { readFileSync } from 'node:fs'")).toBe(true)
    expect(NODE_OR_BUN_SPEC.test('import { test } from "bun:test"')).toBe(true)
    expect(NODE_OR_BUN_SPEC.test("const x = await import('node:path')")).toBe(true)
    expect(NODE_OR_BUN_SPEC.test("import { signal } from '@ship/sail-runtime'")).toBe(false)
    expect(PROCESS_MEMBER.test('env: { ...process.env }')).toBe(true)
    expect(PROCESS_MEMBER.test('process.platform === "darwin"')).toBe(true)
    expect(PROCESS_MEMBER.test('// process.env is Node-only')).toBe(true) // still matches token; comments filtered in scan
    expect(PROCESS_MEMBER.test('postprocess.env')).toBe(false)
  })

  test('apps/**/src product TS has zero node:/bun: imports and process.*', () => {
    const nodeHits = scanHits('apps', NODE_OR_BUN_SPEC, { productSrcOnly: true })
    const procHits = scanHits('apps', PROCESS_MEMBER, { productSrcOnly: true })
    const hits = [...nodeHits, ...procHits]
    expect(hits, boundaryMessage('apps/**/src (product TypeScript)', hits)).toEqual([])
  })

  test('browser product package src has zero node:/bun: imports and process.*', () => {
    // Node servers (deck/hono, deck/host) and test tooling (sail/test-setup)
    // are allowed Node deps. Browser product packages are not.
    const browserRoots = [
      'packages/sail/runtime/src',
      'packages/sail/dom/src',
      'packages/sail/studio-shell/src',
      'packages/sail/studio-client/src',
      'packages/sail/studio-bridge/src',
      'packages/sail/tokens/src',
      'packages/components/sail',
      'packages/deck/router/src',
      'packages/deck/lsp-browser/src',
    ]
    const hits: string[] = []
    for (const root of browserRoots) {
      if (!existsSync(join(REPO, root))) continue
      hits.push(
        ...scanHits(root, NODE_OR_BUN_SPEC, { productSrcOnly: true }),
        ...scanHits(root, PROCESS_MEMBER, { productSrcOnly: true }),
      )
    }
    expect(
      hits,
      boundaryMessage('browser product package src (runtime, dom, studio-*, tokens, components, router)', hits),
    ).toEqual([])
  })
})

describe('G2d zero-node shipped runtime: sail-runtime + sail-dom', () => {
  test('@ship/sail-runtime/src has zero node:/bun:/process', () => {
    const root = 'packages/sail/runtime/src'
    const hits = [
      ...scanHits(root, NODE_OR_BUN_SPEC),
      ...scanHits(root, PROCESS_MEMBER),
    ]
    expect(
      hits,
      boundaryMessage(
        'packages/sail/runtime/src (shipped browser runtime — node is build toolchain only)',
        hits,
      ),
    ).toEqual([])
  })

  test('@ship/sail-dom/src has zero node:/bun:/process', () => {
    const root = 'packages/sail/dom/src'
    const hits = [
      ...scanHits(root, NODE_OR_BUN_SPEC),
      ...scanHits(root, PROCESS_MEMBER),
    ]
    expect(
      hits,
      boundaryMessage(
        'packages/sail/dom/src (shipped browser DOM interpreter — harness is under dom/test/)',
        hits,
      ),
    ).toEqual([])
  })
})
