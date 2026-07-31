# sail

Ahead-of-time-compiled UI framework (`.sail` → ESM) and **deck** host
(routing, SSR, bundling, dev loop). Product docs: **`apps/sail-site`**.

## Prerequisites

- [Bun](https://bun.sh)
- Rust (pinned by `rust-toolchain.toml`; include `wasm32-unknown-unknown`)
- `wasm-pack` only if you rebuild the browser compiler bundle

## Build

```sh
bun install
cargo run -p deck-cli -- build-assets    # bake framework bundles → .deck/framework/
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
Do not commit bake outputs under `.deck/` or `public/sail_compiler_bg.wasm` /
`public/tut-preview-theme.css`.

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
| `crates/sail/` | Compiler, lint, LSP, layout, ops, MCP |
| `crates/deck/` | Host crates + drift-gated `FEATURES.md` |
| `packages/sail/` | Runtime, DOM, tokens, grammar, tutorial |
| `packages/deck/` | Router, host, e2e, docs app |
| `packages/components/sail/` | Component packs |
| `apps/sail-site` | Authored product documentation + catalog |
| `tests/` | Integration, parity, perf baselines |

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
| **User toolchain** | `sail` (from `deck-cli`) | `dev`, `build`, `check`, `lsp`, `mcp`, … |
| **Repo automation** | `cargo xtask` | Cross-language codegen (today: `gen-icons`) |

No third drawer: ad-hoc one-shots do not land under `scripts/`.

### Comment hygiene

Comments carry contracts and **why** — not narration, dates/names, session
monologue, or process ids (`job/…`, `task #N`, mission paths). Full rule and
the `.agents/` release boundary: [docs/INDEX.md](./docs/INDEX.md).

### Entry points (prefer these)

| Command | Class | What it runs |
|---------|-------|--------------|
| `bun install` → `cargo run -p deck-cli -- build-assets` → `cargo build` | Law | Framework bake + default Rust members |
| `cd apps/sail-site && bun run gen` | Law | wasm · prop-registry · grammar · runtime surface · bake · tutorial · tut-preview CSS |
| `cd apps/sail-site && bun run dev` / `build` | Law | `gen` then deck CLI |
| `bun run test` | Law | vitest + `tests/hygiene` repo guards |
| `bun run typecheck` | Law | `scripts/typecheck-components.mjs` (also runs `gen-component-types`) |
| `cargo install --path crates/deck/deck-cli --bin sail` | Product tool | Public CLI (`dev`, `build`, `check`, `lsp`, `mcp`, …) |
| `cargo xtask gen-icons` | Product tool | Bake lucide SVGs → `crates/deck/deck-core/assets/lucide-icons.json` (after `lucide-static` bump) |
| `cd apps/sail-site && bun run qa` / `qa:fast` / `qa:probe` | Recipe | Layout probe (`qa/probe.mjs`); needs dev server |
| `cd apps/sail-site && bun run qa:l1` | Recipe | L1 conformance (`packages/sail/l1-conformance/run.mjs`); own harness, not under `qa/` |

### `scripts/` (repo root — curated; ≤2 files)

| Path | Class | Notes |
|------|-------|-------|
| `scripts/typecheck-components.mjs` | Law | Root `typecheck`; regenerates types then `tsc --noEmit` per component pack |
| `scripts/gen-component-types.mjs` | Law | Emits adjacent `.sail.d.ts` companions; invoked by typecheck |

Repo sanitization / boundary guards live under `tests/hygiene/` (`bun run test:hygiene`).

### `apps/sail-site/scripts/`

| Path | Class | Notes |
|------|-------|-------|
| `gen-runtime-surface.mjs` | Law | Tier-B runtime surface → `src/runtime-surface.gen.ts` (`bun run gen`) |
| `gen-tutorial.mjs` | Law | Tutorial prose/lessons → `src/tutorial-data.ts` |
| `gen-tut-preview-theme.mjs` | Law | Framework CSS concat + font URL rewrite → `public/tut-preview-theme.css` |
| `gen-tut-preview-theme.test.mjs` | Recipe | `bun:test` unit tests for font rewrite (not on root vitest) |
| `theme-preview-proof.mjs` | Recipe | Headless proof of `theme.preview` token hatch |

### `apps/sail-site/qa/`

| Path | Class | Notes |
|------|-------|-------|
| `probe.mjs` | Recipe | Deterministic layout/contrast probe; `bun run qa` |
| `behavior.mjs` | Recipe | Prop / overlay / surface-consistency harness (manual) |

One-off screenshot helpers (`snap-cell`, `snap-hero`, `inspect-catalog`) and the thinner `sweep.mjs` (superseded by `probe --sweep`) were removed.

### `xtask` (repo automation)

| Path | Class | Notes |
|------|-------|-------|
| `xtask` (`gen-icons`) | Product tool | Cross-language icon bake; assets are committed — not part of every cold-start `gen` |
