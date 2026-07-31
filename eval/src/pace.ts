/**
 * Paced multi-arm runner (EVAL-1).
 *
 * Serial: one (task, arm) at a time; sleeps `interval` between runs.
 * Resume-safe: skips completed (task, arm, model) triples already under runs/.
 * Artifacts path: runs/<date>/<task>/<arm>/<modelSlug>/ (triple identity on disk).
 * omp only — model is operator-supplied fuzzy string (never hardcoded).
 */

import { createHash } from 'node:crypto'
import {
  existsSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  writeFileSync,
  statSync,
  rmSync,
} from 'node:fs'
import { dirname, join, delimiter } from 'node:path'
import {
  gradingForArm,
  loadArms,
  materializeArm,
  type LoadedArm,
} from './arm.ts'
import { createDispatch, type DispatchAdapter } from './dispatch/index.ts'
import { runGradeChain, loadBehaviorHtmlFromWorkdir } from './graders/chain.ts'
import { realHost, type HostTooling } from './host.ts'
import { assertSafeSlug, findSailBin, runsDir } from './paths.ts'
import { writeResults } from './report.ts'
import { sanitizeJson, sanitizeText } from './sanitize.ts'
import { loadCorpus, loadTask } from './task.ts'
import type {
  CorpusSummary,
  GradeReport,
  Lane,
  LoadedTask,
  RunResult,
  Trajectory,
} from './types.ts'
import { classifyRun } from './classify.ts'

export interface PaceOptions {
  /** Required fuzzy model string for omp — never defaulted. */
  model: string
  /** Sleep between runs (ms). Default 10 minutes. */
  intervalMs?: number
  /** Arm ids; default all arms under eval/arms/. */
  arms?: string[]
  /** Task slugs; default all tasks that list frontier (or all). */
  tasks?: string[]
  /** Lane for grading policy (default frontier for paced overnight). */
  lane?: Lane
  apiKey?: string
  host?: HostTooling
  adapter?: DispatchAdapter
  keepWorkdir?: boolean
  writeReport?: boolean
  behaviorBaseUrl?: string
  beautyScores?: Record<string, number>
  allowSkip?: boolean
  allowSkipBeauty?: boolean
  /** When true, do not sleep (tests / smoke). */
  noSleep?: boolean
  runsRoot?: string
  tasksRoot?: string
  armsRoot?: string
  /**
   * Dry resume probe: list what would run without dispatching.
   * Still writes RESULTS if writeReport.
   */
  dryRun?: boolean
}

export interface PacePlanEntry {
  task: string
  arm: string
  model: string
  status: 'pending' | 'completed' | 'skipped-resume'
  runDir: string
}

/** Parse interval strings: `10m`, `30s`, `1h`, or raw milliseconds number string. */
export function parseInterval(raw: string): number {
  const s = raw.trim().toLowerCase()
  const m = /^(\d+(?:\.\d+)?)(ms|s|m|h)?$/.exec(s)
  if (!m) {
    throw new Error(
      `invalid --interval ${raw}: use e.g. 10m, 30s, 1h, or milliseconds`,
    )
  }
  const n = Number(m[1])
  const unit = m[2] ?? 'ms'
  switch (unit) {
    case 'ms':
      return Math.max(0, Math.floor(n))
    case 's':
      return Math.max(0, Math.floor(n * 1000))
    case 'm':
      return Math.max(0, Math.floor(n * 60_000))
    case 'h':
      return Math.max(0, Math.floor(n * 3_600_000))
    default:
      return Math.max(0, Math.floor(n))
  }
}

/**
 * Sanitize operator model string for filesystem path segment.
 * Path-safe lowercase slug + short content hash so punctuated variants
 * (`foo/bar` vs `foo-bar`) never share a directory.
 */
export function modelSlug(model: string): string {
  const raw = model.trim()
  if (!raw) {
    throw new Error(
      `model slug empty after sanitize (input was ${JSON.stringify(model)})`,
    )
  }
  const base = raw
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '')
    .slice(0, 60)
  // Stable short hash of the *raw* operator string (preserves distinction).
  const hash = createHash('sha256').update(raw).digest('hex').slice(0, 8)
  const s = (base ? `${base}-${hash}` : hash).slice(0, 80)
  return s
}

/**
 * Triple identity on disk: runs/<date>/<task>/<arm>/<modelSlug>/
 */
export function tripleRunDir(
  runsRoot: string,
  date: string,
  task: string,
  arm: string,
  model: string,
): string {
  return join(runsRoot, date, task, arm, modelSlug(model))
}

/**
 * Match operator fuzzy model against a recorded (resolved) label.
 * - Empty recorded → never complete (corrupt/partial write is retriable).
 * - Exact case-insensitive equality, OR operator fuzzy is a substring of the
 *   *recorded* label (one-way only — avoids grok matching as complete for grok-2).
 */
export function modelsMatch(operatorModel: string, recordedModel: string): boolean {
  const needle = operatorModel.trim().toLowerCase()
  const hay = recordedModel.trim().toLowerCase()
  if (!needle || !hay) return false
  return hay === needle || hay.includes(needle)
}

/**
 * A (task, arm, model) triple is complete when grades.json exists and
 * result.json records a non-empty model that matches the operator fuzzy.
 */
export function isTripleComplete(
  runsRoot: string,
  date: string,
  task: string,
  arm: string,
  model: string,
): { complete: boolean; runDir: string } {
  // Prefer model-keyed path; also probe legacy …/<task>/<arm>/ for older runs.
  const candidates = [
    tripleRunDir(runsRoot, date, task, arm, model),
    join(runsRoot, date, task, arm),
  ]
  for (const runDir of candidates) {
    const status = probeComplete(runDir, model)
    if (status.complete) return status
    // If model-keyed dir exists but incomplete, do not fall through to legacy
    // when the keyed path has partial artifacts (avoids wrong resume).
    if (runDir === candidates[0] && existsSync(join(runDir, 'result.json'))) {
      return status
    }
  }
  return { complete: false, runDir: candidates[0]! }
}

function probeComplete(
  runDir: string,
  model: string,
): { complete: boolean; runDir: string } {
  const gradesPath = join(runDir, 'grades.json')
  const resultPath = join(runDir, 'result.json')
  if (!existsSync(gradesPath) || !existsSync(resultPath)) {
    return { complete: false, runDir }
  }
  try {
    const result = JSON.parse(readFileSync(resultPath, 'utf8')) as {
      model?: string
      trajectory?: { model?: string; meta?: { modelRequested?: string | null } }
      error?: string
    }
    // Incomplete hard-error runs are retriable.
    if (result.error) return { complete: false, runDir }
    const recorded =
      (result.model && String(result.model).trim()) ||
      (result.trajectory?.model && String(result.trajectory.model).trim()) ||
      (result.trajectory?.meta?.modelRequested &&
        String(result.trajectory.meta.modelRequested).trim()) ||
      ''
    // Empty recorded model → incomplete (never match via ''.includes).
    if (!recorded) return { complete: false, runDir }
    if (modelsMatch(model, recorded)) {
      return { complete: true, runDir }
    }
    return { complete: false, runDir }
  } catch {
    return { complete: false, runDir }
  }
}

/** Also scan prior dates under runsRoot for the same triple (cross-night resume). */
export function findCompletedTriple(
  runsRoot: string,
  task: string,
  arm: string,
  model: string,
): string | null {
  if (!existsSync(runsRoot)) return null
  let dates: string[]
  try {
    dates = readdirSync(runsRoot).filter((d) => {
      try {
        return statSync(join(runsRoot, d)).isDirectory()
      } catch {
        return false
      }
    })
  } catch {
    return null
  }
  // Newest first
  dates.sort().reverse()
  for (const date of dates) {
    const { complete, runDir } = isTripleComplete(runsRoot, date, task, arm, model)
    if (complete) return runDir
  }
  return null
}

export async function runPaced(opts: PaceOptions): Promise<CorpusSummary> {
  if (!opts.model || !String(opts.model).trim()) {
    throw new Error(
      'pace requires --model <fuzzy> (no hardcoded default — operator supplies the model)',
    )
  }
  const model = String(opts.model).trim()
  const intervalMs = opts.intervalMs ?? 10 * 60_000
  const lane: Lane = opts.lane ?? 'frontier'
  const host = opts.host ?? realHost()
  // Paced runs are omp-only (EVAL-1 law).
  const adapter = opts.adapter ?? createDispatch('omp')
  if (adapter.mode !== 'omp' && !opts.dryRun) {
    throw new Error(`pace is omp-only (got dispatch ${adapter.mode})`)
  }
  const allowSkip = opts.allowSkip ?? false
  const runsRoot = opts.runsRoot ?? runsDir()
  const startedAt = new Date().toISOString()
  const date = startedAt.slice(0, 10)

  const arms = loadArms(opts.arms, opts.armsRoot)
  if (!arms.length) {
    throw new Error('no arms loaded — check eval/arms/<name>/arm.jsonc')
  }

  let tasks: LoadedTask[]
  if (opts.tasks?.length) {
    tasks = opts.tasks.map((s) => loadTask(assertSafeSlug(s), opts.tasksRoot))
  } else {
    // Overnight default: full corpus (all tasks).
    tasks = loadCorpus(opts.tasksRoot)
  }

  const plan: PacePlanEntry[] = []
  for (const task of tasks) {
    for (const arm of arms) {
      const prior = findCompletedTriple(runsRoot, task.slug, arm.id, model)
      const runDir = prior ?? tripleRunDir(runsRoot, date, task.slug, arm.id, model)
      plan.push({
        task: task.slug,
        arm: arm.id,
        model,
        status: prior ? 'skipped-resume' : 'pending',
        runDir,
      })
    }
  }

  let resumed = 0
  let executed = 0
  const runs: RunResult[] = []

  // Load resumed results into aggregate.
  for (const entry of plan) {
    if (entry.status !== 'skipped-resume') continue
    resumed++
    const loaded = loadExistingResult(entry.runDir, entry.arm)
    if (loaded) {
      runs.push(loaded)
    }
  }

  const pending = plan.filter((e) => e.status === 'pending')
  for (let i = 0; i < pending.length; i++) {
    const entry = pending[i]!
    if (opts.dryRun) {
      entry.status = 'pending'
      continue
    }

    const task = tasks.find((t) => t.slug === entry.task)!
    const arm = arms.find((a) => a.id === entry.arm)!
    const result = await runOnePaced({
      task,
      arm,
      lane,
      model,
      apiKey: opts.apiKey,
      host,
      adapter,
      runDir: entry.runDir,
      keepWorkdir: opts.keepWorkdir,
      behaviorBaseUrl: opts.behaviorBaseUrl,
      beautyScore: opts.beautyScores?.[task.slug],
      allowSkip,
      allowSkipBeauty: opts.allowSkipBeauty ?? true,
    })
    runs.push(result)
    executed++
    entry.status = 'completed'

    // Sleep between runs (not after the last).
    if (i < pending.length - 1 && !opts.noSleep && intervalMs > 0) {
      await sleep(intervalMs)
    }
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
    dispatch: 'omp',
    startedAt,
    finishedAt,
    runs,
    passCount,
    failCount,
    skipCount,
    arms: arms.map((a) => a.id),
    model,
    paced: { intervalMs, resumed, executed },
  }

  if (opts.writeReport !== false) {
    writeResults(summary)
  }
  return summary
}

async function runOnePaced(args: {
  task: LoadedTask
  arm: LoadedArm
  lane: Lane
  model: string
  apiKey?: string
  host: HostTooling
  adapter: DispatchAdapter
  runDir: string
  keepWorkdir?: boolean
  behaviorBaseUrl?: string
  beautyScore?: number
  allowSkip: boolean
  allowSkipBeauty: boolean
}): Promise<RunResult> {
  const { task, arm, lane, model, host, adapter } = args
  const startedAt = new Date().toISOString()
  const t0 = Date.now()
  const runDir = args.runDir
  mkdirSync(runDir, { recursive: true })

  // Fresh workdir every attempt — never inherit leftover agent files from retries.
  const workdir = join(runDir, 'workdir')
  if (existsSync(workdir)) {
    rmSync(workdir, { recursive: true, force: true })
  }
  mkdirSync(workdir, { recursive: true })

  // Screenshots dir for operator beauty (EVAL-1: capture is operator-owned; see README).
  const shotsDir = join(runDir, 'screenshots')
  mkdirSync(shotsDir, { recursive: true })
  writeFileSync(
    join(shotsDir, 'README.md'),
    [
      '# Beauty screenshots (operator-owned)',
      '',
      'EVAL-1 does not auto-capture screenshots. Drop PNGs here after the run for',
      'post-hoc beauty scoring (rubric in eval/beauty-rubric.md). Future harness',
      'versions may add optional playwright/`--base-url` capture.',
      '',
    ].join('\n'),
  )

  try {
    materializeArm(workdir, arm)
    const grading = gradingForArm(task.def.grading, arm.def)

    // sailOnPath: inject SAIL_BIN + prepend bin dir to PATH for the omp child.
    const env: Record<string, string> = {}
    if (arm.def.context.sailOnPath) {
      const sailBin = host.sailBin ?? findSailBin()
      if (sailBin) {
        env.SAIL_BIN = sailBin
        const binDir = dirname(sailBin)
        env.PATH = `${binDir}${delimiter}${process.env.PATH ?? ''}`
      }
    }

    const trajectory = await adapter.run({
      prompt: task.def.prompt,
      workdir,
      model: model || undefined,
      apiKey: args.apiKey,
      task: task.slug,
      fixtureDir: null, // live omp — no golden replay
      scaffold: arm.def.scaffold,
      sailArtifacts: arm.def.context.sailArtifacts,
      env: Object.keys(env).length ? env : undefined,
    })

    const usedStaticFallback =
      Boolean(grading.behavior) &&
      (lane === 'frontier' || lane === 'cheap') &&
      !args.behaviorBaseUrl
    const behaviorHtml = usedStaticFallback
      ? loadBehaviorHtmlFromWorkdir(workdir) ?? loadNonSailHtml(workdir)
      : undefined

    const grades = await runGradeChain({
      host,
      workdir,
      lane,
      grading,
      trajectory,
      spec: task.spec,
      behaviorHtml,
      behaviorBaseUrl: args.behaviorBaseUrl,
      beautyScore: args.beautyScore,
      allowSkip: args.allowSkip,
      enableCheapBehavior: false,
      allowSkipBeauty: args.allowSkipBeauty,
    })

    // Label static source fallback so RESULTS/grades are honest (not a live UI).
    if (
      usedStaticFallback &&
      grades.behavior &&
      !grades.behavior.skipped &&
      !args.behaviorBaseUrl
    ) {
      const tag =
        'static source-text fallback (not a running UI; pass --base-url for live behavior)'
      grades.behavior.detail = grades.behavior.detail
        ? `${grades.behavior.detail}; ${tag}`
        : tag
    }

    const finishedAt = new Date().toISOString()
    const result: RunResult = {
      task: task.slug,
      arm: arm.id,
      lane,
      dispatch: 'omp',
      model: trajectory.model || model,
      workdir: args.keepWorkdir ? workdir : '<workdir>',
      runDir: args.keepWorkdir ? runDir : '<runDir>',
      trajectory: sanitizeJson(trajectory, workdir),
      grades,
      startedAt,
      finishedAt,
      wallMs: Date.now() - t0,
    }
    writeRunArtifacts(runDir, result, trajectory, workdir)
    return result
  } catch (e) {
    const finishedAt = new Date().toISOString()
    const msg = e instanceof Error ? e.message : String(e)
    const emptyTraj: Trajectory = {
      task: task.slug,
      model,
      dispatch: 'omp',
      turns: [],
      turnCount: 0,
      startedAt,
      finishedAt,
      filesWritten: [],
    }
    const result: RunResult = {
      task: task.slug,
      arm: arm.id,
      lane,
      dispatch: 'omp',
      model,
      workdir: '<workdir>',
      runDir: '<runDir>',
      trajectory: emptyTraj,
      grades: { ok: false } satisfies GradeReport,
      startedAt,
      finishedAt,
      wallMs: Date.now() - t0,
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

/**
 * Static source-text fallback for control arms (no --base-url).
 * Prefer implementation sources first: Vite react-tw puts UI in App.jsx while
 * index.html is only the empty #root host. Vanilla has no App.* → index.html.
 */
export function loadNonSailHtml(workdir: string): string | undefined {
  for (const rel of ['src/App.jsx', 'src/App.tsx', 'index.html']) {
    const p = join(workdir, rel)
    if (existsSync(p)) return readFileSync(p, 'utf8')
  }
  return undefined
}

function loadExistingResult(runDir: string, armHint?: string): RunResult | null {
  try {
    const raw = readFileSync(join(runDir, 'result.json'), 'utf8')
    const r = JSON.parse(raw) as RunResult
    if (!r.arm) {
      r.arm = armHint
      if (!r.arm) {
        // Path: …/<task>/<arm>/<modelSlug> → penultimate is arm
        const parts = runDir.replace(/\\/g, '/').split('/').filter(Boolean)
        if (parts.length >= 2) r.arm = parts[parts.length - 2]
      }
    }
    return r
  } catch {
    return null
  }
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
  writeFileSync(
    join(runDir, 'trajectory.jsonl'),
    lines.join('\n') + (lines.length ? '\n' : ''),
  )
}

function sleep(ms: number): Promise<void> {
  return new Promise((r) => setTimeout(r, ms))
}
