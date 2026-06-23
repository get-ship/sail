# deck-bundler — In-Process Rolldown Bundler

`crates/deck/deck-bundler/` owns the asset bundler for Ship's daemon. The
bundler runs **in-process inside `shipd`** (Rolldown, compiled into the binary)
with no child processes and no external worker. The only runtime dependency
outside this crate is the `sui-compiler` binary, which the daemon installs under
`~/.ship/runtime/` and spawns per-file during bundling.

> History (#163, #167): this used to be a `bun`-driven `bundler.ts` worker the
> daemon spawned over a JSON-line stdin/stdout protocol. That worker is
> **retired** — the daemon now bundles in-process via Rolldown. No child process,
> no protocol, no bun in the deployment bundle.

See the decision record:
[`docs/ship-adrs/references/docs/in-process-rolldown-bundler.md`](../../docs/ship-adrs/references/docs/in-process-rolldown-bundler.md)

---

## Runtime directory — `~/.ship/runtime/`

The runtime dir holds exactly one thing: the `sui-compiler` binary the bundler
spawns to compile `.sui` files. It mirrors what `rustup` does for Rust toolchains:
one well-known path, pinned to the Ship version that installed it.

```
~/.ship/runtime/
  sui-compiler        # Rust release build of crates/sui/compiler
```

The directory is owned by
`crates/core/ship-substrate/src/home/ship_home.rs` — every accessor
(`runtime_dir`, `runtime_sui_compiler`) lives there; no other code constructs
paths under the runtime dir directly.

---

## How bundling works

The `Bundler` is in-process:

- **Engine:** Rolldown (oxc-based, Rust-native), embedded in `ship-platform`.
  Requires the pinned toolchain to be Rust **1.95+** (oxc uses `if let` guards,
  stabilised in 1.95) — see `rust-toolchain.toml`.
- **SUI plugin** (`sui_plugin.rs`): a native Rust Rolldown `Plugin`.
  `resolve_id` redirects `./X.compiled.js` siblings, externalises the framework
  singletons (`@ship/sui-runtime`/`-dom`/`sdk-client`), and re-anchors
  `@ship/<pkg>` to workspace `src/`. `load` spawns `sui-compiler` to compile
  `.sui` → JS. Rolldown handles TS transpile + node resolution natively.
- **Output:** `assets_dir_for` — `crates/deck/deck-core/assets` for the baked
  framework bundles, `.deck/` for views/components/spaces. No source-tree writes.
- **API** (`lib.rs`): `Bundler::start` → `build(target)` / `build_all`
  (+ catalog aggregation) / `start_watching` / `subscribe_events` / `shutdown`,
  all async, in-process.

---

## Modules

| File | Owns |
|---|---|
| `src/lib.rs` | Public `Bundler` struct, `BundlerEvent` type |
| `src/lib.rs` | `Bundler::start`, `build`, `build_all`, `start_watching`, `subscribe_events`, `shutdown` |
| `src/sui_plugin.rs` | `SuiPlugin` — the Rolldown plugin; `resolve_id` + `load` hooks |
| `src/rolldown_bundle.rs` | `bundle_target` — drives a Rolldown build with `SuiPlugin` installed |
| `src/targets.rs` | Bundle-target definitions (views, packs, framework assets, apps) |
| `src/routes.rs` | Route-tree walk → `routes.gen.ts` codegen (typed route builders for the shell) |
| `src/watch.rs` | `notify` file watcher; emits `BundlerEvent::ModuleChanged` / `CompileError` on the broadcast channel |

---

## HMR

`start_watching` runs a `notify` watcher over the source dirs; on a `.sui` /
`.ts` / `.js` change it re-bundles the affected targets and emits a
`BundlerEvent` (`ModuleChanged` / `CompileError`) on a broadcast channel. The
daemon (`apps/shipd/src/lib.rs`) forwards these to `views::sui_hmr` →
`/_deck/dev/hmr` SSE → the `live-reload.js` client baked into
`crates/deck/deck-core/assets`. All Rust, no bun, no vite.

(The shell SPA's *dev* server uses vite for its own HMR — `apps/shell` `bun run
dev`. That's dev-only; the prod shell ships as a pre-built static bundle. Vite
never ships in the daemon or the shell bundle.)

---

## CLI

```sh
ship runtime install   # populate ~/.ship/runtime/ (sui-compiler) from local sources
ship runtime update    # wipe + reinstall
ship runtime doctor    # verify sui-compiler is present + executable
```

`runtime install` resolves `sui-compiler` from `--sui-compiler <path>` or a
walk-up search for `target/{release,debug}/sui-compiler` from cwd. `update`
wipes the runtime dir then reinstalls.

---

## Errors

Every `ship_platform::toolchain` error carries a recovery hint in `Display`:

- **"Ship toolchain not installed at …"** → `ship runtime install`
- **"Ship toolchain is incomplete: missing sui-compiler"** → `ship runtime install`
- **"… is not executable"** → `ship runtime install` (resets +x)

---

## bun — dev/build-time only, never shipped

bun is **dev/build-time tooling only**, never in the deploy bundle:
- The `apps/shell` dev server is vite (dev-only); `.sui` compilation is in-process
  via this crate (`sui-build` was retired in #220). The lucide icon set is
  committed source at `crates/deck/deck-core/assets/lucide-icons.json` and
  regenerated only on a lucide-static version bump via
  `cargo run -p xtask -- gen-icons` (#172).

The `apps/shell` / `apps/spaces` SPA prod build is **not** bun: `deck build`
(`crates/deck/deck-cli`, #170) bundles the app via Rolldown, compiling each `.sui`
in-process — no separate pre-compile step. One engine for framework + views + apps.

The framework-asset bake is **not** bun: `cargo xtask build-assets` drives the
same in-process Rolldown `Bundler::build_all`, writing the framework bundles to
`crates/deck/deck-core/assets`. `just build` step [6/9] and `npm run
build:bundles` both call it. There is one bundle-target definition
(`src/targets.rs`) and one engine for both the daemon and the bake.

---

## See also

- `crates/core/ship-platform/src/toolchain/` — runtime-dir install/ensure.
- `docs/ship-adrs/references/docs/in-process-rolldown-bundler.md` — the ADR
  (decision + alternatives + consequences).
- `docs/ship-adrs/references/docs/unified-deck-router.md` — the router ADR
  (route codegen lives in `src/routes.rs`).
