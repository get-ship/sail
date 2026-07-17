# deck-bundler — In-Process Rolldown Bundler

`crates/deck/deck-bundler/` owns the asset bundler for Ship's daemon. The
bundler runs **in-process inside `shipd`** (Rolldown, compiled into the binary)
with no child processes and no external worker. The only runtime dependency
outside this crate is the `sail-compiler` binary, which the daemon installs under
`~/.deck/runtime/` (deck's own namespace) and spawns per-file during bundling.

> History (#163, #167): this used to be a `bun`-driven `bundler.ts` worker the
> daemon spawned over a JSON-line stdin/stdout protocol. That worker is
> **retired** — the daemon now bundles in-process via Rolldown. No child process,
> no protocol, no bun in the deployment bundle.

See the decision record:
[`docs/ship-adrs/references/docs/in-process-rolldown-bundler.md`](../../docs/ship-adrs/references/docs/in-process-rolldown-bundler.md)

---

## Runtime directory — `~/.deck/runtime/`

The runtime dir holds exactly one thing: the `sail-compiler` binary the bundler
spawns to compile `.sail` files. It mirrors what `rustup` does for Rust toolchains:
one well-known path, pinned to the version that installed it. The dir lives under
deck's own `~/.deck/` namespace — deck names no host-private path.

```
~/.deck/runtime/
  sail-compiler        # Rust release build of crates/sail/compiler
```

The path accessor (`runtime_dir`, `runtime_sui_compiler`) is host-owned: in the
Ship deployment it lives in `ship-substrate`'s `home/ship_home.rs`, and no other
code constructs paths under the runtime dir directly. Renaming the accessor's
base from `~/.ship/runtime/` to `~/.deck/runtime/` is a host-side follow-up.

---

## How bundling works

The `Bundler` is in-process:

- **Engine:** Rolldown (oxc-based, Rust-native), embedded in `ship-platform`.
  Requires the pinned toolchain to be Rust **1.95+** (oxc uses `if let` guards,
  stabilised in 1.95) — see `rust-toolchain.toml`.
- **SAIL plugin** (`sail_plugin.rs`): a native Rust Rolldown `Plugin`.
  `resolve_id` redirects `./X.compiled.js` siblings, externalises the framework
  singletons (`@ship/sail-runtime`/`-dom`/`sdk-client`), and re-anchors
  `@ship/<pkg>` to workspace `src/`. `load` spawns `sail-compiler` to compile
  `.sail` → JS. Rolldown handles TS transpile + node resolution natively.
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

`start_watching` runs a `notify` watcher over the source dirs; on a `.sail` /
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
ship runtime install   # populate ~/.deck/runtime/ (sail-compiler) from local sources
ship runtime update    # wipe + reinstall
ship runtime doctor    # verify sail-compiler is present + executable
```

`runtime install` resolves `sail-compiler` from `--sail-compiler <path>` or a
walk-up search for `target/{release,debug}/sail-compiler` from cwd. `update`
wipes the runtime dir then reinstalls.

---

## Errors

Every `ship_platform::toolchain` error carries a recovery hint in `Display`:

- **"Ship toolchain not installed at …"** → `ship runtime install`
- **"Ship toolchain is incomplete: missing sail-compiler"** → `ship runtime install`
- **"… is not executable"** → `ship runtime install` (resets +x)

---

## bun — dev/build-time only, never shipped

bun is **dev/build-time tooling only**, never in the deploy bundle:
- The `apps/shell` dev server is vite (dev-only); `.sail` compilation is in-process
  via this crate (`sail-build` was retired in #220). The lucide icon set is
  committed source at `crates/deck/deck-core/assets/lucide-icons.json` and
  regenerated only on a lucide-static version bump via
  `cargo run -p xtask-deck -- gen-icons` (#172).

The `apps/shell` / `apps/spaces` SPA prod build is **not** bun: `deck build`
(`crates/deck/deck-cli`, #170) bundles the app via Rolldown, compiling each `.sail`
in-process — no separate pre-compile step. One engine for framework + views + apps.

The framework-asset bake is **not** bun: `cargo run -p deck-cli -- build-assets` drives the
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
