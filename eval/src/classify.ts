/** Run pass/fail/skip classification shared by runner, pace, and RESULTS report. */

import type { Lane, RunResult } from './types.ts'

type LegStatus = { ok: boolean; skipped?: boolean }

/**
 * Classify a run for aggregate counting.
 * - fail: hard error, required leg failed, or incomplete required leg
 *   (`skipped && !ok` — e.g. frontier beauty without score)
 * - skip: tooling soft-skip only (`skipped && ok` on compile/vocab/gates, or
 *   soft-skipped frontier behavior) — never a pass
 * - pass: required legs green; beauty soft-skip (`--allow-skip-beauty`) is waived
 */
export function classifyRun(r: RunResult, lane: Lane): 'pass' | 'fail' | 'skip' {
  if (r.error) return 'fail'

  const tooling = [
    r.grades.compile,
    r.grades.vocab_share,
    r.grades.gates,
  ].filter((l): l is LegStatus => l != null)

  const frontierExtra =
    lane === 'frontier'
      ? [r.grades.behavior, r.grades.beauty].filter((l): l is LegStatus => l != null)
      : []

  const hardFail = (l: LegStatus) => !l.ok && !l.skipped
  const incomplete = (l: LegStatus) => Boolean(l.skipped && !l.ok)
  const softSkip = (l: LegStatus) => Boolean(l.skipped && l.ok)

  if (tooling.some(hardFail) || frontierExtra.some(hardFail)) return 'fail'
  // Incomplete required leg (beauty missing score, etc.) → fail so CLI exit ≠ 0
  if (tooling.some(incomplete) || frontierExtra.some(incomplete)) return 'fail'

  // Soft-skipped tooling (missing sail + --allow-skip): never pass
  if (tooling.some(softSkip)) return 'skip'

  // Frontier behavior soft-skip blocks pass; beauty soft-skip is waived
  if (lane === 'frontier' && r.grades.behavior && softSkip(r.grades.behavior)) {
    return 'skip'
  }
  // beauty with skipped&&ok (--allow-skip-beauty) does not block pass

  if (!r.grades.ok) return 'fail'
  return 'pass'
}
