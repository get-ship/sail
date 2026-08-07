/**
 * Prove wrong arg types fail tsc via @ts-expect-error in type-errors.ts,
 * and that surface round-trip type files accept store arg/ret shapes.
 */
import { describe, expect, it } from 'vitest'
import { existsSync } from 'node:fs'
import { spawnSync } from 'node:child_process'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const root = path.dirname(fileURLToPath(import.meta.url))
const pkg = path.join(root, '..')
const repoRoot = path.join(pkg, '..', '..', '..')

function resolveTsc(): string {
  const candidates = [
    path.join(pkg, 'node_modules', '.bin', 'tsc'),
    path.join(repoRoot, 'node_modules', '.bin', 'tsc'),
  ]
  for (const c of candidates) {
    if (existsSync(c)) return c
  }
  return 'tsc'
}

describe('studio-client type contracts', () => {
  it('type-errors.ts + surface-roundtrip-types.ts typecheck', () => {
    // Green means:
    // - @ts-expect-error suppressions in type-errors.ts are used (wrong args rejected)
    // - surface round-trips accept store arg/ret shapes
    // Deleting a METHOD_CONTRACTS method fails this test (round-trip Pick breaks).
    const r = spawnSync(
      resolveTsc(),
      ['-p', 'tsconfig.json', '--noEmit', '--pretty', 'false'],
      { cwd: pkg, encoding: 'utf8' },
    )
    if (r.status !== 0) {
      throw new Error(`tsc failed (status ${r.status}):\n${r.stdout}\n${r.stderr}`)
    }
    expect(r.status).toBe(0)
  })

  it('documents that callRpc + typed methods exist', async () => {
    const mod = await import('../src/index')
    expect(typeof mod.callRpc).toBe('function')
    expect(typeof mod.ops).toBe('function')
    expect(typeof mod.sailAuthoringSurface).toBe('function')
    expect(typeof mod.sailThemeList).toBe('function')
    expect(typeof mod.watch).toBe('function')
    expect(typeof mod.sailThemeFontSearch).toBe('function')
    expect(typeof mod.sailAuthoringEdit).toBe('function')
  })
})
