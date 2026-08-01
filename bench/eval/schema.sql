-- HARNESS-2 run store (slim). Blobs live on disk under the run dir;
-- sqlite holds metrics, grades, artifact pointers only.
-- Schema versioned via schema_meta; migrate in store/db.ts.

PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS schema_meta (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS toolchains (
  toolchain_id  TEXT PRIMARY KEY,
  stamp         TEXT NOT NULL UNIQUE,
  created_at    TEXT NOT NULL,
  sail_bin      TEXT,
  meta_json     TEXT
);

CREATE TABLE IF NOT EXISTS tasks (
  task_id       TEXT PRIMARY KEY,
  prompt_hash   TEXT,
  systems_json  TEXT,
  meta_json     TEXT
);

CREATE TABLE IF NOT EXISTS profiles (
  profile_id    TEXT PRIMARY KEY,
  content_hash  TEXT NOT NULL,
  body_json     TEXT NOT NULL,
  created_at    TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS batches (
  batch_id       TEXT PRIMARY KEY,
  ts             TEXT NOT NULL,
  name           TEXT NOT NULL,
  question       TEXT,
  profile_hash   TEXT NOT NULL,
  harness_ver    TEXT NOT NULL,
  status         TEXT NOT NULL,
  n              INTEGER NOT NULL DEFAULT 1,
  tasks_json     TEXT,
  arms_json      TEXT,
  constants_json TEXT,
  report_dir     TEXT,
  manifest_json  TEXT
);

CREATE TABLE IF NOT EXISTS runs (
  run_id              TEXT PRIMARY KEY,  -- word-pair id
  batch_id            TEXT NOT NULL,
  ts                  TEXT NOT NULL,
  task_id             TEXT NOT NULL,
  arm                 TEXT NOT NULL,
  model               TEXT,              -- resolved model string; NULL + flag if missing
  model_slug          TEXT NOT NULL,
  toolchain           TEXT NOT NULL,
  profile_hash        TEXT NOT NULL,
  harness_ver         TEXT NOT NULL,
  run_dir             TEXT NOT NULL,     -- absolute path; artifacts live here
  status              TEXT NOT NULL,     -- running|complete|failed|quarantine
  wall_ms             INTEGER,
  gate_ok             INTEGER,           -- 0/1/NULL
  compile_ok          INTEGER,           -- 0/1/NULL
  render_ok           INTEGER,           -- 0/1/NULL (blocking: NULL/0 = failed run)
  tokens_in           INTEGER,
  tokens_out          INTEGER,
  turns               INTEGER,
  session_id          TEXT,
  render_method       TEXT,
  flags_json          TEXT,              -- explicit missing-data reasons
  metrics_json        TEXT,
  gate_detail_json    TEXT,
  artifacts_json      TEXT,              -- declared artifacts + hashes + blocking
  failure_classes_json TEXT,
  missing_reasons_json TEXT,             -- map field -> named reason (NULL semantics)
  FOREIGN KEY (batch_id) REFERENCES batches(batch_id)
);

CREATE TABLE IF NOT EXISTS artifacts (
  run_id        TEXT NOT NULL,
  name          TEXT NOT NULL,           -- e.g. render.png, events.jsonl
  rel_path      TEXT NOT NULL,           -- relative to run_dir
  sha256        TEXT,
  bytes         INTEGER,
  blocking      INTEGER NOT NULL DEFAULT 0,
  status        TEXT NOT NULL,           -- present|missing|degraded
  reason        TEXT,                    -- named reason when missing/degraded
  PRIMARY KEY (run_id, name),
  FOREIGN KEY (run_id) REFERENCES runs(run_id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS grades (
  run_id        TEXT NOT NULL,
  key           TEXT NOT NULL,
  value         TEXT,
  note          TEXT,
  grader        TEXT,                    -- operator|bot|…
  ts            TEXT NOT NULL,
  PRIMARY KEY (run_id, key),
  FOREIGN KEY (run_id) REFERENCES runs(run_id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS comments (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  run_id        TEXT NOT NULL,
  body          TEXT NOT NULL,
  author        TEXT,
  ts            TEXT NOT NULL,
  FOREIGN KEY (run_id) REFERENCES runs(run_id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_runs_batch ON runs(batch_id);
CREATE INDEX IF NOT EXISTS idx_runs_task_arm ON runs(task_id, arm);
CREATE INDEX IF NOT EXISTS idx_runs_toolchain ON runs(toolchain);
CREATE INDEX IF NOT EXISTS idx_runs_ts ON runs(ts);
