/**
 * Prove channel-type-errors.ts typechecks (wrong ops rejected via @ts-expect-error).
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

describe('studio-shell channel type contracts', () => {
  it('channel-type-errors.ts typechecks (unknown ops fail compile)', () => {
    const r = spawnSync(
      resolveTsc(),
      [
        '--noEmit',
        '--strict',
        '--module',
        'ESNext',
        '--moduleResolution',
        'bundler',
        '--target',
        'ES2022',
        '--lib',
        'ES2022,DOM',
        '--skipLibCheck',
        'src/channel.ts',
        'src/channel-type-errors.ts',
      ],
      { cwd: pkg, encoding: 'utf8' },
    )
    if (r.status !== 0) {
      throw new Error(`tsc failed (status ${r.status}):\n${r.stdout}\n${r.stderr}`)
    }
    expect(r.status).toBe(0)
  })
})
