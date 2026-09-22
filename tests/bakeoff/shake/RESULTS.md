# RESULTS.md — measured tree-shake baseline, compile-away fixtures

Date: 2026-08-23 · branch `job/bakeoff-compile-away` (pinned head `1991ce2b4`)
Harness: `tests/bakeoff/shake/bundle.ts` (bun + `bunx esbuild`)

## Methodology

- Inputs: compiled renders copied to `shake/render/{counter,list}.js`
  (from `/tmp/bakeoff-emit`, compiler 0.1.0 @ c19bbf6e) + hand-restored
  `.sail.controller.js` bodies matching the fixture `<controller>` sources
  byte-for-byte in logic (`controller()` + `signal()` from `/internal`).
  The `.ts` controller specifier was rewritten to the local `.js` twin —
  esbuild has no Sail TS plugin; the controller body is identical modulo two
  dropped type annotations.
- Bundling: single esbuild pass, `--bundle --minify --tree-shaking=true
  --format=esm --platform=browser --target=es2022`, with
  `--alias:@ship/sail-runtime=<worktree>/packages/sail/runtime/src` and
  `--alias:@ship/tokens=<worktree>/packages/sail/tokens/src` (mirrors the
  runtime package.json `"bun"` export condition; no node_modules needed,
  no deps added).
- Per-module surviving bytes: esbuild metafile `outputs[].inputs[].bytesInOutput`.
- Gzip: `Bun.gzipSync(..., { level: 9 })`.

### Denominator

`packages/sail/runtime/dist` does NOT exist in this worktree (never built), so
per instructions the denominator is **source**: all `packages/sail/runtime/src/**/*.ts`,
**601,337 bytes** across 107 files. All "% of runtime" figures below use this
src denominator and are therefore upper bounds relative to a real dist build.

## Headline numbers

| Fixture | Bundle raw | Bundle gzip | Runtime bytes shipped (raw) | % of bundle that is runtime | % of src-denominator runtime shipped |
|---|---|---|---|---|---|
| counter | **27,615** | **9,335** | **16,132** (23 modules) | 58.4% | **2.68%** |
| list    | **32,630** | **10,973** | **19,000** (25 modules) | 58.2% | **3.16%** |

The full runtime is ~601 KB of TS source; a hello-card ships ~2.7% of it.
The delta between fixtures is exactly the `{#for}` feature: `for.ts` (+2.7 KB)
and its `transition-runner.ts` edge (+54 B).

## Surviving-module ranking (bytesInOutput, minified)

### counter (27 modules total; runtime = 23)

| Class | Module (`packages/sail/runtime/src/…`) | Bytes |
|---|---|---|
| fixture | `counter.js` | 7,782 |
| runtime | `component.ts` | 4,202 |
| tokens | `themes.ts` | 2,776 |
| runtime | `signal.ts` | 2,512 |
| runtime | `component-factory.ts` | 2,301 |
| runtime | `effect-scope.ts` | 1,001 |
| runtime | `component-form.ts` | **950 — GAP** |
| tokens | `generated/themes-meta.ts` | **732 — GAP** |
| runtime | `slot-project.ts` | **702 — partial gap** |
| runtime | `styles/contracts-seam.ts` | **594 — partial gap** |
| runtime | `element-scope.ts` | 411 |
| runtime | `controller-resolver.ts` | 400 |
| runtime | `index.ts` | 349 (duplicate-load warn only) |
| runtime | `compose.ts` | 347 |
| runtime | `error-overlay.ts` | 335 |
| runtime | `host-api.ts` | 315 |
| runtime | `delegate.ts` | 283 |
| runtime | `navigate.ts` | 274 |
| runtime | `bind.ts` | 267 |
| runtime | `error-boundary.ts` | 217 |
| runtime | `component-decoders.ts` | 190 |
| runtime | `decode.ts` | 165 |
| runtime | `tpl.ts` | 156 |
| runtime | `hydrate-mark.ts` | 113 |
| fixture | `counter.sail.controller.js` | 65 |
| runtime | `host-route.ts` | 48 |
| runtime | `internal.ts` | 0 (pure re-export barrel) |

### list (29 modules total; runtime = 25)

Same table plus: `list.js` 9,806 · `for.ts` 2,736 · `transition-runner.ts` 54 ·
`list.sail.controller.js` 184. Everything else within ±2 B of counter.

## Tree-shake gaps found

Named gaps (modules retained although the fixtures never use their features):

1. **Theme catalog cascade — ~7.0 KB raw (~25% of counter's runtime payload),
   dead end-to-end.** Neither fixture mentions any theme symbol. But
   `internal.ts` opens with `export * from './index'`, and the root barrel
   re-exports `theme`, `bootTheme`, `applyTheme`, `THEMES`,
   `isValidTheme`, … That pulls `theme.ts`, `theme-session.ts`,
   `theme-catalog.ts` → `@ship/tokens/themes` (`themes.ts` 2,776 B +
   `generated/themes-meta.ts` 732 B). Verified dead by minified-bundle audit:
   the chain `ie(raw theme list) → at/K/U → ft → mt(buildThemes) → Le(THEMES) →
   Ro(THEME_BY_ID, 0 readers)` plus storage-key consts and
   `PREPAINT_THEME_SCRIPT` (1,046 B string alone) has **zero readers outside
   itself** in both bundles. Root cause: the ABI barrel star-re-exporting the
   author barrel defeats named-import shaking for everything index.ts touches
   that has module-level data.
2. **`component-form.ts` (950/951 B)** — retained because
   `component-factory.ts` calls `installFormHooks(host, internals, …)` inside
   an `if (def.formValue !== undefined || def.formDisabled !== undefined)`
   branch. The guard is runtime-data-dependent, so esbuild cannot prove it
   dead. Fix shape: split form hooks into a lazily-imported helper or gate on
   a static flag.
3. **`slot-project.ts` (702/703 B) + `styles/contracts-seam.ts` (594 B)** —
   same pattern: called unconditionally inside factory/compose mount paths
   (`projectSlots(host, captured)` in `compose.ts`; `ensureLightContracts()`
   in the light-DOM branch of the factory). Not statically removable today;
   they are small but appear in every component app regardless of slot/contract
   usage.
4. **Barrel residue:** `internal.ts` itself contributes 0 B (good — pure
   re-exports shake), but its `export * from './index'` keeps `index.ts`'s
   duplicate-load warning block alive (349 B). Harmless but nonzero.

What DID shake out cleanly (absent from both bundles despite being reachable
through the same barrels): `debounce`/`interval`/`persisted` (`timing.ts`,
`persisted.ts`), stores (`defineStore`/`useStore`/SSR seed helpers from
`store.ts` — value exports dropped even though `store.ts` is graph-reachable),
`object-url.ts`, `base64.ts`, `host-property.ts` values, query/virtual/motion
singletons, all of `/actions/*` (12 modules), morph/raw-html/if/when/slot
binders, `mount.ts`. Named per-feature imports do their job; the remaining cost
is concentrated in the four gaps above.

## Commands

```sh
cd ~/dev/Ship/worktrees/bakeoff-compile-away
bunx esbuild --version                       # 0.28.2 (via bunx cache; no package.json changes)
bun tests/bakeoff/shake/bundle.ts            # bundles BOTH fixtures
bun tests/bakeoff/shake/bundle.ts counter    # one at a time (--fixture-style arg)
```

Effective per-fixture command (as spawned by bundle.ts):

```sh
bunx esbuild tests/bakeoff/shake/render/<fixture>.js \
  --bundle --minify --tree-shaking=true --format=esm \
  --platform=browser --target=es2022 \
  --alias:@ship/sail-runtime=$PWD/packages/sail/runtime/src \
  --alias:@ship/tokens=$PWD/packages/sail/tokens/src \
  --outfile=tests/bakeoff/shake/dist/<fixture>/main.js \
  --metafile=tests/bakeoff/shake/dist/<fixture>/esbuild-meta.json
```

Outputs left in place: `dist/counter/{main.js,main.js.gz,esbuild-meta.json,summary.json}`
and `dist/list/{…}`.

## Blockers / deviations

- None blocking. Deviations: (a) controllers restored as `.js` twins (two type
  annotations dropped) because no TS-transforming bundler config exists here —
  logic identical; (b) denominator is `src/**/*.ts` (601,337 B) because
  `runtime/dist` was never built in this worktree; (c) `prop` resolves through
  the root barrel rather than a dedicated ABI file, which is what drags
  `component-factory.ts` (and through it gaps #2/#3) into every CE-based render.
