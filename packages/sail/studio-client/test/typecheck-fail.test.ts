/**
 * Prove wrong arg types fail tsc via @ts-expect-error in type-errors.ts.
 */
import { describe, expect, it } from 'vitest'
import { spawnSync } from 'node:child_process'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const root = path.dirname(fileURLToPath(import.meta.url))
const pkg = path.join(root, '..')
const tsc = path.join(pkg, 'node_modules', '.bin', 'tsc')

describe('studio-client type contracts', () => {
  it('type-errors.ts: three wrong-arg calls are type errors (@ts-expect-error)', () => {
    // Green means @ts-expect-error suppressions are used (wrong args rejected).
    // If types become too loose, unused @ts-expect-error → tsc non-zero.
    const r = spawnSync(tsc, ['-p', 'tsconfig.json', '--noEmit', '--pretty', 'false'], {
      cwd: pkg,
      encoding: 'utf8',
    })
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
  })
})
