/** Shared types for the sail-owned eval harness. */

export type Lane = 'cheap' | 'frontier'
/** blank/kitchen-sink = sail app scaffolds; react-tw = control-arm baseline. */
export type ScaffoldKind = 'blank' | 'kitchen-sink' | 'react-tw'
/** recorded = CI fixture replay; omp = single real-run dispatch layer. */
export type DispatchMode = 'recorded' | 'omp'

export interface TaskGrading {
  compile?: boolean
  vocab_share?: boolean | { min?: number }
  gates?: boolean
  behavior?: boolean
  beauty?: boolean
}

export interface TaskDef {
  /** Task slug; must match directory name under tasks/. */
  id?: string
  prompt: string
  scaffold: ScaffoldKind
  lanes: Lane[]
  control_arms?: string[]
  grading: TaskGrading
}

export interface LoadedTask {
  slug: string
  dir: string
  def: TaskDef
  spec: BehaviorSpec | null
  goldenDir: string | null
}

/** Behavior steps query by visible text / role / label — never CSS selectors. */
export type Step =
  | { see: string }
  | { notSee: string }
  | { click: string }
  | { fill: { field: string; value: string } }
  | { choose: { option: string; in: string } }

export interface Scenario {
  name: string
  given?: { url?: string }
  steps: Step[]
}

export interface BehaviorSpec {
  view?: string
  scenarios: Scenario[]
}

export interface TrajectoryTurn {
  role: 'user' | 'assistant' | 'system' | 'tool'
  content: string
  /** Optional tool name when role is tool. */
  name?: string
  ts?: string
}

export interface Trajectory {
  task: string
  model: string
  dispatch: DispatchMode
  turns: TrajectoryTurn[]
  /** Wall-clock authoring turns (efficiency metric; turns-not-tokens). */
  turnCount: number
  startedAt: string
  finishedAt: string
  /** Relative paths written under the workdir. */
  filesWritten: string[]
  meta?: Record<string, unknown>
}

export interface Diagnostic {
  severity: string
  message: string
  rule?: string
  line?: number
  col?: number
  path?: string
}

export interface CompileGrade {
  ok: boolean
  errorCount: number
  warningCount: number
  diagnostics: Diagnostic[]
  /** Number of [sail:fix] auto-repairs the check path reported. */
  repairCount: number
  /** Agent turns until clean check (from trajectory); null if never green. */
  iterationsToGreen: number | null
  skipped?: boolean
  detail?: string
}

export interface VocabGrade {
  ok: boolean
  l1Count: number
  primitiveCount: number
  /** L1 / (L1 + primitives). Studio failure ~0.17 is the low-water mark. */
  share: number
  minShare: number
  skipped?: boolean
  detail?: string
}

export interface GatesGrade {
  ok: boolean
  lintErrors: number
  layoutFindings: number
  composeFindings: number
  repairCount: number
  skipped?: boolean
  detail?: string
}

export interface BehaviorGrade {
  ok: boolean
  passed: number
  failed: number
  results: Array<{
    name: string
    passed: boolean
    failedStep?: number
    error?: string
  }>
  skipped?: boolean
  detail?: string
}

export interface BeautyGrade {
  ok: boolean
  score: number | null
  maxScore: number
  source: 'operator' | 'judge' | 'skipped'
  notes?: string
  skipped?: boolean
  detail?: string
}

export interface GradeReport {
  compile?: CompileGrade
  vocab_share?: VocabGrade
  gates?: GatesGrade
  behavior?: BehaviorGrade
  beauty?: BeautyGrade
  /** Overall pass for the legs that ran. */
  ok: boolean
}

export interface RunResult {
  task: string
  lane: Lane
  dispatch: DispatchMode
  model: string
  workdir: string
  runDir: string
  trajectory: Trajectory
  grades: GradeReport
  startedAt: string
  finishedAt: string
  error?: string
}

export interface CorpusSummary {
  lane: Lane
  dispatch: DispatchMode
  startedAt: string
  finishedAt: string
  runs: RunResult[]
  passCount: number
  failCount: number
  skipCount: number
}
