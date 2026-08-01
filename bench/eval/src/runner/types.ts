/**
 * Runner port — LocalRunner now; FleetRunner later speaks FLEET-2 job API.
 * Nothing outside runner knows which is active.
 */

import type { PhaseResult, RunManifest } from '../contract/types.ts'
import type { RunConfig } from '../config/types.ts'

export interface Runner {
  prepare(run: RunHandle): Promise<PhaseResult>
  execute(run: RunHandle): Promise<ExecuteOutcome>
  /** gate → render → collect (adapter phases after execute). */
  collect(run: RunHandle): Promise<CollectBundle>
}

export interface RunHandle {
  manifest: RunManifest
  config: RunConfig
  batchId: string
  runId: string
  runDir: string
}

export interface ExecuteOutcome {
  ok: boolean
  wall_ms: number
  session_id: string | null
  flags: string[]
  message?: string
  /** When seed/no-agent: true. */
  seeded: boolean
}

export interface CollectBundle {
  gate: PhaseResult
  render: PhaseResult
  collect: PhaseResult
  /**
   * Set when a phase hit CONTRACT abort conditions (contract error, crash, timeout).
   * Remaining phases after the abort are skipped with synthetic error results.
   */
  aborted?: {
    phase: 'gate' | 'render' | 'collect'
    reason: string
    flags: string[]
  }
}

export interface RunOutcome {
  batchId: string
  runId: string
  runDir: string
  status: 'complete' | 'failed'
  wall_ms: number
  flags: string[]
  gate_ok: number | null
  render_ok: number | null
  manifestPath: string
}
