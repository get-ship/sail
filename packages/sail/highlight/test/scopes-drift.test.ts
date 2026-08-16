/**
 * Drift: JS scope table must match highlights.scm captures and the
 * Rust SCOPES list (same names, same closed set).
 */
import { readFileSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { describe, expect, test } from 'vitest'
import { SCOPES } from '../src/scopes'

const here = dirname(fileURLToPath(import.meta.url))
const scm = readFileSync(
  resolve(here, '../../grammar/queries/highlights.scm'),
  'utf8',
)
const rust = readFileSync(
  resolve(here, '../../grammar/bindings/rust/highlight.rs'),
  'utf8',
)

function scmCaptures(src: string): string[] {
  const out = new Set<string>()
  const re = /@([a-z][a-z0-9_.]*)/g
  let m: RegExpExecArray | null
  while ((m = re.exec(src))) {
    const name = m[1]!
    if (!name.startsWith('_')) out.add(name)
  }
  return [...out].sort()
}

function rustScopes(src: string): string[] {
  const block = src.split('pub const SCOPES')[1]
  if (!block) return []
  const out: string[] = []
  const re = /"([^"]+)"/g
  let m: RegExpExecArray | null
  while ((m = re.exec(block.split('];')[0] ?? ''))) {
    out.push(m[1]!)
  }
  return out.sort()
}

describe('highlight scope drift', () => {
  test('JS SCOPES equals highlights.scm captures', () => {
    expect([...SCOPES].sort()).toEqual(scmCaptures(scm))
  })

  test('JS SCOPES equals Rust SCOPES', () => {
    expect([...SCOPES].sort()).toEqual(rustScopes(rust))
  })
})
