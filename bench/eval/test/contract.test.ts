import { describe, expect, it } from 'vitest'
import {
  CONTRACT_VERSION,
  checkContractVersion,
  emptyPhaseResult,
  isPhase,
} from '../src/contract/types.ts'
import { validateManifest, validatePhaseResult } from '../src/contract/validate.ts'
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
    limits: { phase_timeout_ms: 60_000 },
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

  it('validateManifest rejects missing fields and empty paths', () => {
    const r = validateManifest({ contract_version: CONTRACT_VERSION })
    expect(r.ok).toBe(false)

    const emptyPaths = validateManifest({
      ...baseManifest(),
      paths: {},
    })
    expect(emptyPaths.ok).toBe(false)
    if (!emptyPaths.ok) expect(emptyPaths.error).toMatch(/paths\./)
  })

  it('validateManifest accepts full shape with model null', () => {
    const r = validateManifest(baseManifest())
    expect(r.ok).toBe(true)
  })

  it('validatePhaseResult requires verdict/artifacts/flags/missing', () => {
    expect(validatePhaseResult({ phase: 'gate', ok: true }).ok).toBe(false)
    expect(
      validatePhaseResult({
        phase: 'gate',
        ok: true,
        verdict: 'pass',
        wall_ms: 1,
        artifacts: [],
        flags: [],
        missing: {},
      }).ok,
    ).toBe(true)
  })

  it('in-process prepare/gate/collect roundtrip', async () => {
    const m = baseManifest()
    const prep = await invokePhase('prepare', m, { inProcess: true, handlers })
    expect(prep.contractError).toBe(false)
    expect(prep.result.phase).toBe('prepare')
    expect(prep.result.ok).toBe(true)

    const gate = await invokePhase(
      'gate',
      { ...m, phase: 'gate', arm: 'vanilla' },
      { inProcess: true, handlers },
    )
    expect(gate.result.phase).toBe('gate')
    expect(gate.result.ok).toBe(true)

    const collect = await invokePhase(
      'collect',
      { ...m, phase: 'collect' },
      { inProcess: true, handlers },
    )
    expect(collect.result.ok).toBe(true)
    expect(collect.result.flags).toContain('missing_usage')
    expect(collect.result.missing.turns).toBe('not_collected_v1')
  })

  it('subprocess prepare invokes eval-adapter (real stdin/stdout contract)', async () => {
    const m = baseManifest()
    const prep = await invokePhase('prepare', m, {
      inProcess: false,
      timeoutMs: 60_000,
    })
    expect(prep.timedOut).toBe(false)
    expect(prep.exitCode).toBe(0)
    expect(prep.contractError).toBe(false)
    expect(prep.result.phase).toBe('prepare')
    expect(prep.result.ok).toBe(true)
    expect(Array.isArray(prep.result.artifacts)).toBe(true)
    expect(typeof prep.result.verdict).toBe('string')
  }, 90_000)

  it('emptyPhaseResult defaults', () => {
    const r = emptyPhaseResult('render')
    expect(r.ok).toBe(false)
    expect(r.artifacts).toEqual([])
  })
})
