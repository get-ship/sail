/**
 * Lightweight structural validation for RunManifest / PhaseResult.
 * (HARNESS-2 asked for zod; we use hand-rolled checks to avoid new deps.
 * Shape is fully typed — this is the runtime door.)
 */

import {
  type PhaseResult,
  type RunManifest,
  checkContractVersion,
  isPhase,
} from './types.ts'

export function validateManifest(raw: unknown): { ok: true; value: RunManifest } | { ok: false; error: string } {
  if (!raw || typeof raw !== 'object') {
    return { ok: false, error: 'manifest must be a JSON object' }
  }
  const m = raw as Record<string, unknown>
  const verErr = checkContractVersion(m.contract_version)
  if (verErr) return { ok: false, error: verErr }

  for (const key of [
    'run_id',
    'batch_id',
    'task_id',
    'arm',
    'model_slug',
    'toolchain',
    'profile_hash',
    'harness_ver',
    'run_dir',
    'sandbox_dir',
    'standards_dir',
    'payload_dir',
  ] as const) {
    if (typeof m[key] !== 'string' || !(m[key] as string).length) {
      return { ok: false, error: `manifest.${key} must be a non-empty string` }
    }
  }

  if (m.phase !== undefined && typeof m.phase === 'string' && !isPhase(m.phase)) {
    return { ok: false, error: `manifest.phase unknown: ${m.phase}` }
  }

  if (!m.paths || typeof m.paths !== 'object') {
    return { ok: false, error: 'manifest.paths required' }
  }

  return { ok: true, value: m as unknown as RunManifest }
}

export function validatePhaseResult(raw: unknown): { ok: true; value: PhaseResult } | { ok: false; error: string } {
  if (!raw || typeof raw !== 'object') {
    return { ok: false, error: 'phase result must be a JSON object' }
  }
  const r = raw as Record<string, unknown>
  if (typeof r.phase !== 'string' || !isPhase(r.phase)) {
    return { ok: false, error: `phase result missing/unknown phase: ${String(r.phase)}` }
  }
  if (typeof r.ok !== 'boolean') {
    return { ok: false, error: 'phase result.ok must be boolean' }
  }
  return { ok: true, value: r as unknown as PhaseResult }
}
