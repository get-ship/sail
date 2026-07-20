# sail

An ahead-of-time-compiled UI framework. `.sail` templates compile to plain ESM
bound to a fine-grained signal runtime — real custom elements, light DOM by
default (shadow only as a per-component containment feature for foreign
content), server-rendered and hydrated. **deck** is the meta-framework that
hosts them: filesystem routing, SSR, bundling, the dev loop.

```
SAIL : deck  ::  React : Next
```

Three layers, dependencies pointing one way (deck → sail, never back):

- **SAIL** — the `.sail` language: AOT compiler (Rust, WASM-safe), lint rules,
  LSP, tree-sitter grammar, and the JS runtime/DOM packages compiled output
  binds to. A template *declares*; its inline `<controller>` block *does*
  (state, lifecycle, I/O).
- **deck** — the host: file-routed pages (`routes/<seg>/page.sail`, `[param]`
  dynamic segments), SSR + hydrate, an in-process Rolldown bundler that
  compiles `.sail` via the linked compiler, HMR with state preservation, static
  builds, and a `DataPort` seam so a host app supplies live data without deck
  knowing the app.
- **Design system** — the OKLCH token substrate (`@ship/tokens`, ~250 tokens,
  LSP-completed) and `<sail-*>` component packs built on it.

## Place in the Ship project

sail is developed alongside an internal counterpart at the author's org — a
polyrepo where sail sits as one repo among private siblings (the core runtime,
internal apps, marketing site, and shared infrastructure crates). That
internal layout isn't required to build or use this repo; it's mentioned here
only because a couple of things below are a direct consequence of it:

- The core runtime repo consumes sail via Cargo path deps; its daemon mounts
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
| `crates/sail/` | The language: `compiler` (`.sail` → ESM, pure, WASM-safe), `compiler-napi` (Node binding), `lint`, `lsp`. |
| `crates/deck/` | The host: `deck-core` (transport-agnostic serving), `deck-axum` (axum adapter), `deck-host`, `deck-bundler` (in-process Rolldown), `deck-config` (`deck.jsonc`), `deck-cli` (the `deck` binary), `deck` (facade), `deck-conformance` (harness), `deck-ssr` (TS-SSR through ship-js; members-only). |
| `crates/build-info` | Per-repo build-time identity (git sha/dirty/timestamp). |
| `packages/sail/` | `runtime` (signals + custom-element wrapper), `dom` (DOM interpreter + hydration), `tokens`, `compiler-wasm` (browser compiler), `grammar` (tree-sitter), `test-setup`, `tutorial` (lesson sources). |
| `packages/deck/` | `router`, `host` (SES-confined controller execution), `hono` (TS sibling of deck-axum), `lsp-browser`, `docs` (deck's own docs site — the canonical standalone example and release gate), `e2e` (Playwright against the real CLI), `eval`, `test-harness`. |
| `packages/components/sail/` | Component packs: `primitives`, `charts`, `media`, `viewer`, `canvas`, `chat-elements`, `code-editor`. |
| `apps/sail-site` | The SAIL site: docs, tutorial, component catalog + interactive playground. A deck app dogfooding the framework. |
| `packages/deck/eval` | The SAIL authoring eval (`@ship/sail-eval`) — lane agents build a route from a spec; compile + lint + structural rubric grade the output. |
| `tests/` | `integration` (vitest), `parity` (emit parity), `perf` (js-framework-benchmark baselines). |
| `docs/` | `ARCHITECTURE.md` (the governing language/userland map). Product docs live in `apps/sail-site`. |
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
`Cargo.toml`). `deck-ssr` pulls V8 via the `ship-js` crate from the ship-exec
git repo (opt-in: `cargo build -p deck-ssr`). Default members need no V8.
`ship-js` is intended for crates.io publication; the git dep is the interim.

### Cold worktree → live sail-site

No borrowed `.deck/`, no pre-baked `public/*.wasm`. From a clean checkout:

```sh
bun install
cd apps/sail-site && bun run gen         # wasm + framework bake + tutorial assets
# default binds 127.0.0.1:7890 (override with SAIL_SITE_ADDR=127.0.0.1:7892)
bun run dev
```

`gen` is the honest one-command bake: wasm-pack → prop-registry → grammar →
runtime surface → `deck build-assets` → tutorial data → tut-preview-theme
(font URLs rewritten to `/_deck/fonts/…`). `dev` re-runs `gen` then starts
`deck dev`. Playwright uses port **7891** and only reuses a server when
`SAIL_SITE_REUSE_SERVER=1`.

Do **not** invent `public/fonts` — framework faces are served at
`GET /_deck/fonts/{file}`. Do **not** commit bake outputs under `.deck/` or
`public/sail_compiler_bg.wasm` / `public/tut-preview-theme.css`.

Run the SAIL site (after gen, or via `bun run dev` which gens first):

```sh
cd apps/sail-site && bun run dev          # deck dev on SAIL_SITE_ADDR (default :7890)
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
`query()` subscriptions are a host capability today — a host mounts
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
- `crates/sail/README.md` — the SAIL Rust side and the sail/deck split.
- `crates/deck/README.md` — deck architecture, modes, `DataPort`, mounting.
- `crates/deck/FEATURES.md` — the drift-gated feature inventory.
- `packages/deck/docs/` — deck's docs site, built and served by deck.
- `packages/deck/eval/` — the authoring eval rig (`@ship/sail-eval`).
