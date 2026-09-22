/**
 * Drift gates that do not need cargo / stdlib.
 * prop-registry stays off this file while L1 is moving stdlib.
 */
import { describe, expect, test } from 'bun:test'
import { spawnSync } from 'node:child_process'
import { resolve } from 'node:path'

const REPO = resolve(import.meta.dir, '../..')

function check(script: string): void {
  const r = spawnSync('bun', [script, '--check'], {
    cwd: REPO,
    encoding: 'utf8',
  })
  expect(r.status, r.stderr || r.stdout || `${script} --check failed`).toBe(0)
}

describe('generated surfaces --check', () => {
  test(
    'runtime-surface allowlist matches the runtime barrel',
    () => {
      check('apps/sail-site/scripts/gen-runtime-surface.ts')
    },
    { timeout: 30_000 },
  )

  test(
    'runtime-surface --check fails when dest is absent',
    () => {
      const missing = resolve(REPO, 'apps/sail-site/gen/.__hygiene_absent-runtime-surface.ts')
      const r = spawnSync(
        'bun',
        ['apps/sail-site/scripts/gen-runtime-surface.ts', '--check', '--out', missing],
        { cwd: REPO, encoding: 'utf8' },
      )
      expect(r.status, r.stderr || r.stdout || 'expected non-zero for missing dest').not.toBe(0)
    },
    { timeout: 30_000 },
  )

  test(
    'tutorial-data.ts matches content/tutorial',
    () => {
      check('apps/sail-site/scripts/gen-tutorial.ts')
    },
    { timeout: 30_000 },
  )
})
