---
group: Views
order: 6
title: SUI — Tooling
description: The toolchain from .sail source to a live hot-reloaded view.
---

# SUI — Tooling

The chain from `.sail` source on disk to a live, hot-reloaded view in the browser passes through six tools. Each maps to one entry point.

## `sail-compiler` — the Rust ahead-of-time compiler

- **Library:** `crates/sail/compiler/src/lib.rs`. Pure transformation; no filesystem walk, no daemon, no network. The `compile(source, opts) -> CompileOutput` entry is documented in `references/4-layer-overview.md` § Layer 1.
- **Binary:** `crates/sail/compiler/src/bin/sail-compiler.rs` — the `sail-compiler` Cargo bin. One subcommand today: `build`.

```sh
# Compile a single file. Emits to the project mirror tree
# (<repo-root>/.ship/sail-out/<src-rel>/<stem>.compiled.js by default):
cargo run -q -p sail-compiler -- build views/home/page.sail

# Compile a directory recursively, into a single output dir (legacy <stem>.js):
cargo run -q -p sail-compiler -- build views --out target/sail-debug --clean

# Component mode — implied by --component-name:
cargo run -q -p sail-compiler -- build packages/components/ui-components/src/sail-button/sail-button.sail \
  --component-name sail-button \
  --styles packages/components/ui-components/src/sail-button/sail-button.css
```

Flags (from `crates/sail/compiler/src/bin/sail-compiler.rs`):

| Flag | Purpose |
|---|---|
| `--out <dir>` | Single output directory (legacy `<stem>.js` naming). When omitted, emits to `<repo-root>/.ship/sail-out/<src-rel>/<stem>.compiled.js`. |
| `--clean` | Remove each source file's mirror directory (or `--out` directory) before emitting. Idempotent rebuilds. |
| `--mode <view\|component\|auto>` | Override mode (otherwise auto-detected from `--component-name`). |
| `--no-dts` | Suppress `.sail.d.ts` emission. |
| `--no-ssr` | Suppress `.ssr.html` emission (e.g. island bundles). |
| `--styles <file>` | Prepend a CSS file as a `<style>` block. Component sources only. |
| `--component-name <tag>` | Set the custom element tag name; implies component mode. |

The binary is also exposed as a `cdylib` (`crate-type = ["lib", "cdylib"]` in `Cargo.toml`) so editor extensions and the deck bundler can call it without spawning a subprocess. The Rust release build is what the daemon ships at `~/.ship/runtime/sail-compiler` (see `crates/deck/deck-bundler/README.md` — the toolchain doc moved next to its code per the docs charter).

## The deck bundler — the ONLY bundler

There is one bundler: the in-process Rust Rolldown bundler in `crates/deck/deck-bundler/` (#163, #167, #220). The retired `@ship/sail-build` (esbuild, one package at a time) and the old `packages/deck/bundler/index.ts` Bun worker are both gone — components are no longer pre-compiled to on-disk `dist/` bundles per package.

**How it works.** A Rust Rolldown plugin (`SuiPlugin`) compiles each `.sail` via `sail-compiler` in its `load` hook — no bun worker, no subprocess, no `.compiled.js` pre-step. Membership is **traced**, not declared: each entry (a view, an app, a marketing/arcade page) is bundled by following its imports and template tags.

**Two modes**, wired to `DaemonMode`:
- **Development** — the daemon serves compiled `.sail` as un-bundled native ESM; HMR re-compiles only the edited component.
- **Production** — Rolldown emits per-entry traced chunks with `sail-runtime`/`sail-dom` as the shared external, immutable-cached.

**Baking.** `cargo run -p deck-cli -- build-assets` (aliased `bun run build:bundles`) bakes the framework singletons + the `sail-components` umbrella + per-component islands (`<bundle separate />`) + per-view bundles + the site's `sail-marketing.js` / `sail-arcade.js` into `crates/deck/deck-core/assets/` and `.deck/` — the same on-disk asset trees the release binary `include_str!`s. It's what `smoke-deploy.sh` runs and what `just cem` invokes.

## `sail-lsp` — the language server

Source: `crates/sail/lsp/` (Rust). A thin LSP/stdio shell over the `sail-compiler`
analysis facade — **one grammar, no TypeScript, no WASM, no Node**. The same
Rust intelligence serves three consumers:

- **Claude Code** — `.lsp.json` at the repo root points the editor's LSP
  client at the `sail-lsp` binary (`extensionToLanguage: { ".sail": "sui" }`).
- **opencode** — add to the (user-local, gitignored) `opencode.json`:
  `"lsp": { "sui": { "command": ["sail-lsp"], "extensions": [".sail"] } }`. The
  binary ignores extra args, so opencode's `--stdio` convention is harmless.
- **VS Code / any editor** — `apps/vscode` launches `sail-lsp` over stdio
  (override with the `sui.serverPath` setting); any LSP-capable editor maps the
  `sui` language id to the `sail-lsp` binary the same way.

```sh
just install-dev        # builds + installs `sail-lsp` to ~/.local/bin
sail-lsp                 # LSP server on stdio (what editors + .lsp.json launch)
```

Intelligence is the Rust compiler: `sail_compiler::check_all` (multi-diagnostic),
`SymbolTable` (scope names + kinds + declaration offsets, byte-walked from the
inline `<controller>` block — no `typescript` package), and an IR walk for the
tag/event dependency surface (`sail_compiler::analyze`).

> **Retired:** the former TypeScript LSP (`packages/deck/lsp`, ex-`ship-set-lsp`)
> — a fork of the `.sail` grammar that also bundled the full `typescript` package
> — has been deleted. Its grammar duplicated `sail-compiler`; editors use the
> `sail-lsp` binary, and the docs-playground browser analyzer (`packages/deck/lsp-browser`)
> calls the WASM `analyze`. One grammar, no TS fork.

VS Code: `apps/vscode/` auto-launches `sail-lsp` on `.sail` open. Other editors map
the `sui` language ID to the `sail-lsp` binary over stdio.

## Per-edit lint and compile loop

MCP is removed from this project. The verification loop is the binary. Run both per edit, not at PR time.

```sh
# Fast per-edit lint (grammar, missing bindings, unstable keys, purity,
# css tokens, prop values). Exits 1 on findings — wire it into your loop:
cargo run -p sail-lint --example check -- views/hello/page.sail

# Full compile (lint folds into the diagnostic stream; aborts on errors):
cargo run -p sail-compiler -- build views/hello/page.sail
```

## UI Studio — the in-browser catalog

`apps/shell/src/studio/`, mounted at `/studio` (rail icon: `columns`).

| File | Owns |
|---|---|
| `page.sail` (with its inline `<controller>` block) | Catalog index — search, filter (All / Components / Views), grouped nav |
| `[name]/page.sail` (with its inline `<controller>` block) | Detail page — exercise prop knobs, slot compositions, deep-link state, render matrix |
| `catalog.ts` | The component / view registry the index reads |
| `[name]/matrix.ts` | Render-matrix configurations (prop × value grid) |
| `[name]/slot-compositions.ts` | Slot composition presets |
| `[name]/deep-link.ts` | URL-state serialization for the detail page |
| `[name]/snippet.ts` | Copy-as-snippet renderer |

Use it to find a component, browse its props (including enum values from `<prop values="…">`), exercise edge cases (long strings, empty slots, error states), and copy the matching template snippet straight into your view.

## Devtools panes — in-page introspection

`packages/components/ui-components/src/sail-devtools/` ships five panes that share a separate bundle (opted in via `<bundle separate />` in each `.sail` source). Each is a `.sail` file with its own inline `<controller>` block, lazy-loaded on first mount via the `sail-devtools.js` bundle.

| Tag | What it shows |
|---|---|
| `<sail-devtools-source>` | The mounted view's template + manifest. Editable for user-scoped views; read-only for project-scoped (Fork affordance for project scope). Embeds `<sail-code-editor>` lazily. |
| `<sail-devtools-components>` | Live component tree of the mounted view |
| `<sail-devtools-lint>` | SUI lint findings for the current file |
| `<sail-devtools-schema>` | Prop schema for selected component |
| `<sail-devtools-signals>` | Live signal values; subscribes + repaints on change |

`apps/shell/src/main.ts:139` (`// Notify devtools tabs of navigation`) wires navigation events into the panes so they refresh on route changes.

## `dispatch.sh` — worktree + job-spec + agent spawn

`.ship/skills/spawn-agent/scripts/dispatch.sh` is the canonical script that creates a fresh worktree, writes a job spec, compiles the agent, and opens a terminal in the worktree. Used directly by `cmdr` and indirectly by the in-app dispatch dialog.

```sh
bash .ship/skills/spawn-agent/scripts/dispatch.sh \
  --slug docs-sail-arc \
  --agent ship-designer \
  --spec .artifacts/job-spec.md
```

Flags (from `dispatch.sh`):

| Flag | Default | Meaning |
|---|---|---|
| `--slug <name>` | required | Worktree directory name + branch name (`job/<slug>`). |
| `--agent <agent>` | required | Agent profile to compile into the worktree (`ship-designer`, `commander`, etc.). |
| `--spec <file>` | required | Path to the job-spec markdown file. Copied to `.artifacts/job-spec.md` in the worktree. |
| `--dir <path>` | resolved | Worktree base. Precedence: `--dir` > `SHIP_WORKTREE_DIR` env > `ship config get worktrees.dir` > `~/dev/ship-worktrees`. |
| `--base <branch>` | current HEAD | Base branch for the worktree. |
| `--no-open` | false | Skip opening a terminal in the worktree. |
| `--dry-run` | false | Print actions without executing them. |
| `SHIP_AGENT_MODEL` env | unset | Override the agent's default model. |

The script honours `SHIP_DISPATCH_CONFIRM` for non-interactive confirmation prompts and resolves `~` in the worktree base. It is not specific to SUI work — it dispatches any agent — but every SUI authoring job (this docs job included) lands in a worktree created by it.

## Scaffolds

```sh
# Drop a fresh view directory:
deck scaffold view settings
deck scaffold view settings --path /settings
deck scaffold view settings --component settings-toolbar

# Drop a fresh component directory:
deck scaffold component my-widget
deck scaffold component my-widget --controller
```

Both scaffolds emit the manifest / template / controller / style files documented in `references/authoring-patterns.md` § Scaffolds. Run them from the repo root.

## End-to-end edit loop in dev

```sh
just dev                              # daemon (in-process Rolldown bundler)
# (optional) just shell-dev          # vite outer shell at :3200, proxies to daemon
# edit views/home/page.sail or controller.ts
#  → the daemon's in-process bundler (notify watch) re-bundles via Rolldown,
#    writing to crates/deck/deck-core/assets/... or .deck/...
#  → emits BundlerEvent::ModuleChanged (broadcast) → sui_hmr
#  → shipd republishes on /_deck/dev/hmr (SSE)
#  → apps/shell/src/hmr.ts cache-busts /_deck/views/home/view.js
#  → mount() re-runs in place; SSR snapshot stays, no full reload
```

`just dev` starts only the daemon (`scripts/dev/start.sh`); bundling + watching happen IN-PROCESS (`crates/deck/deck-bundler/`, `Bundler::start_watching`) — no spawned worker. To hit the daemon through the outer shell, run `just shell-dev` in a second terminal — it proxies `/api`, `/_deck`, `^/sui/.*`, and `/view-host` to the daemon.

For a clean dev restart use `just dev-restart`. Visual regressions: `just visual` (Playwright vs committed PNGs; pass `--update-snapshots` to refresh after a deliberate change). Perf regressions: `just bench-check` (fails on >15% vs the latest committed baseline; override via `BENCH_REGRESSION_THRESHOLD`).

## Fresh-clone bootstrap

`just bootstrap` runs the full first-time-setup sequence (verifies bun on PATH, then `cargo build -p ship-cli` → `bun install` → `bun run --filter @ship/ui-components icons` → `bun run --filter @ship/ui-components build` → `bun run --filter @ship/ui-components build` → `just dev-runtime-install`). After it completes, `just dev` brings up the daemon.

## Development vs production modes

Shipd has two operating modes, selected via `--mode <development|production>` (or `SHIP_DAEMON_MODE`). The same binary, the same routes — different headers, different bundlers, different feedback loops.

| Aspect | Development (default) | Production |
|---|---|---|
| Frontend serving | Vite dev server (`just shell-dev`) at 3200 +offset, proxying `/api`, `/_deck`, `/view-host` to the daemon | Daemon serves prebuilt shell HTML directly; no Vite, no proxy, no second port |
| Bundler | In-process Rolldown (`Bundler::start_watching`, notify) re-bundles on edit; no spawned worker | Pre-bundled by `just build`; daemon `include_str!`s the static output |
| HMR | `/_deck/dev/hmr` SSE channel; `apps/shell/src/hmr.ts` cache-busts view modules and re-runs `mount()` in place | Disabled — HMR routes are not registered in production |
| Cache headers | `Cache-Control: no-cache` on every asset and view artifact; no ETag | `Cache-Control: public, immutable, max-age=31536000` on framework bundles, `must-revalidate` on view artifacts; ETag on every response |
| SSR cache | Bypassed every request (sources may have changed under us) | Memoized on first hit and reused until the daemon restarts |
| Error detail | Full error bodies | Redacted; only the kind of failure leaks to the client |
| Status bar | `<sail-status-bar mode="development">` shows a hairline `DEV` badge between the connection dot and the branch | Badge hidden; same component, no chrome |

The mode is exposed to the browser via `<meta name="ship-mode">` in the shell HTML. shell's `main.ts` reads it into `window.__shipMode` (`'Production'` | `'Development'`) at boot and reflects it onto the status bar's `mode` attribute. When shipd serves the prebuilt shell itself, `deck::inject_mode_meta` rewrites the meta to match the daemon's actual mode; the static default in `apps/shell/index.html` is `development` so the Vite dev server doesn't need any special handling.

Producing a deployable artifact set:

```sh
just build       # one-shot: gen-types → component packs (primitives, ui-components,
                 # charts, pipelines) → cargo run -p deck-cli -- build-assets → cargo build
                 # --release → verify outputs
ship build       # equivalent CLI wrapper for users without `just` installed
```

Both write release binaries to `target/release/{ship,shipd,mcp}` and rebuild the front-end asset trees that shipd's `include_str!` directives consume (`crates/deck/deck-core/assets/`, `apps/shipd/assets/`). After `just build`, run `shipd --mode production` to serve them.
