# sui

An ahead-of-time-compiled UI framework. `.sui` templates compile to plain ESM
bound to a fine-grained signal runtime — real custom elements, light DOM by
default (shadow only as a per-component containment feature for foreign
content), server-rendered and hydrated. **deck** is the meta-framework that
hosts them: filesystem routing, SSR, bundling, the dev loop.

```
SUI : deck  ::  React : Next
```

Three layers, dependencies pointing one way (deck → sui, never back):

- **SUI** — the `.sui` language: AOT compiler (Rust, WASM-safe), lint rules,
  LSP, tree-sitter grammar, and the JS runtime/DOM packages compiled output
  binds to. A template *declares*; its inline `<controller>` block *does*
  (state, lifecycle, I/O).
- **deck** — the host: file-routed pages (`routes/<seg>/page.sui`, `[param]`
  dynamic segments), SSR + hydrate, an in-process Rolldown bundler that
  compiles `.sui` via the linked compiler, HMR with state preservation, static
  builds, and a `DataPort` seam so a host app supplies live data without deck
  knowing the app.
- **Design system** — the OKLCH token substrate (`@ship/tokens`, ~250 tokens,
  LSP-completed) and `<sail-*>` component packs built on it.

## Place in the Ship project

sui is developed alongside an internal counterpart at the author's org — a
polyrepo where sui sits as one repo among private siblings (the core runtime,
internal apps, marketing site, and shared infrastructure crates). That
internal layout isn't required to build or use this repo; it's mentioned here
only because a couple of things below are a direct consequence of it:

- The core runtime repo consumes sui via Cargo path deps; its daemon mounts
  `deck::router()` and implements the `DataPort` seam.
- `deck-ssr` (the Deno/V8-based production SSR path) depends on a private
  sibling crate — the org's shared `deno_core`/V8 isolate — via a
  machine-specific Cargo path dependency. That's a pending distribution
  decision, not yet resolved for outside consumers; see Build, below.
- A couple of internal apps consume the published `@ship/*` packages.

Honest caveat: `@ship/*` packages are not yet published to public npm — the
`.npmrc` scope override that used to route them to an internal registry has
been removed, but the packages themselves aren't live on the public registry
yet. Consuming them from outside this repo means building from source here
in the meantime. Everything in this repo other than `deck-ssr` resolves from
the bun workspace / crates.io with no external sibling checkout required.

## Layout

| Path | What it is |
|---|---|
| `crates/sail/` | The language: `compiler` (`.sui` → ESM, pure, WASM-safe), `compiler-napi` (Node binding), `lint`, `lsp`. |
| `crates/deck/` | The host: `deck-core` (transport-agnostic serving), `deck-axum` (axum adapter), `deck-host`, `deck-bundler` (in-process Rolldown), `deck-config` (`deck.jsonc`), `deck-cli` (the `deck` binary), `deck` (facade), `deck-conformance` (harness), `deck-ssr` (TS-SSR through ship-js; members-only). |
| `crates/build-info` | Per-repo build-time identity (git sha/dirty/timestamp). |
| `packages/sail/` | `runtime` (signals + custom-element wrapper), `dom` (DOM interpreter + hydration), `tokens`, `compiler-wasm` (browser compiler), `grammar` (tree-sitter), `test-setup`, `tutorial` (lesson sources). |
| `packages/deck/` | `router`, `host` (SES-confined controller execution), `hono` (TS sibling of deck-axum), `lsp-browser`, `docs` (deck's own docs site — the canonical standalone example and release gate), `e2e` (Playwright against the real CLI), `eval`, `test-harness`. |
| `packages/components/sail/` | Component packs: `primitives`, `charts`, `media`, `viewer`, `canvas`, `chat-elements`, `code-editor`. |
| `apps/sail-site` | The SUI site: docs, tutorial, component catalog + interactive playground. A deck app dogfooding the framework. |
| `packages/deck/eval` | The SUI authoring eval (`@ship/sail-eval`) — lane agents build a view from a spec; compile + lint + structural rubric grade the output. |
| `tests/` | `integration` (vitest), `parity` (emit parity), `perf` (js-framework-benchmark baselines). |
| `docs/` | `ARCHITECTURE.md` (the governing language/userland map), `LANG-GAPS.md` (the migration ledger), generated reports. |
| `scripts/`, `xtask-deck/` | Repo tooling: typecheck orchestration, config lints, lucide icon bake. |

## Build

Prerequisites: [Bun](https://bun.sh), Rust (pinned by `rust-toolchain.toml`,
wasm32 target included). `wasm-pack` only for the browser compiler bundle.

```sh
bun install                              # JS workspace
cargo run -p deck-cli -- build-assets    # bake framework bundles → .deck/framework/
cargo build                              # default members
```

The order matters: `deck-core`'s build script `include_str!`s the framework
bundles (`sail-runtime.js`, `sail-dom.js`, …) and fails with instructions when
they haven't been baked. Baked output is gitignored, never committed.

A bare `cargo build` excludes `deck-ssr` (see `default-members` in the root
`Cargo.toml`). Honest caveat: this isn't just an opt-in feature flag —
`deck-ssr`'s V8 isolate dependency (`ship-js`) is currently a Cargo path
dependency pointed at a private sibling repo via a machine-specific absolute
path, so `cargo build -p deck-ssr` will not build for anyone outside the
author's machine today. Making `deck-ssr` buildable from a clean clone (a
published/pinned `ship-js`, or an alternative SSR path) is a later
distribution item, not yet done. Nothing else in this repo is affected:
`cargo build` / `cargo test` (default members) work from a clean clone with
no sibling checkout.

Run the SUI site:

```sh
cd apps/sail-site && bun run dev          # deck dev server on 127.0.0.1:7878
```

## Test

```sh
bun run test                             # vitest across all JS packages + repo gates
cargo test                               # Rust workspace (default members)
bun run typecheck                        # component packs
cd packages/deck/e2e && bun run test     # browser e2e: real deck CLI, headless Chromium
```

`tests/perf/` holds checked-in js-framework-benchmark baselines;
`crates/sail/compiler/fuzz` is a cargo-fuzz target (own workspace, nightly).

## The deck CLI

```sh
cargo run -p deck-cli -- dev --project apps/sail-site   # standalone dev + HMR
cargo run -p deck-cli -- build --project <app> --bundle-framework
cargo run -p deck-cli -- routes --root <app>           # routes/ → routes.gen.ts
cargo run -p deck-cli -- scaffold view <name>          # also: component, pack
cargo run -p deck-cli -- emit --target <name> --out <dir>
```

Standalone deck serves routing, assets, SSR and HMR without any daemon; live
`<data>` subscriptions are a host capability today — a host mounts
`deck::multiplex_router()` and injects its `DataPort`.

## Conventions

- `main` is guarded; work lands on `feat/next`. Commit types: `feat`, `fix`,
  `refactor`, `test`, `docs`, `chore`.
- Boundary and drift are test-enforced: the compiler emits structure, never
  serve URLs (`crates/sail/compiler/tests/boundary_invariants.rs`); deck names
  no app concept (`crates/deck/deck-core/tests/boundary.rs`); the feature
  inventory `crates/deck/FEATURES.md` is gated by
  `deck-core/tests/features_doc.rs` so it cannot silently rot.
- Lucide icons are committed source; regenerate after a `lucide-static` bump
  with `cargo run -p xtask-deck -- gen-icons`.

## Docs

- `.claude/skills/sail-authoring/SKILL.md` — the generated authoring surface
  (regenerate with `cargo run -p sail-compiler -- skill`; drift-gated by
  `crates/sail/compiler/tests/skill_examples_compile.rs`).
- `docs/ARCHITECTURE.md` — the governing language / stdlib / userland map.
- `crates/sail/README.md` — the SUI Rust side and the sui/deck split.
- `crates/deck/README.md` — deck architecture, modes, `DataPort`, mounting.
- `crates/deck/FEATURES.md` — the drift-gated feature inventory.
- `packages/deck/docs/` — deck's docs site, built and served by deck.
- `packages/deck/eval/` — the authoring eval rig (`@ship/sail-eval`).
