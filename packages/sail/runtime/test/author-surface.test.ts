// Tier B allowlist is the public root barrel (TS-1 WI2.5).
// packages/sail/runtime/author-surface.json is the docs/gen SSOT.
// Type-only exports are erased at runtime, so we assert against index.ts source.

import { readFileSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'

const here = dirname(fileURLToPath(import.meta.url))
const author = JSON.parse(
  readFileSync(resolve(here, '../author-surface.json'), 'utf8'),
) as {
  exports: Record<string, string>
  categories: Record<string, string>
}
const indexSrc = readFileSync(resolve(here, '../src/index.ts'), 'utf8')

function rootExportNames(): { value: Set<string>; type: Set<string> } {
  const value = new Set<string>()
  const type = new Set<string>()
  const blocks = indexSrc.matchAll(/export\s+(type\s+)?\{([\s\S]*?)\}\s+from/g)
  for (const m of blocks) {
    const isType = Boolean(m[1])
    for (const part of m[2].split(',')) {
      const cleaned = part.replace(/\/\/.*$/, '').trim()
      if (!cleaned) continue
      const segs = cleaned.split(/\s+as\s+/)
      const name = segs[segs.length - 1]!.trim()
      if (!name) continue
      ;(isType ? type : value).add(name)
    }
  }
  return { value, type }
}

function rootMentions(name: string): boolean {
  return new RegExp(`\\b${name}\\b`).test(indexSrc)
}

describe('author-surface (tier B)', () => {
  it('every allowlisted name appears on the root barrel', () => {
    const missing = Object.keys(author.exports).filter((n) => !rootMentions(n))
    expect(missing, `missing from index.ts:\n  ${missing.join('\n  ')}`).toEqual([])
  })

  it('root barrel exports exactly the allowlist (no untaught public leaks)', () => {
    const { value, type } = rootExportNames()
    const root = new Set([...value, ...type])
    const allow = new Set(Object.keys(author.exports))
    const extra = [...root].filter((n) => !allow.has(n)).sort()
    const missing = [...allow].filter((n) => !root.has(n)).sort()
    expect(extra, `root exports not in author-surface.json:\n  ${extra.join('\n  ')}`).toEqual(
      [],
    )
    expect(
      missing,
      `author-surface.json missing from root exports:\n  ${missing.join('\n  ')}`,
    ).toEqual([])
  })

  it('categories cover every allowlisted export', () => {
    for (const [name, cat] of Object.entries(author.exports)) {
      expect(author.categories[cat], `${name} → unknown category ${cat}`).toBeTypeOf('string')
    }
  })

  it('does not list emit-only ABI symbols', () => {
    for (const abi of [
      'bindFor',
      'bindIf',
      'bindText',
      'runAction',
      'runController',
      '__tpl',
      'wireFormMutation',
      'delegateEvents',
      'mountComponent',
      'selector',
      'withHost',
      'mount', // host route mount — /internal only; ambient uses mountAmbient
      '__invalidate',
      'applyPatch',
      'emit',
      'mockQueryState',
      'writeCssVar',
      'bindCssVar',
    ]) {
      expect(author.exports[abi], `ABI/host symbol ${abi} must not be tier B`).toBeUndefined()
    }
  })

  it('root no longer value-exports wireFormMutation / selector / host mount', () => {
    for (const banned of ['wireFormMutation', 'selector', 'withHost', '__invalidate']) {
      const leaked = indexSrc.split('\n').some((l) => {
        const t = l.trimStart()
        return (
          t.startsWith('export ') &&
          !t.startsWith('export type ') &&
          new RegExp(`\\b${banned}\\b`).test(t)
        )
      })
      expect(leaked, `${banned} must not value-export from root`).toBe(false)
    }
    // Host `mount` must not be the free export name (mountAmbient is the ambient alias).
    const hostMount = indexSrc.split('\n').some((l) => {
      const t = l.trimStart()
      return (
        t.startsWith('export ') &&
        !t.startsWith('export type ') &&
        /\bmount\b/.test(t) &&
        !/\bmountAmbient\b/.test(t)
      )
    })
    expect(hostMount, 'host mount must not value-export from root').toBe(false)
  })
})
