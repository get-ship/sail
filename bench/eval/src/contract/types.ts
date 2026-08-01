/**
 * RUN CONTRACT v1 — typed seam between core and evaluation adapters.
 *
 * Core invokes:  eval-adapter <phase>  with RunManifest on stdin
 * Adapter replies: PhaseResult on stdout
 *
 * See CONTRACT.md for normative semantics.
 */

/** Contract version string. Bump only with a coordinated core+adapter change. */
export const CONTRACT_VERSION = 'run-contract/1' as const
export type ContractVersion = typeof CONTRACT_VERSION

export type Phase = 'prepare' | 'gate' | 'render' | 'collect'

export type PhaseVerdict = 'pass' | 'fail' | 'skip' | 'error'

export type ArtifactStatus = 'present' | 'missing' | 'degraded'

/** Declared artifact under the run dir. Blobs on disk; this is the index row. */
export interface ArtifactDecl {
  name: string
  /** Path relative to run_dir. */
  rel_path: string
  blocking: boolean
  status: ArtifactStatus
  sha256?: string | null
  bytes?: number | null
  /** Named reason when status is missing or degraded. Never silent. */
  reason?: string | null
}

export interface ContractError {
  code: string
  message: string
}

export interface RunPaths {
  events: string
  manifest: string
  gate: string
  render: string
  source: string
}

export interface PhaseLimits {
  phase_timeout_ms: number
}

/**
 * Run manifest — core → adapter (stdin).
 * Adapters must not assume anything about store layout beyond these paths.
 */
export interface RunManifest {
  contract_version: ContractVersion | string
  run_id: string
  batch_id: string
  task_id: string
  arm: string
  model_slug: string
  model: string | null
  toolchain: string
  profile_hash: string
  harness_ver: string
  /** Absolute path; all artifacts live here. */
  run_dir: string
  /** Agent workspace (absolute). Often run_dir/sandbox. */
  sandbox_dir: string
  /** Absolute path to bench/ standards root. */
  standards_dir: string
  /** Absolute path to arm payload base tree. */
  payload_dir: string
  task: Record<string, unknown>
  profile: Record<string, unknown>
  paths: RunPaths
  env: Record<string, string>
  limits: PhaseLimits
  phase: Phase
  /** Extra phase-specific inputs (e.g. seed flag for prepare). */
  options?: Record<string, unknown>
}

/** Phase result — adapter → core (stdout). */
export interface PhaseResult {
  contract_version: ContractVersion | string
  phase: Phase
  ok: boolean
  verdict: PhaseVerdict
  wall_ms: number
  artifacts: ArtifactDecl[]
  metrics: Record<string, unknown>
  flags: string[]
  /** Field → named reason for explicit NULL semantics. */
  missing: Record<string, string>
  detail: Record<string, unknown>
  error: ContractError | null
}

export function emptyPhaseResult(phase: Phase, wall_ms = 0): PhaseResult {
  return {
    contract_version: CONTRACT_VERSION,
    phase,
    ok: false,
    verdict: 'error',
    wall_ms,
    artifacts: [],
    metrics: {},
    flags: [],
    missing: {},
    detail: {},
    error: null,
  }
}

export function isPhase(s: string): s is Phase {
  return s === 'prepare' || s === 'gate' || s === 'render' || s === 'collect'
}

/** Validate contract_version; returns error message or null if ok. */
export function checkContractVersion(v: unknown): string | null {
  if (typeof v !== 'string' || !v.startsWith('run-contract/')) {
    return `unsupported or missing contract_version: ${JSON.stringify(v)}`
  }
  // v1 accepts only exact v1 for now
  if (v !== CONTRACT_VERSION) {
    return `unsupported contract_version '${v}' (this adapter speaks ${CONTRACT_VERSION})`
  }
  return null
}
