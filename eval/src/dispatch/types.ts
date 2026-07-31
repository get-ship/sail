import type { DispatchMode, Trajectory } from '../types.ts'

export interface DispatchRequest {
  prompt: string
  workdir: string
  /**
   * Operator-supplied model fuzzy id (pass-through to omp --model).
   * Empty/undefined = do not pass --model; omp profile resolves it.
   * Never defaulted by this package to a concrete provider model.
   */
  model?: string
  /** Optional API key pass-through (omp --api-key). Prefer env/omp profile. */
  apiKey?: string
  /** Task slug for fixture lookup. */
  task: string
  /** Absolute path to golden/fixture source when recorded. */
  fixtureDir?: string | null
  /**
   * Scaffold/context profile for prompt framing (EVAL-1).
   * Control arms must not receive sail-specific framing.
   * Default (omit): sail blank framing (backward-compatible).
   */
  scaffold?: import('../types.ts').ScaffoldKind
  /** Explicit sail-artifact flag; when false, never frame as sail authoring. */
  sailArtifacts?: boolean
  /** Extra env for the agent process. */
  env?: Record<string, string>
  /** Abort after this many ms. */
  timeoutMs?: number
}

export interface DispatchAdapter {
  readonly mode: DispatchMode
  run(req: DispatchRequest): Promise<Trajectory>
}
