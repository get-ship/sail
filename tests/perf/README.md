# SAIL perf baselines

Checked-in js-framework-benchmark medians for the owned `keyed/sail` app.

| Path | Role |
|------|------|
| `frameworks/keyed/sail/` | **Owned** app — edit only here |
| `$BENCH_HOME/frameworks/keyed/sail` | Sync output (default `~/dev/js-framework-benchmark`) |
| `baselines/` | Capture JSON (`YYYY-MM-DD-*.json`) |

```sh
bun tests/perf/scripts/sync-keyed-sail.mjs
# BENCH_HOME=/path/to/js-framework-benchmark bun tests/perf/scripts/sync-keyed-sail.mjs
# bun tests/perf/scripts/sync-keyed-sail.mjs --dry-run
```

No in-repo bench driver; run upstream js-framework-benchmark tooling after sync.
