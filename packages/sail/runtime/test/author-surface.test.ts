// Tier B allowlist must stay a subset of the root barrel text.
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

function rootMentions(name: string): boolean {
  // Value or type re-export line containing the identifier as a word token.
  return new RegExp(`\\b${name}\\b`).test(indexSrc)
}

describe('author-surface (tier B)', () => {
  it('every allowlisted name appears on the root barrel', () => {
    const missing = Object.keys(author.exports).filter((n) => !rootMentions(n))
    expect(missing, `missing from index.ts:\n  ${missing.join('\n  ')}`).toEqual([])
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
      '__tpl',
      'wireFormMutation',
      'delegateEvents',
      'mountComponent',
    ]) {
      expect(author.exports[abi], `ABI symbol ${abi} must not be tier B`).toBeUndefined()
    }
  })

  it('root no longer value-exports wireFormMutation (tier C /internal only)', () => {
    const leaked = indexSrc.split('\n').some((l) => {
      const t = l.trimStart()
      return (
        t.startsWith('export ') &&
        !t.startsWith('export type ') &&
        /\bwireFormMutation\b/.test(t)
      )
    })
    expect(leaked).toBe(false)
  })
})
