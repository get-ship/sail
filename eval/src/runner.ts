import { existsSync, mkdirSync, writeFileSync } from 'node:fs'
import { join } from 'node:path'
import { createDispatch, type DispatchAdapter } from './dispatch/index.ts'
import {
  loadBehaviorHtmlFromWorkdir,
  runGradeChain,
} from './graders/chain.ts'
import { realHost, type HostTooling } from './host.ts'
import {
  assertSafeSlug,
  fixturesDir,
  runsDir,
} from './paths.ts'
import { writeResults } from './report.ts'
import { sanitizeJson, sanitizeText } from './sanitize.ts'
import { injectSkillFile, scaffoldWorkdir } from './scaffold.ts'
import { loadCorpus, loadTask, listTasks } from './task.ts'
import { classifyRun } from './classify.ts'
import type {
  CorpusSummary,
  DispatchMode,
  GradeReport,
  Lane,
  LoadedTask,
  RunResult,
  Trajectory,
} from './types.ts'

export { classifyRun }

export interface RunOptions {
  lane?: Lane
  dispatch?: DispatchMode
  /**
   * Model fuzzy id — pass-through to omp only. Never defaulted to a
   * concrete provider model in this package. Recorded lane uses the
   * synthetic label `recorded` for metadata only.
   */
  model?: string
  /** Optional API key pass-through to omp (--api-key). Prefer env/profile. */
  apiKey?: string
  /** Task slugs; default all tasks that list the lane. */
  tasks?: string[]
  host?: HostTooling
  adapter?: DispatchAdapter
  /**
   * Keep workdir under runs/<date>/<task>/workdir after the run.
   * Workdirs are always inside the repo (eval/runs/…) so sail ops accept paths.
   */
  keepWorkdir?: boolean
  /** Write RESULTS.md (default true). */
  writeReport?: boolean
  /** Base URL for frontier behavior (deck serve). */
  behaviorBaseUrl?: string
  beautyScores?: Record<string, number>
  /**
   * When true, missing sail/host soft-skips legs. Default **false** (strict):
   * missing tooling fails the run so CI cannot greenwash.
   */
  allowSkip?: boolean
  /** Opt-in cheap-lane source-text behavior heuristics (not deck/playwright). */
  enableCheapBehavior?: boolean
  /** Soft-skip frontier beauty when no score is provided. */
  allowSkipBeauty?: boolean
  /**
   * Override scaffold kind (e.g. `react-tw` control arm). Default: task's
   * scaffold field. Control arms are the same omp path, different scaffold.
   */
  scaffold?: import('./types.ts').ScaffoldKind
  /** Root for tasks/ (tests). */
  tasksRoot?: string
  runsRoot?: string
}

export async function runCorpus(opts: RunOptions = {}): Promise<CorpusSummary> {
  const lane: Lane = opts.lane ?? 'cheap'
  const dispatchMode: DispatchMode = opts.dispatch ?? 'recorded'
  // Recorded lane labels metadata as "recorded". omp: only what the operator
  // passed — empty string means "omp profile resolves; do not pass --model".
  const model =
    opts.model ?? (dispatchMode === 'recorded' ? 'recorded' : '')
  const host = opts.host ?? realHost()
  const adapter = opts.adapter ?? createDispatch(dispatchMode)
  const allowSkip = opts.allowSkip ?? false
  const startedAt = new Date().toISOString()
  const date = startedAt.slice(0, 10)
  const runsRoot = opts.runsRoot ?? runsDir()

  let tasks: LoadedTask[]
  if (opts.tasks?.length) {
    tasks = opts.tasks.map((s) => loadTask(assertSafeSlug(s), opts.tasksRoot))
  } else {
    tasks = loadCorpus(opts.tasksRoot).filter((t) => t.def.lanes.includes(lane))
  }

  const runs: RunResult[] = []
  for (const task of tasks) {
    const result = await runOneTask({
      task,
      lane,
      dispatchMode,
      model,
      apiKey: opts.apiKey,
      host,
      adapter,
      date,
      runsRoot,
      keepWorkdir: opts.keepWorkdir,
      behaviorBaseUrl: opts.behaviorBaseUrl,
      beautyScore: opts.beautyScores?.[task.slug],
      allowSkip,
      enableCheapBehavior: opts.enableCheapBehavior ?? false,
      allowSkipBeauty: opts.allowSkipBeauty ?? false,
      scaffoldOverride: opts.scaffold,
    })
    runs.push(result)
  }

  const finishedAt = new Date().toISOString()
  let passCount = 0
  let failCount = 0
  let skipCount = 0
  for (const r of runs) {
    const kind = classifyRun(r, lane)
    if (kind === 'pass') passCount++
    else if (kind === 'skip') skipCount++
    else failCount++
  }

  const summary: CorpusSummary = {
    lane,
    dispatch: dispatchMode,
    startedAt,
    finishedAt,
    runs,
    passCount,
    failCount,
    skipCount,
  }

  if (opts.writeReport !== false) {
    writeResults(summary)
  }
  return summary
}

async function runOneTask(args: {
  task: LoadedTask
  lane: Lane
  dispatchMode: DispatchMode
  model: string
  apiKey?: string
  host: HostTooling
  adapter: DispatchAdapter
  date: string
  runsRoot: string
  keepWorkdir?: boolean
  behaviorBaseUrl?: string
  beautyScore?: number
  allowSkip: boolean
  enableCheapBehavior: boolean
  allowSkipBeauty: boolean
  scaffoldOverride?: import('./types.ts').ScaffoldKind
}): Promise<RunResult> {
  const { task, lane, dispatchMode, model, host, adapter } = args
  const startedAt = new Date().toISOString()
  const runDir = join(args.runsRoot, args.date, task.slug)
  mkdirSync(runDir, { recursive: true })

  // Always under eval/runs/… so sail ops see workspace-relative paths.
  const workdir = join(runDir, 'workdir')
  mkdirSync(workdir, { recursive: true })

  try {
    const scaffoldKind = args.scaffoldOverride ?? task.def.scaffold
    scaffoldWorkdir(workdir, scaffoldKind)
    // Skill/author-surface injection is sail-only — control arms must not leak.
    const sailScaffold =
      scaffoldKind === 'blank' || scaffoldKind === 'kitchen-sink'
    if (sailScaffold) {
      injectSkillFile(workdir)
    }

    const fixtureDir = resolveFixtureDir(task, dispatchMode)

    const trajectory = await adapter.run({
      prompt: task.def.prompt,
      workdir,
      model: model || undefined,
      apiKey: args.apiKey,
      task: task.slug,
      fixtureDir,
      scaffold: scaffoldKind,
      sailArtifacts: sailScaffold,
    })

    // Static HTML only when cheap-behavior opt-in or frontier without baseUrl.
    const needHtml =
      (lane === 'cheap' && args.enableCheapBehavior) ||
      (lane === 'frontier' && !args.behaviorBaseUrl)
    const behaviorHtml = needHtml ? loadBehaviorHtmlFromWorkdir(workdir) : undefined

    const grades = await runGradeChain({
      host,
      workdir,
      lane,
      grading: task.def.grading,
      trajectory,
      spec: task.spec,
      behaviorHtml,
      behaviorBaseUrl: args.behaviorBaseUrl,
      beautyScore: args.beautyScore,
      allowSkip: args.allowSkip,
      enableCheapBehavior: args.enableCheapBehavior,
      allowSkipBeauty: args.allowSkipBeauty,
    })

    const finishedAt = new Date().toISOString()
    const result: RunResult = {
      task: task.slug,
      lane,
      dispatch: dispatchMode,
      // Prefer trajectory-resolved model (reproducibility); never invent one.
      model: trajectory.model || model,
      workdir: args.keepWorkdir ? workdir : '<workdir>',
      runDir: args.keepWorkdir ? runDir : '<runDir>',
      trajectory: sanitizeTrajectory(trajectory, workdir),
      grades,
      startedAt,
      finishedAt,
    }

    writeRunArtifacts(runDir, result, trajectory, workdir)
    return result
  } catch (e) {
    const finishedAt = new Date().toISOString()
    const msg = e instanceof Error ? e.message : String(e)
    const emptyTraj: Trajectory = {
      task: task.slug,
      model,
      dispatch: dispatchMode,
      turns: [],
      turnCount: 0,
      startedAt,
      finishedAt,
      filesWritten: [],
    }
    const result: RunResult = {
      task: task.slug,
      lane,
      dispatch: dispatchMode,
      model,
      workdir: '<workdir>',
      runDir: '<runDir>',
      trajectory: emptyTraj,
      grades: { ok: false } satisfies GradeReport,
      startedAt,
      finishedAt,
      error: sanitizeText(msg, workdir),
    }
    try {
      writeRunArtifacts(runDir, result, emptyTraj, workdir)
    } catch {
      /* ignore */
    }
    return result
  }
}

/** Golden first; optional fixtures/recorded/<slug> fallback for recorded dispatch. */
function resolveFixtureDir(task: LoadedTask, dispatchMode: DispatchMode): string | null {
  if (task.goldenDir) return task.goldenDir
  if (dispatchMode === 'recorded') {
    const fallback = join(fixturesDir(), 'recorded', task.slug)
    if (existsSync(fallback)) return fallback
  }
  return null
}

function sanitizeTrajectory(t: Trajectory, workdir: string): Trajectory {
  return sanitizeJson(t, workdir)
}

function writeRunArtifacts(
  runDir: string,
  result: RunResult,
  trajectory: Trajectory,
  workdir: string,
): void {
  mkdirSync(runDir, { recursive: true })
  const safeResult = sanitizeJson(result, workdir)
  writeFileSync(join(runDir, 'grades.json'), JSON.stringify(safeResult.grades, null, 2))
  writeFileSync(join(runDir, 'result.json'), JSON.stringify(safeResult, null, 2))

  const lines = trajectory.turns.map((turn) =>
    JSON.stringify(sanitizeJson(turn, workdir)),
  )
  writeFileSync(join(runDir, 'trajectory.jsonl'), lines.join('\n') + (lines.length ? '\n' : ''))
}

export { listTasks, loadTask, loadCorpus }
