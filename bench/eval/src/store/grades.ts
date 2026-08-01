/**
 * Grades API — same write path for CLI and future viewer.
 */

import type { RunStore } from './db.ts'

export interface GradeInput {
  /** run_id, run_dir, or basename */
  runRef: string
  pairs: Array<{ key: string; value: string }>
  note?: string
  grader?: string
}

export function writeGrades(store: RunStore, input: GradeInput) {
  const run = store.resolveRun(input.runRef)
  const ts = new Date().toISOString()
  const written = []
  for (const p of input.pairs) {
    const row = {
      run_id: run.run_id,
      key: p.key,
      value: p.value,
      note: input.note ?? null,
      grader: input.grader ?? 'operator',
      ts,
    }
    store.setGrade(row)
    written.push(row)
  }
  return { run_id: run.run_id, run_dir: run.run_dir, grades: written }
}
