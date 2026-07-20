# keyed/sail — js-framework-benchmark app (owned)

This is the [js-framework-benchmark](https://github.com/krausest/js-framework-benchmark)
`keyed/sail` implementation. It is **owned in this repo**, not in a fork of the
upstream benchmark. We maintain only these source files, never the whole harness.

**Single source of truth:** this directory.

The external checkout path `$BENCH_HOME/frameworks/keyed/sail` is a **sync
output**. Sync with:

```sh
bun tests/perf/scripts/sync-keyed-sail.mjs
```

(`BENCH_HOME` defaults to `~/dev/js-framework-benchmark`.) That script also
removes the stale `$BENCH_HOME/frameworks/keyed/sui` tree if present. Edit here;
the checkout copy is overwritten on the next sync. Do not hand-edit the
checkout copy.

There is no in-repo `just bench` / `bench-lib.mjs` driver today. After sync,
build and run with js-framework-benchmark’s own tooling.

## Authored set (synced)

```
package.json              frameworkVersion 0.1.0 + esbuild build scripts
package-lock.json         minimal lock (runner isFrameworkDir needs it)
index.html                <sail-keyed> mount + dist/main.js
sail.config.json          pack marker (stylesheets: []; not a product pack)
src/main.ts               side-effect import of the compiled component
src/sail-keyed.sail       keyed table + bench controller (foreign chrome)
scripts/precompile.mjs    compiles .sail → src/dist/, links @ship/sail-runtime
README.md                 this file
```

`node_modules/`, `dist/`, `src/dist/`, and `bun.lock` are **not** the owned
source set — build state, regenerated in the checkout (or locally for smoke).

## Harness identity (not a product exemplar)

- Top of `sail-keyed.sail` documents foreign js-framework-benchmark chrome.
- Bootstrap-looking static `class="…"` strings exist only so the benchmark’s
  shared `/css/currentStyle.css` lays out the page. Sail’s floor bans `class=`
  for product components; this file is benchmark driver chrome, not an app to
  copy. No config key lifts that floor — do not invent one.
- Row selection uses a local `use:benchDanger` Action that toggles `classList`
  for the driver contract `tbody>tr.danger` only.

## Resolving the sail toolchain

`precompile.mjs` needs the sail compiler + runtime. Resolution order:

1. `SAIL_REPO` env — explicit checkout root (cutover-safe knob).
2. `SAIL_COMPILER` / `SAIL_RUNTIME` envs — granular overrides.
3. Discovery — probes `~/dev/sail`, then `~/dev/Ship/sail`, for a tree that
   contains `packages/sail/runtime`.

Build the compiler first: `cargo build -p sail-compiler` under that checkout.

## Optional in-tree smoke

```sh
# from Ship / sail monorepo root
cargo build -p sail-compiler
SAIL_REPO="$PWD" bun tests/perf/frameworks/keyed/sail/scripts/precompile.mjs
```

## First run in a fresh BENCH_HOME checkout

```sh
bun tests/perf/scripts/sync-keyed-sail.mjs
cd "$BENCH_HOME/frameworks/keyed/sail" && bun install
# then the benchmark repo's build-prod / webdriver flow
```
