# SAIL perf baselines

Two suites share this tree: runtime keyed-bench medians, and compiler
phase profiles.

| Path | Role |
|------|------|
| `frameworks/keyed/sail/` | **Owned** keyed-bench app — edit only here |
| `$BENCH_HOME/frameworks/keyed/sail` | Sync output (default `~/dev/js-framework-benchmark`) |
| `baselines/YYYY-MM-DD-*.json` | Runtime keyed-bench captures |
| `baselines/YYYY-MM-DD-compiler.json` | Compiler phase / RAM / HMR profile |

```sh
# Runtime keyed bench (sync only; run upstream after)
bun tests/perf/scripts/sync-keyed-sail.mjs
# BENCH_HOME=/path/to/js-framework-benchmark bun tests/perf/scripts/sync-keyed-sail.mjs
# bun tests/perf/scripts/sync-keyed-sail.mjs --dry-run

# Compiler profile (continue-on-error corpus; no per-file wall)
bun tests/perf/scripts/profile-compiler.mjs
bun tests/perf/scripts/profile-compiler.mjs --skip-dev-rss

# sail-dev + Studio-tab RSS (PERF-7; after a successful boot, not cargo regen)
bun tests/perf/scripts/profile-sail-dev-ram.mjs
bun tests/perf/scripts/profile-compiler.mjs --sail-dev-ram
```

The compiler driver is `sail-compiler profile`. It times product
`compile()` per file, then type-gate / layout / token / HMR anatomy.
It does not fail-fast: red files are recorded and the walk continues.

The RAM driver boots `sail dev --project apps/studio`, waits until
HTTP serves, then samples `/proc/<pid>/status` for the daemon +
children and (if Studio serves) the Playwright Chromium renderer.
It then applies 10 HMR edits to a tiny Studio route and samples
again. Cargo-regen peaks are not recorded as steady state.
