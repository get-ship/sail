/**
 * Drift guard: freeze annexes A–E must match live manifests.
 * Run: bun test packages/sail/freeze-annexes/freeze-annexes.test.ts
 */
import { describe, expect, test } from 'bun:test'
import { spawnSync } from 'node:child_process'
import { resolve } from 'node:path'
import { readFileSync } from 'node:fs'

const REPO = resolve(import.meta.dir, '../../..')

describe('freeze annexes', () => {
  test('gen-freeze-annexes --check is clean', () => {
    const r = spawnSync('node', ['scripts/gen-freeze-annexes.mjs', '--check'], {
      cwd: REPO,
      encoding: 'utf8',
    })
    expect(r.status, r.stderr || r.stdout).toBe(0)
  })

  test('index counts match annex files', () => {
    const index = JSON.parse(
      readFileSync(resolve(import.meta.dir, 'index.json'), 'utf8'),
    )
    const a = JSON.parse(readFileSync(resolve(import.meta.dir, 'a-l1-elements.json'), 'utf8'))
    const b = JSON.parse(readFileSync(resolve(import.meta.dir, 'b-tokens.json'), 'utf8'))
    const c = JSON.parse(readFileSync(resolve(import.meta.dir, 'c-public-actions.json'), 'utf8'))
    const d = JSON.parse(readFileSync(resolve(import.meta.dir, 'd-runtime-exports.json'), 'utf8'))
    const e = JSON.parse(
      readFileSync(resolve(import.meta.dir, 'e-channel-vocabulary.json'), 'utf8'),
    )
    expect(index.annexes.A.count).toBe(a.count)
    expect(index.annexes.B.count).toBe(b.count)
    expect(index.annexes.C.count).toBe(c.count)
    expect(index.annexes.D.count).toBe(d.count)
    expect(index.annexes.E.count).toBe(e.setterCount)
    expect(a.count).toBeGreaterThan(40)
    expect(b.count).toBeGreaterThan(300)
    expect(c.count).toBe(11)
    expect(d.count).toBeGreaterThan(100)
    expect(e.setterCount).toBeGreaterThan(100)
    expect(c.manifestDrift.surfaceOnly).toEqual([])
    expect(c.manifestDrift.manifestOnly).toEqual([])
  })
})
