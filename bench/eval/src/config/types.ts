/**
 * Config types for Task, Profile, Toolchain, Batch, Run.
 * Validated at load time (see load.ts).
 */

export interface TaskDef {
  id: string
  prompt: string
  systems?: string[]
  scaffold?: string
  grading?: Record<string, unknown>
  falsification?: string
}

export interface ProfileDef {
  id: string
  skill_source?: string
  agents_md?: string
  policy?: string
  mcp?: boolean
  description?: string
  [key: string]: unknown
}

export interface ToolchainInfo {
  /** Alias or stamp used in FS path (e.g. "current" or git stamp). */
  id: string
  /** Concrete stamp written into run rows. */
  stamp: string
  sailBin: string | null
  createdAt: string
}

export type ArmId = 'sail' | 'react' | 'vanilla' | string

export interface ArmConfig {
  id: ArmId
  /** Subdir under lab-kit/payload or standards payload. */
  payload: string
  /** Gate kind. */
  gate: 'sail_check' | 'vite_build' | 'html_structure' | 'none'
  render: 'browser' | 'takumi' | 'none'
}

export const ARMS: Record<string, ArmConfig> = {
  sail: { id: 'sail', payload: 'sail', gate: 'sail_check', render: 'browser' },
  react: { id: 'react', payload: 'react-base', gate: 'vite_build', render: 'browser' },
  'react-tw': { id: 'react-tw', payload: 'react-base', gate: 'vite_build', render: 'browser' },
  vanilla: { id: 'vanilla', payload: 'vanilla', gate: 'html_structure', render: 'browser' },
}

export interface BatchConfig {
  name: string
  question?: string
  tasks: string[]
  arms: string[]
  n: number
  profileId: string
  modelSlug: string
  model: string | null
  toolchain: string
}

export interface RunConfig {
  taskId: string
  arm: string
  modelSlug: string
  model: string | null
  toolchain: string
  profileId: string
  /** Skip live agent; keep scaffold / optional seed source. */
  seed: boolean
  batchName?: string
  n?: number
}

export const HARNESS_VER = 'sail-bench-0.1.0'
