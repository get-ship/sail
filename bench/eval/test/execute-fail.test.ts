/**
 * Execute failure must not report status=complete even if render would pass.
 */

import { describe, expect, it } from 'vitest'
import { mkdtempSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { LocalRunner } from '../src/runner/local.ts'
import { repoRoot, standardsDir } from '../src/naming/paths.ts'

describe('execute failure status', () => {
  it('live path without grok (PATH cleared) marks run failed', async () => {
    const lab = mkdtempSync(join(tmpdir(), 'sail-bench-exec-fail-'))
    const runner = new LocalRunner({
      lab,
      standards: standardsDir(repoRoot()),
      repo: repoRoot(),
      subprocess: false,
    })

    const prevPath = process.env.PATH
    // Hide grok; keep bare minimum so nothing else breaks unexpectedly
    process.env.PATH = '/usr/bin:/bin'

    try {
      const outcome = await runner.runOne({
        taskId: 'parallax-landing',
        arm: 'sail',
        modelSlug: 'grok',
        model: 'grok',
        toolchain: 'current',
        profileId: 'p0',
        seed: false, // force live execute path
      })
      // Even if a browser render of the scaffold would succeed, execute failure
      // must force status=failed (issue #1).
      expect(outcome.status).toBe('failed')
      expect(outcome.flags).toContain('execute_failed')
      expect(
        outcome.flags.some(
          (f) =>
            f.includes('grok') ||
            f === 'execute_skipped' ||
            f === 'dispatch_nonzero_exit',
        ),
      ).toBe(true)
    } finally {
      process.env.PATH = prevPath
    }
  }, 120_000)
})
