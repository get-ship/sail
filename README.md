# sail

Ahead-of-time-compiled UI framework (`.sail` → ESM) and **deck** host
(routing, SSR, bundling, dev loop). Product docs: **`apps/sail-site`**.

**Eval / agent lab** lives outside this product repo: `~/dev/sail-lab`
(adapters + standards) with grades/list/report through fleet
`bridge.v1.EvalService`. See `~/dev/sail-lab/fleet/README.md` and mission
scratch `SAIL-USE-EVALSERVICE.md`. Do not write run artifacts into this tree.

## Prerequisites

- [Bun](https://bun.sh)
- Rust (pinned by `rust-toolchain.toml`; include `wasm32-unknown-unknown`)
- `wasm-pack` only if you rebuild the browser compiler bundle

## Build

```sh
bun install
cargo run -p deck-cli --bin deck-bake -- build-assets    # bake framework bundles → .sail/framework/
cargo build                              # default members (excludes deck-ssr)
```

`deck-core` `include_str!`s baked framework bundles; run `build-assets` first.
`deck-ssr` is opt-in (`cargo build -p deck-ssr`) and pulls V8 via `ship-js`.

### Cold worktree → live sail-site

```sh
bun install
cd apps/sail-site && bun run gen         # wasm + framework bake + tutorial assets
bun run dev                              # default 127.0.0.1:7890 (SAIL_SITE_ADDR to override)
```

Do not invent `public/fonts` — faces are at `GET /_deck/fonts/{file}`.
Do not commit bake outputs under `.sail/` or `public/sail_compiler_bg.wasm`.

## Test

```sh
bun run test                             # vitest + repo gates
cargo test                               # Rust default members
bun run typecheck                        # component packs
cd packages/deck/e2e && bun run test     # real deck CLI, headless Chromium
```

## CLI

```sh
cargo install --path crates/deck/deck-cli --bin sail
# or
cargo run -p deck-cli --bin sail -- dev --project apps/sail-site
cargo run -p deck-cli --bin sail -- build --project <app> --bundle-framework
cargo run -p deck-cli --bin sail -- check path/to/file.sail
```

Peers on the same binary: `sail lsp`, `sail mcp`, `sail layout`, `sail tokens`,
`sail doctor`, `sail version`.

## Layout (pointers only)

| Path | Role |
|---|---|
| `crates/sail/` | Compiler, lint, LSP, layout, ops, MCP, render |
| `crates/deck/` | Host crates |
| `packages/sail/` | Runtime, DOM, tokens, grammar, tutorial, compiler-wasm, studio bridge/shell/client |
| `packages/deck/` | Router, host, hono, e2e, docs app, test-harness |
| `packages/components/sail/` | Component packs (L2 + chat-elements) |
| `apps/sail-site` | Authored product documentation + catalog |
| `apps/studio` | Dev overlay (`/_studio`): edit, checks, design, tasks |
| `skills/` | Agent skills (taste, output, motion-review, qa, design) |
| `tests/` | Integration, parity, perf baselines, hygiene gates |

Generated authoring skill: `cargo run -p sail-compiler -- skill` (do not hand-edit).

## Tooling

Script inventory for this monorepo. Classes:

| Class | Meaning |
|-------|---------|
| **Law** | Required for gen / build / test CI — break = red |
| **Product tool** | Documented user or eng command (`sail`, publish, migrate, icon bake) |
| **Recipe** | Optional local helper; can die without ceremony |

Cold start never requires an undocumented `node foo.mjs` — use the package.json / cargo entry points below.

### Two automation entry points

| Audience | Command | Owns |
|----------|---------|------|
| **User toolchain** | `sail` (from `deck-cli`) | `create`, `dev`, `build`, `check`, `types`, `lsp`, `mcp`, … |
| **This checkout (bake)** | `deck-bake` (same crate) | `build-assets`, `emit` |

No third drawer: ad-hoc one-shots do not land under `scripts/`. Icon packs are declared in `sail.config` (`icons.pack` + pinned version); used glyphs land in the app `dist/` graph.

### Comment hygiene

Comments carry contracts and **why** — not narration, dates/names, session
monologue, or process ids (`job/…`, `task #N`, mission paths). Enforced by
`tests/hygiene/` (sanitization scan + repo purity gate).

### Entry points (prefer these)

| Command | Class | What it runs |
|---------|-------|--------------|
| `bun install` → `cargo run -p deck-cli --bin deck-bake -- build-assets` → `cargo build` | Law | Framework bake + default Rust members |
| `cd apps/sail-site && bun run gen` | Law | wasm · prop-registry · grammar · runtime surface · bake · tutorial |
| `cd apps/sail-site && bun run dev` / `build` | Law | `gen` then deck CLI |
| `bun run test` | Law | vitest + `tests/hygiene` repo guards |
| `bun run typecheck` | Law | `scripts/typecheck-components.ts` (also runs `gen-component-types`) |
| `cargo install --path crates/deck/deck-cli --bin sail` | Product tool | Public CLI (`dev`, `build`, `check`, `lsp`, `mcp`, …) |
| `bun crates/sail/compiler/tests/l1-matrix/probe.ts` | Recipe | L1 × THEME_META matrix on the owned fixture (not sail-site) |

### `scripts/` (repo root — curated; ≤2 files)

| Path | Class | Notes |
|------|-------|-------|
| `scripts/typecheck-components.ts` | Law | Root `typecheck`; regenerates types then `tsc --noEmit` per component pack |
| `scripts/gen-component-types.ts` | Law | Emits adjacent `.sail.d.ts` companions; invoked by typecheck |

Repo sanitization / boundary guards live under `tests/hygiene/` (`bun run test:hygiene`).

### Hooks (local)

```
git config core.hooksPath hooks
```

Git runs `hooks/pre-commit` and `hooks/pre-push` from this tree. No copy into `.git/hooks`.

| hook | when | what |
|------|------|------|
| pre-commit | every commit | refuse `.artifacts/`; `rustfmt --check` on staged `.rs` only (`skip_children`); `sail check` on staged `.sail` |
| pre-commit | **merge commit** | plus `cargo clippy -p sail-compiler -p sail-lint` and `bun run typecheck` |
| pre-push | every push | `bun test tests/hygiene` (~2s) |

No cargo test, no Playwright, no corpus build. Skip with `--no-verify`.

### `apps/sail-site/scripts/`

| Path | Class | Notes |
|------|-------|-------|
| `gen-runtime-surface.ts` | Law | Tier-B runtime surface → `apps/sail-site/gen/runtime-surface.ts` (`bun run gen`) |
| `gen-tutorial.mjs` | Law | Tutorial prose/lessons → `src/compositions/tutorial-data.ts` |

### `apps/sail-site/qa/`

| Path | Class | Notes |
|------|-------|-------|
| `probe.mjs` | Recipe | Deterministic layout/contrast probe; `bun run qa` |

One-off screenshot helpers (`snap-cell`, `snap-hero`, `inspect-catalog`) and the thinner `sweep.mjs` (superseded by `probe --sweep`) were removed.

### Icons

Declare a pinned pack in `sail.config.jsonc` (`icons.pack`, optional `icons.version` / `icons.roles`). Author markup uses roles (`<icon name="settings" />`), never pack prefixes. The compiler collects used roles; Deck writes only those glyph bytes into `dist/`.
