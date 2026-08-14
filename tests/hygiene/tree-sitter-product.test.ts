/**
 * Thread J — tree-sitter is not the product parser.
 * Compiler parse / LSP / fmt / Studio overlay must not import it.
 * packages/sail/grammar may keep the sidecar; nothing else loads it.
 */
import { describe, expect, test } from 'bun:test'
import { existsSync, readdirSync, readFileSync, statSync } from 'node:fs'
import { join, relative, resolve } from 'node:path'

const REPO = resolve(import.meta.dir, '../..')
const SKIP_DIRS = new Set([
  'node_modules',
  'target',
  'dist',
  '.deck',
  '.artifacts',
  '.git',
  'grammar',
])

const NEEDLES = [
  'tree-sitter-sail',
  'tree_sitter_sail',
  'web-tree-sitter',
  'highlights.scm',
  'injections.scm',
]

function* walk(dir: string): Generator<string> {
  let entries: string[]
  try {
    entries = readdirSync(dir)
  } catch {
    return
  }
  for (const entry of entries) {
    if (SKIP_DIRS.has(entry) || entry.startsWith('.')) continue
    const p = join(dir, entry)
    let st
    try {
      st = statSync(p)
    } catch {
      continue
    }
    if (st.isDirectory()) yield* walk(p)
    else if (st.isFile() && /\.(rs|ts|tsx|js|mjs)$/.test(entry)) yield p
  }
}

describe('tree-sitter is not a product import', () => {
  test('product trees do not load tree-sitter-sail or its queries', () => {
    const hits: string[] = []
    for (const rootRel of ['apps', 'packages', 'crates']) {
      const root = join(REPO, rootRel)
      if (!existsSync(root)) continue
      for (const file of walk(root)) {
        const rel = relative(REPO, file)
        if (rel.startsWith('packages/sail/grammar/')) continue
        if (rel.includes('/test/') || rel.includes('/tests/')) continue
        if (/\.(test|spec)\.(ts|tsx|rs)$/.test(rel)) continue
        const text = readFileSync(file, 'utf8')
        for (const line of text.split('\n')) {
          const t = line.trim()
          if (t.startsWith('//') || t.startsWith('*') || t.startsWith('/*')) continue
          if (NEEDLES.some((n) => line.includes(n))) {
            hits.push(`${rel}: ${t}`)
          }
        }
      }
    }
    expect(hits, `product must not import tree-sitter:\n${hits.join('\n')}`).toEqual([])
  })
})
