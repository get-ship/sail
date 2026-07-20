# SAIL perf baselines

Checked-in median results from the
[js-framework-benchmark](https://github.com/krausest/js-framework-benchmark)
suite, captured against the owned `keyed/sail` app at a known git SHA.

## Source of truth

| Path | Role |
|------|------|
| `tests/perf/frameworks/keyed/sail/` | **Owned** keyed app (edit only here) |
| `$BENCH_HOME/frameworks/keyed/sail` | **Sync output** — overwritten by the sync script; never hand-edit |
| `$BENCH_HOME/frameworks/keyed/sui` | **Stale** pre-rename leftover — the sync script deletes it |

Sync:

```sh
bun tests/perf/scripts/sync-keyed-sail.mjs
# or: BENCH_HOME=/path/to/js-framework-benchmark bun tests/perf/scripts/sync-keyed-sail.mjs
# dry-run: bun tests/perf/scripts/sync-keyed-sail.mjs --dry-run
```

Default `BENCH_HOME` is `~/dev/js-framework-benchmark`.

There is **no in-repo driver** today: no `just bench`, no `just bench-check`,
no `scripts/dev/bench-lib.mjs`, no `scripts/dev/perf-profile.mjs`, and no CI
bench gate. Capturing a new baseline means running js-framework-benchmark’s
own tooling against the synced `keyed/sail` tree after a manual or scripted
build there. Do not treat missing recipes as working.

## Baseline files

Files live under `tests/perf/baselines/`. Naming:

- `YYYY-MM-DD-baseline.json` — standard capture
- `YYYY-MM-DD-<label>.json` — deliberate inflection (optimization, methodology)

**Lexically latest** filename on disk (sorted) is currently
`2026-06-26-reconstituted.json`. Prefer that name when referring to “latest
file,” not a hard-coded older label.

`2026-05-14-current.json` (swap1k 7.2 ms, SHA `747a5f17`) is **architecturally
stale**: it was captured against the old cursor-based `bindFor` in the bundled
runtime, before the LIS-based `packages/sail/runtime/src/for.ts` replaced it.
Comparing a contemporary LIS runtime to that file reports a spurious
`05_swap1k` gap that is an architecture-accounting artifact, not a live code
regression. Re-capture before trusting any gate built on these numbers.

Other historical notes:

- `2026-05-13-post-d2-d4.json` — post D2 (`bindFor` scratch reuse) + D4
  (bind\* short-circuit); SHA `2a951ef6`
- `2026-05-13-baseline.json` — pre-perf-cycle reference (`56bea690`); do not
  overwrite

## Format

```json
{
  "captured": "YYYY-MM-DD",
  "git_sha": "<short SHA>",
  "framework_version": "0.1.0",
  "compared_against": ["solid", "svelte", "vue", "vanillajs"],
  "operations":   { "01_run1k":  { "sail": 59.8, "solid": 49.5 }, "...": "..." },
  "memory_mb":    { "22_run-memory":  { "sail": 3.4 }, "...": "..." },
  "size_kb":      { "41_size-uncompressed": { "sail": 18.3 }, "...": "..." },
  "first_paint_ms": { "43_first-paint": { "sail": 38.0 } }
}
```

Median values only — variance lives in the raw
`webdriver-ts/results/*.json` files inside the bench checkout.

## Optional compile smoke (in-tree)

Does **not** run the multi-hour webdriver suite. Verifies the owned app still
compiles:

```sh
# from repo root; build the compiler first if needed
cargo build -p sail-compiler
SAIL_REPO="$PWD" bun tests/perf/frameworks/keyed/sail/scripts/precompile.mjs
```

`precompile.mjs` resolves `SAIL_REPO` (or `SAIL_COMPILER` / `SAIL_RUNTIME`),
links `@ship/sail-runtime`, and emits under the app’s `src/dist/`.

## Capturing a baseline (external harness)

1. Sync: `bun tests/perf/scripts/sync-keyed-sail.mjs`
2. Once in the checkout: `cd "$BENCH_HOME/frameworks/keyed/sail" && bun install`
3. Build prod for sail (and any comparison frameworks you care about) with the
   benchmark repo’s usual `npm run build-prod` / framework scripts
4. Run js-framework-benchmark’s webdriver driver for the frameworks you want
5. Fold medians into a new `tests/perf/baselines/YYYY-MM-DD-….json` and commit

Requires a sail checkout the synced `precompile.mjs` can find (`SAIL_REPO` or
discovery), with `cargo build -p sail-compiler` already done.

## Profiles

`tests/perf/profiles/*.cpuprofile` are checked-in V8 CPU profiles from earlier
manual captures (open in Chrome DevTools → Performance → Load profile). There
is no in-tree profiler driver script at present.

## On the keyed app itself

See `tests/perf/frameworks/keyed/sail/README.md`. It is foreign
js-framework-benchmark chrome (Bootstrap class names in the template for the
shared CSS sheet, `use:benchDanger` for `tr.danger` selection) — **not** a Sail
product exemplar.
