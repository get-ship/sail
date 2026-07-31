-- Agent lab run-record schema (EVAL-2).
-- Schema is committed under bench/; the store is lab.sqlite outside the repo.

PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS runs (
  run_id                 TEXT PRIMARY KEY,
  ts                     TEXT NOT NULL,          -- ISO-8601 UTC when run started
  task_id                TEXT NOT NULL,
  arm                    TEXT NOT NULL,          -- sail | react-tw
  model                  TEXT NOT NULL,
  toolchain              TEXT NOT NULL,          -- {git_sha}{-dirty}-r{cargo_version}
  profile_hash           TEXT,                   -- sha256 of resolved profile jsonc
  harness_ver            TEXT NOT NULL,
  wall_ms                INTEGER,
  turns                  INTEGER,
  tokens_in              INTEGER,                -- NULL when unavailable (must flag)
  tokens_out             INTEGER,                -- NULL when unavailable (must flag)
  compile_ok             INTEGER,                -- 0/1/NULL (NULL = not applicable e.g. react)
  iterations_to_clean    INTEGER,                -- NULL if never clean
  vocab_share            REAL,                   -- NULL if N/A
  layout_findings_json   TEXT,                   -- JSON array
  failure_classes_json   TEXT,                   -- JSON array of taxonomy tags
  beauty_json            TEXT,                   -- per-criterion scores + grader; NULL until human-graded
  run_dir                TEXT NOT NULL            -- absolute path to sandbox/artifacts
);

CREATE TABLE IF NOT EXISTS run_events (
  run_id     TEXT NOT NULL,
  seq        INTEGER NOT NULL,
  event_json TEXT NOT NULL,
  PRIMARY KEY (run_id, seq),
  FOREIGN KEY (run_id) REFERENCES runs(run_id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_runs_task_arm ON runs(task_id, arm);
CREATE INDEX IF NOT EXISTS idx_runs_ts ON runs(ts);
CREATE INDEX IF NOT EXISTS idx_runs_toolchain ON runs(toolchain);
