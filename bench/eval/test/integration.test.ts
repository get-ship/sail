/**
 * Phase-1 acceptance path (seed, no live agent).
 * Uses real bench/ standards + temp lab root.
 */

import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync, statSync } from 'node:fs'
import { mkdtempSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { LocalRunner } from '../src/runner/local.ts'
import { writeBatchReport } from '../src/report/batch.ts'
import { RunStore } from '../src/store/db.ts'
import { isWordPairId } from '../src/naming/word-pair.ts'
import { repoRoot, standardsDir, storePath } from '../src/naming/paths.ts'

describe('sail-bench integration (seed)', () => {
  it('run + report for parallax-landing / sail / current', async () => {
    const lab = mkdtempSync(join(tmpdir(), 'sail-bench-lab-'))
    const runner = new LocalRunner({
      lab,
      standards: standardsDir(repoRoot()),
      repo: repoRoot(),
      subprocess: false,
    })

    const outcome = await runner.runOne({
      taskId: 'parallax-landing',
      arm: 'sail',
      modelSlug: 'seed',
      model: 'seed',
      toolchain: 'current',
      profileId: 'p0',
      seed: true,
    })

    expect(isWordPairId(outcome.runId)).toBe(true)
    expect(existsSync(outcome.runDir)).toBe(true)
    expect(existsSync(join(outcome.runDir, 'manifest.json'))).toBe(true)
    expect(existsSync(join(outcome.runDir, 'events.jsonl'))).toBe(true)
    expect(existsSync(join(outcome.runDir, 'gate.json'))).toBe(true)
    expect(existsSync(join(outcome.runDir, 'sandbox', 'routes', 'page.sail'))).toBe(true)

    // render.png may be present (chrome available) or missing with marker
    const renderOk = existsSync(join(outcome.runDir, 'render.png'))
    if (!renderOk) {
      expect(existsSync(join(outcome.runDir, 'render.MISSING.json'))).toBe(true)
      expect(outcome.status).toBe('failed')
      expect(outcome.flags.some((f) => f.includes('render') || f.includes('blocking'))).toBe(true)
    } else {
      expect(statSync(join(outcome.runDir, 'render.png')).size).toBeGreaterThan(0)
      expect(outcome.render_ok).toBe(1)
      expect(outcome.status).toBe('complete')
    }

    // db row
    const store = RunStore.openExisting(storePath(lab))
    const row = store.getRun(outcome.runId)
    expect(row).not.toBeNull()
    expect(row!.task_id).toBe('parallax-landing')
    expect(row!.arm).toBe('sail')
    expect(row!.run_dir).toBe(outcome.runDir)
    // events are files, not db blobs
    expect(row!.missing_reasons_json).toBeTruthy()
    store.close()

    // report: zero copied bytes
    const report = writeBatchReport(lab, outcome.batchId)
    expect(report.copiedBytes).toBe(0)
    expect(existsSync(report.indexPath)).toBe(true)
    expect(existsSync(join(report.reportDir, 'SUMMARY.md'))).toBe(true)
    const index = readFileSync(report.indexPath, 'utf8')
    expect(index).toContain(outcome.runId)
    expect(index).toContain('zero copied bytes')
    // must reference, not embed base64 images
    expect(index).not.toMatch(/data:image/)
  }, 120_000)
})
