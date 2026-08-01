import { describe, expect, it } from 'vitest'
import { mkdtempSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { RunStore } from '../src/store/db.ts'
import { writeGrades } from '../src/store/grades.ts'

describe('RunStore', () => {
  it('scopes runs per batch and enforces profile immutability', () => {
    const dir = mkdtempSync(join(tmpdir(), 'sail-bench-store-'))
    const store = RunStore.open(join(dir, 'lab-bench.sqlite'))

    store.upsertBatch({
      batch_id: 'b1',
      ts: '2026-08-01T00:00:00Z',
      name: 'n',
      question: null,
      profile_hash: 'aaa',
      harness_ver: 't',
      status: 'running',
      n: 1,
      tasks_json: null,
      arms_json: null,
      constants_json: null,
      report_dir: null,
      manifest_json: null,
    })

    store.upsertRun({
      run_id: 'calm-otter',
      batch_id: 'b1',
      ts: '2026-08-01T00:00:00Z',
      task_id: 'parallax-landing',
      arm: 'sail',
      model: null,
      model_slug: 'seed',
      toolchain: 'current',
      profile_hash: 'aaa',
      harness_ver: 't',
      run_dir: '/tmp/x',
      status: 'complete',
      wall_ms: 10,
      gate_ok: 1,
      compile_ok: 1,
      render_ok: 1,
      tokens_in: null,
      tokens_out: null,
      turns: null,
      session_id: null,
      render_method: 'browser',
      flags_json: '["missing_usage"]',
      metrics_json: null,
      gate_detail_json: null,
      artifacts_json: null,
      failure_classes_json: null,
      missing_reasons_json: '{"tokens_in":"usage_unavailable"}',
    })

    expect(store.runsForBatch('b1')).toHaveLength(1)

    expect(() =>
      store.upsertBatch({
        batch_id: 'b1',
        ts: 't',
        name: 'n',
        question: null,
        profile_hash: 'bbb',
        harness_ver: 't',
        status: 'running',
        n: 1,
        tasks_json: null,
        arms_json: null,
        constants_json: null,
        report_dir: null,
        manifest_json: null,
      }),
    ).toThrow(/profile immutable/)

    const g = writeGrades(store, {
      runRef: 'calm-otter',
      pairs: [{ key: 'beauty', value: '3' }],
      note: 'ok',
    })
    expect(g.grades[0]?.key).toBe('beauty')
    expect(store.gradesForRun('calm-otter')).toHaveLength(1)

    store.close()
  })
})
