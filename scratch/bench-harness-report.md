# bench-harness report

**Branch:** `job/bench-harness`  
**Worktree:** `/home/madvib/dev/Ship/worktrees/bench-harness`  
**Base tip:** `8035efd3`  
**Commit:** `94b0c2b4`

## Goal

One owned keyed app in-tree; `BENCH_HOME` is sync target only; README tells the truth. No fake `just bench` gate.

## Done

### 1. Truth in docs

- Rewrote `tests/perf/README.md`:
  - States driver scripts (`just bench`, `bench-lib.mjs`, `perf-profile.mjs`) are **absent**
  - Documents SoT: `tests/perf/frameworks/keyed/sail` only
  - External `$BENCH_HOME/frameworks/keyed/sail` = sync output
  - Stale `keyed/sui` removed by sync
  - Lexically latest baseline = `2026-06-26-reconstituted.json`; notes `2026-05-14-current.json` still architecturally stale
  - Optional precompile smoke command documented
- Rewrote `tests/perf/frameworks/keyed/sail/README.md` to match (sync script, no fake just recipes)

### 2. Sync script

- Added `tests/perf/scripts/sync-keyed-sail.mjs`
  - `BENCH_HOME` default `~/dev/js-framework-benchmark`
  - Copies owned authored set → `$BENCH_HOME/frameworks/keyed/sail`
  - Deletes `$BENCH_HOME/frameworks/keyed/sui` if present
  - `--dry-run` / `DRY_RUN=1` supported
- **Verified:** dry-run listed 8 files + stale sui remove; live run deleted keyed/sui and created keyed/sail under the existing checkout

### 3. Harness identity

- Top-of-file comment on `src/sail-keyed.sail` (foreign chrome / not product exemplar / benchDanger contract)
- `sail.config.json` with existing recognized key only: `{ "stylesheets": [] }`  
  - No invented `"harness"` key (unknown keys fail closed as Rejected)
  - Floor still bans product `class=`; Bootstrap class strings in this file remain foreign driver chrome documented as such — no language/config lift
- `precompile.mjs` ownership comments + discovery paths updated (sail probes + in-tree worktree root; no longer names missing bench-lib)

### 4. Optional smoke

- Documented:
  ```sh
  cargo build -p sail-compiler
  SAIL_REPO="$PWD" bun tests/perf/frameworks/keyed/sail/scripts/precompile.mjs
  ```
- Not run here: no `target/{debug,release}/sail-compiler` binary in this worktree at job time. Sync path was exercised instead.

## Not done (by design)

- No language changes (`class:`, `prop:`, interop)
- No fake green CI bench gate
- No push/land
- No multi-hour webdriver bench

## Acceptance check

| Gate | Status |
|------|--------|
| Script exists and is runnable | yes — dry-run + live against real BENCH_HOME |
| README does not claim missing driver works | yes |
| Single SoT path documented | `tests/perf/frameworks/keyed/sail` |
| Commit on branch | yes (this job) |
