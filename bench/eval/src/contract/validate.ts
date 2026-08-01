/**
 * Lightweight structural validation for RunManifest / PhaseResult.
 * (HARNESS-2 asked for zod; we use hand-rolled checks to avoid new deps.
 * Shape is fully typed — this is the runtime door.)
 */

import {
  type PhaseResult,
  type PhaseVerdict,
  type RunManifest,
  checkContractVersion,
  isPhase,
} from './types.ts'

const PATH_KEYS = ['events', 'manifest', 'gate', 'render', 'source'] as const
const VERDICTS = new Set<PhaseVerdict>(['pass', 'fail', 'skip', 'error'])

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

  // model may be string or null (resolved model optional with named reason later)
  if (m.model !== null && m.model !== undefined && typeof m.model !== 'string') {
    return { ok: false, error: 'manifest.model must be string or null' }
  }

  if (m.phase !== undefined && typeof m.phase === 'string' && !isPhase(m.phase)) {
    return { ok: false, error: `manifest.phase unknown: ${m.phase}` }
  }

  if (!m.paths || typeof m.paths !== 'object' || Array.isArray(m.paths)) {
    return { ok: false, error: 'manifest.paths required object' }
  }
  const paths = m.paths as Record<string, unknown>
  for (const key of PATH_KEYS) {
    if (typeof paths[key] !== 'string' || !(paths[key] as string).length) {
      return { ok: false, error: `manifest.paths.${key} must be a non-empty string` }
    }
  }

  if (!m.limits || typeof m.limits !== 'object' || Array.isArray(m.limits)) {
    return { ok: false, error: 'manifest.limits required object' }
  }
  const limits = m.limits as Record<string, unknown>
  if (typeof limits.phase_timeout_ms !== 'number' || !Number.isFinite(limits.phase_timeout_ms)) {
    return { ok: false, error: 'manifest.limits.phase_timeout_ms must be a finite number' }
  }

  if (m.env !== undefined && (typeof m.env !== 'object' || m.env === null || Array.isArray(m.env))) {
    return { ok: false, error: 'manifest.env must be an object when present' }
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
  if (typeof r.verdict !== 'string' || !VERDICTS.has(r.verdict as PhaseVerdict)) {
    return { ok: false, error: `phase result.verdict must be pass|fail|skip|error (got ${String(r.verdict)})` }
  }
  if (typeof r.wall_ms !== 'number' || !Number.isFinite(r.wall_ms)) {
    return { ok: false, error: 'phase result.wall_ms must be a finite number' }
  }
  if (!Array.isArray(r.artifacts)) {
    return { ok: false, error: 'phase result.artifacts must be an array' }
  }
  if (!Array.isArray(r.flags)) {
    return { ok: false, error: 'phase result.flags must be an array' }
  }
  if (!r.missing || typeof r.missing !== 'object' || Array.isArray(r.missing)) {
    return { ok: false, error: 'phase result.missing must be an object' }
  }
  if (r.metrics !== undefined && (typeof r.metrics !== 'object' || r.metrics === null || Array.isArray(r.metrics))) {
    return { ok: false, error: 'phase result.metrics must be an object when present' }
  }
  if (r.detail !== undefined && (typeof r.detail !== 'object' || r.detail === null || Array.isArray(r.detail))) {
    return { ok: false, error: 'phase result.detail must be an object when present' }
  }
  return { ok: true, value: r as unknown as PhaseResult }
}
