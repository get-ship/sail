import { writeFileSync } from 'node:fs'
import type { CorpusSummary, GradeReport, RunResult } from './types.ts'
import { resultsPath } from './paths.ts'
import { sanitizeNote } from './sanitize.ts'

function pct(n: number, d: number): string {
  if (d === 0) return '—'
  return `${((100 * n) / d).toFixed(0)}%`
}

function gradeCell(g: GradeReport): string {
  const parts: string[] = []
  if (g.compile) {
    parts.push(g.compile.skipped ? 'compile:skip' : g.compile.ok ? 'compile:✓' : 'compile:✗')
  }
  if (g.vocab_share) {
    if (g.vocab_share.skipped) parts.push('vocab:skip')
    else parts.push(`vocab:${(g.vocab_share.share * 100).toFixed(0)}%`)
  }
  if (g.gates) {
    parts.push(g.gates.skipped ? 'gates:skip' : g.gates.ok ? 'gates:✓' : `gates:✗(${g.gates.lintErrors})`)
  }
  if (g.behavior) {
    if (g.behavior.skipped) parts.push('behavior:skip')
    else parts.push(`behavior:${g.behavior.passed}/${g.behavior.passed + g.behavior.failed}`)
  }
  if (g.beauty) {
    if (g.beauty.skipped) parts.push('beauty:skip')
    else parts.push(`beauty:${g.beauty.score}/${g.beauty.maxScore}`)
  }
  return parts.join(' ')
}

function turnsCell(r: RunResult): string {
  const it = r.grades.compile?.iterationsToGreen
  if (it != null) return String(it)
  return String(r.trajectory.turnCount)
}

/** Build RESULTS.md markdown from a corpus summary. Aggregates only — no raw paths. */
export function formatResultsMd(summary: CorpusSummary): string {
  const lines: string[] = []
  lines.push('# Sail eval RESULTS')
  lines.push('')
  // Honesty marker — unmistakable: recorded 100% is fixture↔golden, not agent skill.
  if (summary.dispatch === 'recorded') {
    lines.push(
      '> **UNPROVEN harness.** This report is **recorded-lane fixture replay**: graders score committed goldens, not a live agent. A 100% pass rate here does **not** mean agents author good sail. Real evidence requires `--dispatch omp` runs.',
    )
  } else {
    lines.push(
      '> Pre-alpha honesty: live omp-dispatched run summary. Trajectories under `eval/runs/` (gitignored). Aggregates only here. Harness remains UNPROVEN until a corpus of real omp runs + control arms lands.',
    )
  }
  lines.push('> Trajectories live under `eval/runs/` (gitignored). Aggregates only here.')
  lines.push('')
  lines.push(`- **Lane:** \`${summary.lane}\``)
  lines.push(`- **Dispatch:** \`${summary.dispatch}\``)
  lines.push(`- **Started:** ${summary.startedAt}`)
  lines.push(`- **Finished:** ${summary.finishedAt}`)
  lines.push(
    `- **Pass / Fail / Skip:** ${summary.passCount} / ${summary.failCount} / ${summary.skipCount}`,
  )
  lines.push(
    `- **Pass rate:** ${pct(summary.passCount, summary.passCount + summary.failCount)}`,
  )
  lines.push('')
  lines.push('## Tasks')
  lines.push('')
  lines.push('| Task | OK | Turns | Grades | Notes |')
  lines.push('|---|:---:|---:|---|---|')
  for (const r of summary.runs) {
    const ok = r.error ? 'err' : r.grades.ok ? '✓' : '✗'
    const notes = r.error
      ? sanitizeNote(r.error)
      : r.grades.compile?.detail
        ? sanitizeNote(r.grades.compile.detail)
        : r.grades.vocab_share?.detail
          ? sanitizeNote(r.grades.vocab_share.detail)
          : ''
    lines.push(
      `| ${r.task} | ${ok} | ${turnsCell(r)} | ${gradeCell(r.grades)} | ${notes} |`,
    )
  }
  lines.push('')
  lines.push('## Leg averages')
  lines.push('')

  const vocabShares = summary.runs
    .map((r) => r.grades.vocab_share)
    .filter((v): v is NonNullable<typeof v> => !!v && !v.skipped)
  if (vocabShares.length) {
    const avg =
      vocabShares.reduce((a, v) => a + v.share, 0) / vocabShares.length
    lines.push(`- **Vocab share (mean):** ${(avg * 100).toFixed(1)}% (n=${vocabShares.length})`)
  } else {
    lines.push('- **Vocab share (mean):** —')
  }

  const compiles = summary.runs
    .map((r) => r.grades.compile)
    .filter((c): c is NonNullable<typeof c> => !!c && !c.skipped)
  if (compiles.length) {
    const green = compiles.filter((c) => c.ok).length
    const repairs = compiles.reduce((a, c) => a + c.repairCount, 0)
    lines.push(`- **Compile green:** ${green}/${compiles.length}`)
    lines.push(`- **Auto-repairs (sum):** ${repairs}`)
  }

  const turns = summary.runs.map((r) => r.trajectory.turnCount)
  if (turns.length) {
    const mean = turns.reduce((a, b) => a + b, 0) / turns.length
    lines.push(`- **Turns (mean):** ${mean.toFixed(1)}`)
  }

  lines.push('')
  lines.push('## Notes')
  lines.push('')
  lines.push(
    '- Cheap lane grades compile + vocab_share + gates only (pass `--cheap-behavior` for source-text heuristics).',
  )
  lines.push(
    '- Frontier adds playwright behavior + beauty (judge/operator). Missing beauty score is incomplete unless `--allow-skip-beauty`.',
  )
  lines.push(
    '- Studio vocab low-water mark: **17%**. Default min share: **25%**. Strict mode: missing sail fails (not pass).',
  )
  lines.push('')
  return lines.join('\n')
}

export function writeResults(
  summary: CorpusSummary,
  path = resultsPath(),
): void {
  writeFileSync(path, formatResultsMd(summary), 'utf8')
}
