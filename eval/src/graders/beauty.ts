import { existsSync, readFileSync } from 'node:fs'
import { beautyRubricPath } from '../paths.ts'
import type { BeautyGrade, Lane } from '../types.ts'

export interface BeautyGradeOpts {
  lane: Lane
  /**
   * Operator or judge score 0–10. When omitted:
   * - cheap: skipped (mechanical only + optional sample)
   * - frontier: incomplete unless allowSkip (default false for frontier pass)
   */
  score?: number | null
  notes?: string
  rubricPath?: string
  /**
   * When true, missing frontier score is a soft skip (ok: true).
   * When false (default), frontier without score is incomplete (ok: false).
   */
  allowSkip?: boolean
}

export const BEAUTY_MAX = 10
/** Pass threshold for frontier beauty when scored. */
export const BEAUTY_PASS_MIN = 6

export function loadRubric(path = beautyRubricPath()): string {
  if (!existsSync(path)) {
    return '# Beauty rubric missing'
  }
  return readFileSync(path, 'utf8')
}

/**
 * Beauty leg. Cheap lane skips by default (operator sample-grades offline).
 * Frontier expects a judge/operator score against the committed rubric.
 * This package does not pick a judge model — operator decision.
 */
export async function gradeBeauty(opts: BeautyGradeOpts): Promise<BeautyGrade> {
  const rubric = loadRubric(opts.rubricPath)
  if (!rubric.includes('Hierarchy')) {
    // Soft warning only — rubric file should stay versioned.
  }

  if (opts.lane === 'cheap') {
    if (opts.score == null) {
      return {
        ok: true,
        score: null,
        maxScore: BEAUTY_MAX,
        source: 'skipped',
        skipped: true,
        detail: 'cheap lane: beauty is operator sample-grade only',
        notes: opts.notes,
      }
    }
    return {
      ok: opts.score >= BEAUTY_PASS_MIN,
      score: opts.score,
      maxScore: BEAUTY_MAX,
      source: 'operator',
      notes: opts.notes,
    }
  }

  // frontier
  if (opts.score == null) {
    const allowSkip = opts.allowSkip ?? false
    if (allowSkip) {
      return {
        ok: true,
        score: null,
        maxScore: BEAUTY_MAX,
        source: 'skipped',
        skipped: true,
        detail:
          'frontier beauty skipped (--allow-skip-beauty); judge-model choice is an operator decision',
        notes: opts.notes,
      }
    }
    // Incomplete: not a pass. Skipped flag so classifyRun can count as skip
    // when allowSkip tooling, or fail overall grades.ok.
    return {
      ok: false,
      score: null,
      maxScore: BEAUTY_MAX,
      source: 'skipped',
      skipped: true,
      detail:
        'frontier beauty requires judge/operator score (pass --allow-skip-beauty to soft-skip)',
      notes: opts.notes,
    }
  }
  return {
    ok: opts.score >= BEAUTY_PASS_MIN,
    score: opts.score,
    maxScore: BEAUTY_MAX,
    source: 'judge',
    notes: opts.notes,
  }
}
