# SAIL keyed-bench fixture

`frameworks/keyed/sail/` is the owned js-framework-benchmark app. Sync it
upstream with `bun tests/perf/scripts/sync-keyed-sail.mjs`.

Compiler phase timing is `sail-compiler profile`. Write dumps under
`.artifacts/perf/` (gitignored). Do not commit JSON baselines.
