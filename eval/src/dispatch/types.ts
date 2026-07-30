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
  /** Extra env for the agent process. */
  env?: Record<string, string>
  /** Abort after this many ms. */
  timeoutMs?: number
}

export interface DispatchAdapter {
  readonly mode: DispatchMode
  run(req: DispatchRequest): Promise<Trajectory>
}
