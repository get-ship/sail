# sui repo — phase-2 extraction notes

This repo is the deck + sui code carved out of the ship monorepo. Phase 2 made
it build standalone: it now has its own Cargo + bun workspace manifests, with a
small set of **cross-repo edges** pointing back at the ship repo for the few
things that stayed there. Those absolute paths are TEMP — they relativize to
`../ship/...` after the layout reshuffle (each is tagged with a
`// TEMP: ... becomes ../ship/...` comment in the Rust/Cargo sources; JSON can't
carry comments, so the bun edges are documented here).

Ship repo (the source / cross-repo target):
`/home/madvib/dev/ship-worktrees/sui-framework`

## Status: GREEN

- `cargo build` (default-members) — green
- `cargo build --workspace` (includes `deck-ssr`, which links V8 via `ship-js`) — green
- `bun install` — clean, idempotent

Bootstrap order on a fresh checkout (the `deck-core` build.rs requires baked
framework bundles to exist before it compiles):

```
bun install
cargo run -p deck-cli -- build-assets   # bakes .deck/framework/*.js + stages into deck-core/assets/
cargo build
```

`build-assets` produces the 4 framework singletons (`sui-runtime.js`,
`sui-dom.js`, `sui-sdk-client.js`, `deck-route.js`) into `.deck/framework/`;
`deck-core/build.rs` stages them into `crates/deck/deck-core/assets/` for
`include_str!`. Both locations are gitignored (regenerated, never committed) —
the committed assets are `lucide-icons.json`, `live-reload.js`, and the HTML/CSS
templates.

## Cargo workspace

Root `Cargo.toml` ported from ship's root (resolver 2, profiles, the lint
posture, and the subset of `[workspace.dependencies]` the carved crates use).
`rust-toolchain.toml` and `.cargo/config.toml` copied from ship (the `xtask`
alias was dropped — no xtask crate here). `Cargo.lock` copied from ship so
transitive versions match what ship builds with (critical — `rolldown` resolves
to `1.1.x` without the lock, whose renamed `code_splitting` field breaks
`deck-bundler`; ship pins `1.0.3`).

### members (build via `cargo build --workspace`)

- `crates/sui/compiler`
- `crates/sui/compiler-napi`
- `crates/sui/lint`
- `crates/sui/lsp`
- `crates/sui/signals`
- `crates/deck/deck-config`
- `crates/deck/deck-host`
- `crates/deck/deck-bundler`
- `crates/deck/deck-cli`
- `crates/deck/deck-core`
- `crates/deck/deck-axum`
- `crates/deck/deck`
- `crates/deck/deck-conformance`
- `crates/deck/deck-ssr` — **members-only**, excluded from `default-members`
- `packages/sui/grammar` (the `tree-sitter-sui` Rust crate)

### default-members (build via bare `cargo build`)

Same as members **minus `deck-ssr`**. `deck-ssr` links V8 (via `ship-js`); its
`cargo test` fails to link as `-shared` (the known V8/deno
`R_X86_64_TPOFF32` quirk — ship excludes `ship-host` for the same reason). The
**lib builds fine** (`cargo build -p deck-ssr` is green); it's just kept out of
the default set so a bare `cargo build` stays light. `cargo test -p deck-ssr`
test-link failure is expected, not a regression.

### excluded

- `.artifacts` — build scratch
- `crates/sui/compiler/fuzz` — declares its own `[workspace]` (cargo-fuzz drives
  it directly under nightly)

## bun workspace

Root `package.json` is a bun workspace over the carved JS package trees. Globs:
`packages/components/sui/*`, `packages/deck/*`, and the four real JS packages
under `packages/sui/` listed **explicitly** (`compiler-wasm`, `dom`, `runtime`,
`tokens`).

`packages/sui/grammar` (`tree-sitter-sui`) is **deliberately excluded** from the
bun workspace: its `binding.gyp` makes bun auto-run a `node-gyp` native build on
install, which fails in environments without `node-gyp`. Nothing in the carved
JS packages imports `tree-sitter-sui`, and the grammar's Rust crate builds via
cargo independently, so excluding it from bun is correct, not a workaround. The
tree-sitter node binding / CLI (`tree-sitter generate`) is a dev-tooling tier
unrelated to the workspace build.

## Cross-repo edges (the ship-edges — TEMP absolute paths → `../ship/...` later)

### Cargo (Rust)

1. **`sui-compiler` → `build-info`** — `build-info` is a tiny leaf crate that
   stayed in ship (not carved). Rewired in `crates/sui/compiler/Cargo.toml` in
   two places (the `[build-dependencies]` and `[dependencies.build-info]`):
   `path = "/home/madvib/dev/ship-worktrees/sui-framework/crates/build-info"`

2. **`deck-ssr` → `ship-js`** — the L0 deno_core/V8 isolate, stayed in ship.
   Rewired in `crates/deck/deck-ssr/Cargo.toml`:
   `ship-js = { path = "/home/madvib/dev/ship-worktrees/sui-framework/crates/core/ship-substrate/crates/ship-js", features = ["deno"] }`

3. **`deck-bundler` `sui-sdk-client` framework target → ship `@ship/sdk-client`
   source.** The `sui-sdk-client` framework singleton is bundled from
   `packages/deck/sdk-client/src/index.ts`, which stayed in ship. Rewired the
   hardcoded entry in `crates/deck/deck-bundler/src/targets.rs`
   (`framework_singletons()`) from a `repo_root.join(...)` relative path to the
   ship absolute path. Without this, `cargo run -p deck-cli -- build-assets`
   fails with `UnresolvedEntry` and `deck-core/build.rs` can't bake.

### bun (JS) — documented here (JSON can't carry the TEMP comment)

4. **`@ship/sdk-client` → ship** (`packages/components/sui/primitives`):
   `"@ship/sdk-client": "file:/home/madvib/dev/ship-worktrees/sui-framework/packages/deck/sdk-client"`

5. **`@ship/test-setup` → ship** (7 consumers: `components/sui/{charts,
   chat-elements,media,primitives,viewer}`, `sui/dom`, `sui/runtime`):
   `"@ship/test-setup": "file:/home/madvib/dev/ship-worktrees/sui-framework/packages/test-setup"`

6. **`@ship/sdk` override → ship** (root `package.json` `overrides`). `@ship/sdk`
   is a *transitive* `workspace:*` dep of the linked `@ship/sdk-client`; bun
   re-resolves the linked package's `workspace:*` deps against this workspace
   (where `@ship/sdk` doesn't exist), so it's pinned via an override:
   `"@ship/sdk": "file:/home/madvib/dev/ship-worktrees/sui-framework/packages/deck/sdk"`

Note: the task spec suggested bun `link:` for these, but bun's `link:` protocol
requires a globally-registered link (`bun link`), not an absolute filesystem
path. Bun's path-dependency protocol is `file:` (the carved repo already used
`file:../..` in `packages/sui/runtime/example`), so all bun edges use `file:`.

## Nothing else blocked

Every carved crate/package resolves against either the in-repo workspace or one
of the documented ship-edges (`build-info`, `ship-js`, `@ship/sdk-client` /
`@ship/sdk`, `@ship/test-setup`). No carved crate needed a ship crate outside
that set, so no further code was carved out of ship.
