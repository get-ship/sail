/**
 * Emitter ownership. Every tracked generated file has a named writer
 * and a gate. A new *.gen.ts without a registry row is a third state.
 */
import { describe, expect, test } from 'bun:test'
import { execSync } from 'node:child_process'
import { existsSync, readFileSync } from 'node:fs'
import { resolve } from 'node:path'

const REPO = resolve(import.meta.dir, '../..')
const REG = JSON.parse(readFileSync(resolve(import.meta.dir, 'emitters.json'), 'utf8')) as {
  productOut: string
  forbiddenOut: string[]
  trackedGenerated: Array<{ path: string; emitter: string; gate: string }>
}

const tracked = (glob: string): string[] =>
  execSync(`git ls-files ${glob}`, { encoding: 'utf8', cwd: REPO })
    .trim()
    .split('\n')
    .filter(Boolean)

describe('emitter registry', () => {
  test('deploy is dist; generate is .sail; .deck is not a home', () => {
    expect(REG.productOut).toBe('dist')
    expect(REG.generateOut).toBe('.sail')
    expect(REG.forbiddenOut).toContain('.deck')
    expect(REG.forbiddenOut).toContain('node_modules/.sail')
    expect(REG.forbiddenOut).toContain('.sail-out')
    const layout = readFileSync(resolve(REPO, 'crates/sail/compiler/src/out_layout.rs'), 'utf8')
    expect(layout).toContain('pub const DEFAULT_OUT_DIR: &str = ".sail"')
    expect(layout).not.toMatch(/DEFAULT_OUT_DIR[^\n]*\.deck/)
    const deck = readFileSync(resolve(REPO, 'crates/deck/deck-config/src/lib.rs'), 'utf8')
    expect(deck).toContain('pub const DECK_OUT_DIRNAME: &str = "dist"')
    expect(deck).toContain('pub const SAIL_GENERATE_DIRNAME: &str = ".sail"')
    expect(deck).not.toMatch(/DECK_OUT_DIRNAME[^\n]*\.deck/)
    const ambients = readFileSync(resolve(REPO, 'crates/sail/compiler/src/ambients.rs'), 'utf8')
    expect(ambients).toContain('pub const PROJECT_SAIL_DIR_REL: &str = ".sail"')
    expect(ambients).not.toContain('node_modules/.sail')
  })

  test('every registry path exists and is tracked', () => {
    const missing = REG.trackedGenerated.filter(
      (e) => !existsSync(resolve(REPO, e.path)) || !tracked(e.path).includes(e.path),
    )
    expect(
      missing.map((e) => e.path),
      'registry lists a file that is not tracked',
    ).toEqual([])
  })

  test('every tracked *.gen.ts is in the registry', () => {
    const listed = new Set(REG.trackedGenerated.map((e) => e.path))
    const orphans = tracked("'*.gen.ts' '*.gen.tsx'").filter((p) => !listed.has(p))
    expect(orphans, 'new generated TS without an emitter row').toEqual([])
  })

  test('every emitter row names a writer and a gate', () => {
    const bad = REG.trackedGenerated.filter((e) => !e.emitter || !e.gate)
    expect(bad).toEqual([])
  })

  test('load-bearing site gen scripts expose --check', () => {
    for (const rel of [
      'apps/sail-site/scripts/gen-runtime-surface.ts',
      'apps/sail-site/scripts/gen-tutorial.ts',
    ]) {
      const src = readFileSync(resolve(REPO, rel), 'utf8')
      expect(src.includes('--check'), `${rel} must refuse to be a write-only script`).toBe(true)
    }
    const runtime = readFileSync(
      resolve(REPO, 'apps/sail-site/scripts/gen-runtime-surface.ts'),
      'utf8',
    )
    expect(runtime).toContain('apps/sail-site/gen/runtime-surface.ts')
    expect(runtime).not.toContain('src/runtime-surface.gen.ts')
    expect(runtime).not.toContain('Missing dest is not drift')
  })
})
