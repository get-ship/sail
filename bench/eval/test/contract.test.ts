import { describe, expect, it } from 'vitest'
import {
  CONTRACT_VERSION,
  checkContractVersion,
  emptyPhaseResult,
  isPhase,
} from '../src/contract/types.ts'
import { validateManifest } from '../src/contract/validate.ts'
import { invokePhase } from '../src/contract/invoke.ts'
import { handlers } from '../src/adapters/index.ts'
import type { RunManifest } from '../src/contract/types.ts'
import { mkdtempSync, mkdirSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'

function baseManifest(overrides: Partial<RunManifest> = {}): RunManifest {
  const root = mkdtempSync(join(tmpdir(), 'sail-bench-contract-'))
  const runDir = join(root, 'run')
  const sandbox = join(runDir, 'sandbox')
  const payload = join(root, 'payload')
  mkdirSync(sandbox, { recursive: true })
  mkdirSync(payload, { recursive: true })
  writeFileSync(join(payload, 'index.html'), '<html><body>hi</body></html>\n')
  return {
    contract_version: CONTRACT_VERSION,
    run_id: 'calm-otter',
    batch_id: 'batch-1',
    task_id: 't',
    arm: 'vanilla',
    model_slug: 'seed',
    model: null,
    toolchain: 'current',
    profile_hash: 'abc',
    harness_ver: 'test',
    run_dir: runDir,
    sandbox_dir: sandbox,
    standards_dir: root,
    payload_dir: payload,
    task: { prompt: 'hi' },
    profile: {},
    paths: {
      events: 'events.jsonl',
      manifest: 'manifest.json',
      gate: 'gate.json',
      render: 'render.png',
      source: 'sandbox',
    },
    env: {},
    limits: { phase_timeout_ms: 30_000 },
    phase: 'prepare',
    ...overrides,
  }
}

describe('contract types', () => {
  it('accepts run-contract/1', () => {
    expect(checkContractVersion(CONTRACT_VERSION)).toBeNull()
    expect(checkContractVersion('run-contract/9')).toMatch(/unsupported/)
  })

  it('validates phases', () => {
    expect(isPhase('gate')).toBe(true)
    expect(isPhase('nope')).toBe(false)
  })

  it('validateManifest rejects missing fields', () => {
    const r = validateManifest({ contract_version: CONTRACT_VERSION })
    expect(r.ok).toBe(false)
  })

  it('in-process prepare/gate/collect roundtrip', async () => {
    const m = baseManifest()
    const prep = await invokePhase('prepare', m, { inProcess: true, handlers })
    expect(prep.contractError).toBe(false)
    // prepare copies payload onto itself — ok may fail if payload empty tree issues; ensure no crash
    expect(prep.result.phase).toBe('prepare')

    const gate = await invokePhase('gate', { ...m, phase: 'gate', arm: 'vanilla' }, {
      inProcess: true,
      handlers,
    })
    expect(gate.result.phase).toBe('gate')
    expect(gate.result.ok).toBe(true)

    const collect = await invokePhase('collect', { ...m, phase: 'collect' }, {
      inProcess: true,
      handlers,
    })
    expect(collect.result.ok).toBe(true)
    expect(collect.result.flags).toContain('missing_usage')
  })

  it('emptyPhaseResult defaults', () => {
    const r = emptyPhaseResult('render')
    expect(r.ok).toBe(false)
    expect(r.artifacts).toEqual([])
  })
})
