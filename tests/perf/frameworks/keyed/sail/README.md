# keyed/sui — js-framework-benchmark app (vendored)

This is the [js-framework-benchmark](https://github.com/krausest/js-framework-benchmark)
`keyed/sui` implementation. It is **owned here**, not in a fork of the upstream
benchmark repo — we maintain only these source files, never the whole harness.

The benchmark runner expects the app to live at `$BENCH_HOME/frameworks/keyed/sui`
inside an external js-framework-benchmark checkout. `scripts/dev/bench-lib.mjs`
copies the files below into that location (`syncSuiFramework()`) before every
build, so this directory is the single source of truth. Edit here; the checkout
copy is overwritten on the next `just bench` / `just bench-check`.

## Synced files (the authored set)

```
package.json              frameworkVersion 0.1.0 + esbuild build scripts
package-lock.json         minimal npm lock (the runner's isFrameworkDir needs it)
index.html                <sail-keyed> mount + dist/main.js
src/main.ts               side-effect import of the compiled component
src/sail-keyed.sui         the keyed table template
src/sail-keyed.controller.ts   row model + run/add/update/clear/swap/select/remove
scripts/precompile.mjs    compiles .sui → dist/, links @ship/sail-runtime
```

`node_modules/`, `dist/`, and `bun.lock` are **not** vendored — they're build
state, regenerated in the checkout.

## Resolving the sui toolchain

`precompile.mjs` needs the sui compiler + runtime. It resolves a single sui
checkout root, in order:

1. `SAIL_REPO` env — explicit checkout root (the cutover-safe knob).
2. `SAIL_COMPILER` / `SAIL_RUNTIME` envs — granular overrides.
3. Discovery — probes `~/dev/sui`, then `~/dev/Ship/sui`, for one that contains
   `packages/sail/runtime`.

If the sui checkout moves, set `SAIL_REPO=/new/path` (or rely on discovery if it
lands in a probed location). No absolute paths are baked in.

## First run in a fresh checkout

The synced dir needs its dev deps once (esbuild):

```sh
cd "$BENCH_HOME/frameworks/keyed/sail" && bun install
```

After that, `just bench` / `just bench-check` from the Ship repo drive everything.
