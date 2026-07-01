#!/usr/bin/env bun
// record.ts — replay graded jsonl rows into the sui-eval-runs collection.
//
// Reads .artifacts/sui-eval-runs/<run-id>.jsonl (one per lane), reshapes each
// row into the collection's field set, and POSTs `collections.create_item`
// via `ship ops call`. Best-effort: the jsonl artifact is the durable record;
// the collection makes it queryable from views.

import { execSync } from 'node:child_process'
import { existsSync, readFileSync, readdirSync } from 'node:fs'
import { join, resolve } from 'node:path'

interface LaneRow {
  runId: string
  laneId: string
  worktree: string
  compile: { passed: boolean; stderr: string }
  lint: { errors: number; warnings: number; output: string }
  structural: {
    manifestOk: boolean
    requiredFragments: { found: string[]; missing: string[] }
    cssTokenViolations: string[]
    viewFiles: string[]
  }
  behavior?: { total: number; passed: number; score: number }
  composite: number
}

const RUN_ID = process.env.SHIP_PIPELINE_RUN_ID ?? `local-${process.env.SHIP_ACTOR_ID ?? 'unknown'}`
// cwd-relative — must match where grade.ts writes. Run from the repo root.
const RUNS_DIR = resolve('.artifacts/sui-eval-runs')

if (!existsSync(RUNS_DIR)) {
  console.error(`record: no graded runs at ${RUNS_DIR} — did grade step run?`)
  process.exit(0)
}

// Read either the current run's file or every jsonl we find (after a daemon
// restart the env var may be unset; we still want to drain pending rows).
const files = existsSync(join(RUNS_DIR, `${RUN_ID}.jsonl`))
  ? [join(RUNS_DIR, `${RUN_ID}.jsonl`)]
  : readdirSync(RUNS_DIR).filter((f) => f.endsWith('.jsonl')).map((f) => join(RUNS_DIR, f))

let inserted = 0
let skipped = 0

for (const file of files) {
  const lines = readFileSync(file, 'utf8').split('\n').filter((l) => l.trim())
  for (const raw of lines) {
    let row: LaneRow
    try { row = JSON.parse(raw) as LaneRow } catch { skipped++; continue }
    const itemId = `${row.runId}/${row.laneId}`
    const data = {
      run_id: row.runId,
      lane_id: row.laneId,
      composite: Number(row.composite.toFixed(4)),
      compile_passed: row.compile.passed,
      lint_errors: row.lint.errors,
      lint_warnings: row.lint.warnings,
      worktree: row.worktree,
      view_files: row.structural.viewFiles,
      css_violations: row.structural.cssTokenViolations,
      missing_fragments: row.structural.requiredFragments.missing,
      ...(row.behavior
        ? {
            behavior_passed: row.behavior.passed,
            behavior_total: row.behavior.total,
            behavior_score: Number(row.behavior.score.toFixed(4)),
          }
        : {}),
    }
    const payload = JSON.stringify({
      collection: 'sui-eval-runs',
      id: itemId,
      data,
    })
    try {
      execSync(`ship ops call collections.create_item --json '${payload.replace(/'/g, "'\\''")}'`, {
        encoding: 'utf8',
        stdio: ['ignore', 'pipe', 'pipe'],
      })
      inserted++
    } catch (e) {
      const err = e as { stderr?: Buffer | string }
      const msg = err.stderr?.toString() ?? String(e)
      // Duplicate id = already-recorded; treat as skipped, not failure.
      if (msg.includes('UNIQUE') || msg.includes('already exists')) {
        skipped++
      } else {
        console.warn(`record: insert failed for ${itemId}: ${msg.slice(0, 200)}`)
        skipped++
      }
    }
  }
}

console.log(`record: inserted=${inserted} skipped=${skipped} (collection sui-eval-runs)`)
process.exit(0)
