import { readFileSync, existsSync } from 'node:fs'
import { join } from 'node:path'
import type { HostTooling } from '../host.ts'
import type {
  BehaviorSpec,
  GradeReport,
  Lane,
  TaskGrading,
  Trajectory,
} from '../types.ts'
import { gradeBehavior } from './behavior.ts'
import { gradeBeauty } from './beauty.ts'
import { gradeCompile } from './compile.ts'
import { gradeGates } from './gates.ts'
import { gradeVocab } from './vocab.ts'

export interface GradeChainOpts {
  host: HostTooling
  workdir: string
  lane: Lane
  grading: TaskGrading
  trajectory?: Trajectory
  spec?: BehaviorSpec | null
  /**
   * Static HTML for behavior grading. Only used when:
   * - lane is frontier with baseUrl unavailable, or
   * - `enableCheapBehavior` is true (explicit opt-in; not the cheap default).
   */
  behaviorHtml?: string
  /** Live URL for playwright behavior. */
  behaviorBaseUrl?: string
  beautyScore?: number | null
  allowSkip?: boolean
  /**
   * Opt-in: run source-text behavior heuristics on cheap lane.
   * Spec cheap lane is compile+vocab+gates only; this is not deck/playwright.
   */
  enableCheapBehavior?: boolean
  /** Soft-skip frontier beauty when no score (default false → incomplete). */
  allowSkipBeauty?: boolean
}

/**
 * Grading chain in order: compile → vocab_share → gates → behavior → beauty.
 * Cheap lane (default): compile + vocab + gates only.
 * Frontier: all enabled legs; beauty without score is incomplete unless allowSkipBeauty.
 */
export async function runGradeChain(opts: GradeChainOpts): Promise<GradeReport> {
  const g = opts.grading
  const allowSkip = opts.allowSkip ?? false
  const report: GradeReport = { ok: true }

  const wantCompile = g.compile !== false
  const wantVocab = Boolean(g.vocab_share)
  const wantGates = Boolean(g.gates)
  const wantBeauty = Boolean(g.beauty)

  // Behavior: frontier always when enabled; cheap only with explicit opt-in.
  const wantBehavior =
    Boolean(g.behavior) &&
    (opts.lane === 'frontier' ||
      (opts.lane === 'cheap' && Boolean(opts.enableCheapBehavior)))

  if (wantCompile) {
    report.compile = await gradeCompile({
      host: opts.host,
      workdir: opts.workdir,
      trajectory: opts.trajectory,
      allowSkip,
    })
    if (!report.compile.ok && !report.compile.skipped) report.ok = false
    if (report.compile.skipped && !allowSkip) report.ok = false
  }

  if (wantVocab) {
    const min =
      typeof g.vocab_share === 'object' && g.vocab_share && 'min' in g.vocab_share
        ? g.vocab_share.min
        : undefined
    report.vocab_share = await gradeVocab({
      host: opts.host,
      workdir: opts.workdir,
      minShare: min,
      allowSkip,
    })
    if (!report.vocab_share.ok && !report.vocab_share.skipped) report.ok = false
    if (report.vocab_share.skipped && !allowSkip) report.ok = false
  }

  if (wantGates) {
    report.gates = await gradeGates({
      host: opts.host,
      workdir: opts.workdir,
      allowSkip,
    })
    if (!report.gates.ok && !report.gates.skipped) report.ok = false
    if (report.gates.skipped && !allowSkip) report.ok = false
  }

  if (wantBehavior) {
    const html =
      opts.behaviorBaseUrl != null
        ? undefined
        : opts.behaviorHtml
    report.behavior = await gradeBehavior({
      spec: opts.spec ?? null,
      html,
      baseUrl: opts.behaviorBaseUrl,
      allowSkip,
    })
    if (!report.behavior.ok && !report.behavior.skipped) report.ok = false
  } else if (Boolean(g.behavior) && opts.lane === 'cheap') {
    report.behavior = {
      ok: true,
      passed: 0,
      failed: 0,
      results: [],
      skipped: true,
      detail:
        'cheap lane: behavior is source-text heuristics only — pass --cheap-behavior to enable',
    }
  }

  if (wantBeauty) {
    report.beauty = await gradeBeauty({
      lane: opts.lane,
      score: opts.beautyScore ?? null,
      allowSkip: opts.lane === 'frontier' ? Boolean(opts.allowSkipBeauty) : true,
    })
    if (!report.beauty.ok && !report.beauty.skipped) report.ok = false
    // Frontier missing score: beauty returns skipped + ok:false → not a pass
    if (opts.lane === 'frontier' && report.beauty.skipped && !report.beauty.ok) {
      report.ok = false
    }
  }

  return report
}

/** Best-effort: read routes/page.sail (or first .sail) as pseudo-HTML for static behavior. */
export function loadBehaviorHtmlFromWorkdir(workdir: string): string | undefined {
  const candidates = [
    join(workdir, 'routes', 'page.sail'),
    join(workdir, 'page.sail'),
  ]
  for (const c of candidates) {
    if (existsSync(c)) {
      // .sail is HTML-like enough for text/role queries on static structure.
      return readFileSync(c, 'utf8')
    }
  }
  return undefined
}
