/**
 * Execute failure must not report status=complete even if render would pass.
 */

import { describe, expect, it } from 'vitest'
import { mkdtempSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { LocalRunner } from '../src/runner/local.ts'
import { repoRoot, standardsDir } from '../src/naming/paths.ts'
import type { ExecuteOutcome } from '../src/runner/types.ts'

describe('execute failure status', () => {
  it('injected execute failure forces status=failed (env-independent)', async () => {
    const lab = mkdtempSync(join(tmpdir(), 'sail-bench-exec-fail-'))
    const runner = new LocalRunner({
      lab,
      standards: standardsDir(repoRoot()),
      repo: repoRoot(),
      subprocess: false,
      // Stub execute — no PATH tricks, works even if /usr/bin/grok exists.
      executeOverride: async (): Promise<ExecuteOutcome> => ({
        ok: false,
        wall_ms: 1,
        session_id: null,
        flags: ['execute_skipped', 'grok_not_on_path'],
        seeded: false,
        message: 'stub: execute forced fail',
      }),
    })

    const outcome = await runner.runOne({
      taskId: 'parallax-landing',
      arm: 'sail',
      modelSlug: 'grok',
      model: 'grok',
      toolchain: 'current',
      profileId: 'p0',
      seed: false,
    })

    // Even if a browser render of the scaffold succeeds, execute failure
    // must force status=failed.
    expect(outcome.status).toBe('failed')
    expect(outcome.flags).toContain('execute_failed')
    expect(outcome.flags).toContain('execute_skipped')
  }, 120_000)
})
