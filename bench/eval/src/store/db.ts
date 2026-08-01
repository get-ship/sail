/**
 * bun:sqlite store — metrics, grades, artifact pointers only.
 * Events/transcripts stay as files in the run dir.
 */

import { Database } from 'bun:sqlite'
import { existsSync, mkdirSync, readFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import type { ArtifactDecl } from '../contract/types.ts'

const SCHEMA_VERSION = '2'

export interface BatchRow {
  batch_id: string
  ts: string
  name: string
  question: string | null
  profile_hash: string
  harness_ver: string
  status: string
  n: number
  tasks_json: string | null
  arms_json: string | null
  constants_json: string | null
  report_dir: string | null
  manifest_json: string | null
}

export interface RunRow {
  run_id: string
  batch_id: string
  ts: string
  task_id: string
  arm: string
  model: string | null
  model_slug: string
  toolchain: string
  profile_hash: string
  harness_ver: string
  run_dir: string
  status: string
  wall_ms: number | null
  gate_ok: number | null
  compile_ok: number | null
  render_ok: number | null
  tokens_in: number | null
  tokens_out: number | null
  turns: number | null
  session_id: string | null
  render_method: string | null
  flags_json: string | null
  metrics_json: string | null
  gate_detail_json: string | null
  artifacts_json: string | null
  failure_classes_json: string | null
  missing_reasons_json: string | null
}

export interface GradeRow {
  run_id: string
  key: string
  value: string | null
  note: string | null
  grader: string | null
  ts: string
}

export class StoreError extends Error {
  constructor(
    message: string,
    readonly code: string = 'STORE',
  ) {
    super(message)
    this.name = 'StoreError'
  }
}

export class RunStore {
  readonly path: string
  private db: Database

  private constructor(path: string, db: Database) {
    this.path = path
    this.db = db
  }

  static open(dbPath: string): RunStore {
    mkdirSync(dirname(dbPath), { recursive: true })
    const db = new Database(dbPath, { create: true })
    db.exec('PRAGMA foreign_keys = ON;')
    const store = new RunStore(dbPath, db)
    store.migrate()
    return store
  }

  static openExisting(dbPath: string): RunStore {
    if (!existsSync(dbPath)) {
      throw new StoreError(`store not found at ${dbPath}`, 'NOT_FOUND')
    }
    return RunStore.open(dbPath)
  }

  close(): void {
    this.db.close()
  }

  private migrate(): void {
    // Base tables (CREATE IF NOT EXISTS) — greenfield + idempotent.
    const schemaPath = join(dirname(fileURLToPath(import.meta.url)), '../../schema.sql')
    if (existsSync(schemaPath)) {
      this.db.exec(readFileSync(schemaPath, 'utf8'))
    }

    const row = this.db.query('SELECT value FROM schema_meta WHERE key = ?').get('version') as
      | { value: string }
      | null
    let current = row?.value ?? '0'
    if (!row) {
      this.db
        .query('INSERT INTO schema_meta (key, value) VALUES (?, ?)')
        .run('version', '0')
      current = '0'
    }

    const curNum = Number.parseInt(current, 10)
    const targetNum = Number.parseInt(SCHEMA_VERSION, 10)
    if (Number.isFinite(curNum) && Number.isFinite(targetNum) && curNum > targetNum) {
      throw new StoreError(
        `store schema version ${current} is newer than supported ${SCHEMA_VERSION}`,
        'SCHEMA_NEWER',
      )
    }

    // Ordered migrations: version N applies steps to reach N+1, …
    // v0 → v2: baseline schema.sql already applied (greenfield phase-1).
    // Future ALTER TABLE steps go here keyed by from-version.
    const steps: Array<{ from: string; to: string; apply: () => void }> = [
      {
        from: '0',
        to: '2',
        apply: () => {
          // schema.sql CREATE IF NOT EXISTS is the v2 baseline; no ALTERs yet.
        },
      },
    ]

    let ver = current
    for (const step of steps) {
      if (ver === step.from) {
        step.apply()
        this.db
          .query(
            `INSERT INTO schema_meta (key, value) VALUES ('version', ?)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value`,
          )
          .run(step.to)
        ver = step.to
      }
    }

    if (ver !== SCHEMA_VERSION) {
      // Unknown intermediate — stamp target only if still behind and no steps matched
      // (e.g. fresh insert path already at schema tables).
      if (ver === '0' || Number.parseInt(ver, 10) < targetNum) {
        this.db
          .query(
            `INSERT INTO schema_meta (key, value) VALUES ('version', ?)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value`,
          )
          .run(SCHEMA_VERSION)
      }
    }
  }

  upsertBatch(row: BatchRow): void {
    const existing = this.getBatch(row.batch_id)
    if (existing && existing.profile_hash !== row.profile_hash) {
      throw new StoreError(
        `profile immutable: batch ${row.batch_id} stored hash ${existing.profile_hash} != ${row.profile_hash}`,
        'PROFILE_IMMUTABLE',
      )
    }
    if (existing) {
      this.db
        .query(
          `UPDATE batches SET status=?, report_dir=COALESCE(?, report_dir),
           manifest_json=COALESCE(?, manifest_json) WHERE batch_id=?`,
        )
        .run(row.status, row.report_dir, row.manifest_json, row.batch_id)
      return
    }
    this.db
      .query(
        `INSERT INTO batches (
          batch_id, ts, name, question, profile_hash, harness_ver, status, n,
          tasks_json, arms_json, constants_json, report_dir, manifest_json
        ) VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?)`,
      )
      .run(
        row.batch_id,
        row.ts,
        row.name,
        row.question,
        row.profile_hash,
        row.harness_ver,
        row.status,
        row.n,
        row.tasks_json,
        row.arms_json,
        row.constants_json,
        row.report_dir,
        row.manifest_json,
      )
  }

  getBatch(batchId: string): BatchRow | null {
    return (
      (this.db.query('SELECT * FROM batches WHERE batch_id = ?').get(batchId) as BatchRow | null) ??
      null
    )
  }

  resolveBatchId(ref: string): string {
    if (this.getBatch(ref)) return ref
    const all = this.db.query('SELECT batch_id, report_dir FROM batches').all() as Array<{
      batch_id: string
      report_dir: string | null
    }>
    for (const r of all) {
      if (r.report_dir && (r.report_dir.endsWith(ref) || r.report_dir.includes(ref))) {
        return r.batch_id
      }
    }
    throw new StoreError(`unknown batch '${ref}'`, 'UNKNOWN_BATCH')
  }

  upsertRun(row: RunRow): void {
    this.db
      .query(
        `INSERT INTO runs (
          run_id, batch_id, ts, task_id, arm, model, model_slug, toolchain,
          profile_hash, harness_ver, run_dir, status, wall_ms, gate_ok, compile_ok,
          render_ok, tokens_in, tokens_out, turns, session_id, render_method,
          flags_json, metrics_json, gate_detail_json, artifacts_json,
          failure_classes_json, missing_reasons_json
        ) VALUES (
          ?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?
        )
        ON CONFLICT(run_id) DO UPDATE SET
          status=excluded.status,
          wall_ms=excluded.wall_ms,
          gate_ok=excluded.gate_ok,
          compile_ok=excluded.compile_ok,
          render_ok=excluded.render_ok,
          tokens_in=excluded.tokens_in,
          tokens_out=excluded.tokens_out,
          turns=excluded.turns,
          session_id=excluded.session_id,
          render_method=excluded.render_method,
          flags_json=excluded.flags_json,
          metrics_json=excluded.metrics_json,
          gate_detail_json=excluded.gate_detail_json,
          artifacts_json=excluded.artifacts_json,
          failure_classes_json=excluded.failure_classes_json,
          missing_reasons_json=excluded.missing_reasons_json`,
      )
      .run(
        row.run_id,
        row.batch_id,
        row.ts,
        row.task_id,
        row.arm,
        row.model,
        row.model_slug,
        row.toolchain,
        row.profile_hash,
        row.harness_ver,
        row.run_dir,
        row.status,
        row.wall_ms,
        row.gate_ok,
        row.compile_ok,
        row.render_ok,
        row.tokens_in,
        row.tokens_out,
        row.turns,
        row.session_id,
        row.render_method,
        row.flags_json,
        row.metrics_json,
        row.gate_detail_json,
        row.artifacts_json,
        row.failure_classes_json,
        row.missing_reasons_json,
      )
  }

  getRun(runId: string): RunRow | null {
    return (this.db.query('SELECT * FROM runs WHERE run_id = ?').get(runId) as RunRow | null) ?? null
  }

  /** Resolve by run_id or by run_dir basename / absolute path. */
  resolveRun(ref: string): RunRow {
    const byId = this.getRun(ref)
    if (byId) return byId
    const byDir = this.db.query('SELECT * FROM runs WHERE run_dir = ?').get(ref) as RunRow | null
    if (byDir) return byDir
    const all = this.db.query('SELECT * FROM runs').all() as RunRow[]
    const hit = all.find((r) => r.run_dir.endsWith(`/${ref}`) || r.run_dir.endsWith(ref))
    if (hit) return hit
    throw new StoreError(`unknown run '${ref}'`, 'UNKNOWN_RUN')
  }

  runsForBatch(batchId: string): RunRow[] {
    return this.db
      .query('SELECT * FROM runs WHERE batch_id = ? ORDER BY ts, run_id')
      .all(batchId) as RunRow[]
  }

  upsertArtifacts(runId: string, arts: ArtifactDecl[]): void {
    const stmt = this.db.query(
      `INSERT INTO artifacts (run_id, name, rel_path, sha256, bytes, blocking, status, reason)
       VALUES (?,?,?,?,?,?,?,?)
       ON CONFLICT(run_id, name) DO UPDATE SET
         rel_path=excluded.rel_path,
         sha256=excluded.sha256,
         bytes=excluded.bytes,
         blocking=excluded.blocking,
         status=excluded.status,
         reason=excluded.reason`,
    )
    for (const a of arts) {
      stmt.run(
        runId,
        a.name,
        a.rel_path,
        a.sha256 ?? null,
        a.bytes ?? null,
        a.blocking ? 1 : 0,
        a.status,
        a.reason ?? null,
      )
    }
  }

  setGrade(row: GradeRow): void {
    this.db
      .query(
        `INSERT INTO grades (run_id, key, value, note, grader, ts)
         VALUES (?,?,?,?,?,?)
         ON CONFLICT(run_id, key) DO UPDATE SET
           value=excluded.value,
           note=excluded.note,
           grader=excluded.grader,
           ts=excluded.ts`,
      )
      .run(row.run_id, row.key, row.value, row.note, row.grader, row.ts)
  }

  gradesForRun(runId: string): GradeRow[] {
    return this.db.query('SELECT * FROM grades WHERE run_id = ?').all(runId) as GradeRow[]
  }

  addComment(runId: string, body: string, author: string | null): void {
    this.db
      .query('INSERT INTO comments (run_id, body, author, ts) VALUES (?,?,?,?)')
      .run(runId, body, author, new Date().toISOString())
  }
}
