---
group: Agents
order: 1
title: SUI — Agent Surfaces
description: Every programmatic surface an agent uses to author, diagnose, and exercise SUI views — the ship CLI, the LSP, sui-eval scoring, and the ConnectRPC op surface.
---

# SUI — Agent Surfaces

The set of programmatic surfaces an agent uses to author, diagnose, and
exercise SUI views or components. Three planes:

| Plane | Use it for | Lives in |
|---|---|---|
| **`ship ops call`** | Live operations against an active project. Per-domain ConnectRPC ops (`apps/mcp/` was removed in #142-144). List with `ship ops list`. | `ship ops call <op-name> --json '{…}'` |
| **CLI lint + compile loop** | Fast per-edit verification — the primary authoring path. | `cargo run -p sui-lint --example check -- <file>` (lint); `cargo run -p sui-compiler -- build <file>` (full compile) |
| **`ship sui`** | SUI language intelligence (diagnostics, hover, dependencies) over ConnectRPC. The daemon answers in-process via the `sui_compiler` facade. | `ship sui diagnostics <file>`, `ship sui hover <file> <line> <col>`, `ship sui dependencies <file>` |
| **`sui-lsp` binary** | Native Rust LSP over stdio for editors and the ship-agent's tooling. Same grammar as the build. | `crates/sui/lsp/` |
| **`@ship/sui-lsp-browser`** | Browser-only structural analyser + completion sources. Web worker, no Node deps, no `typescript` package. Used by the docs playground; the shell editor can adopt it too. | `packages/deck/lsp-browser/src/` |
| **`@ship/sui-eval`** | Regression-quality scoring of agent-produced SUI against a behavioural spec | `packages/deck/eval/src/index.ts` |

Read this alongside `SKILL.md` (the four-layer overview) and
`agent-authoring.md` (the do-now checklist for shipping a view in one
turn).

---

## 1. Agent CLI surface

`apps/mcp/` no longer exists — it was removed in #142-144. Agents use the
`ship` CLI and the ConnectRPC op surface instead. There are no ship MCP tools.

### 1.1 SUI language intelligence (`ship sui`)

The `ship sui` subcommand calls `SuiService` over ConnectRPC. The daemon
answers in-process via the `sui_compiler` facade. Requires a running daemon.

| Command | Use |
|---|---|
| `ship sui diagnostics <file>` | All diagnostics (parse + semantic + lint) for a `.sui` file — JSON array. |
| `ship sui hover <file> <line> <col>` | Hover info for the symbol at 0-based `(line, col)`. |
| `ship sui dependencies <file>` | Dependency surface: custom-element tags, handled events, controller path. |

For offline / no-daemon use, the binary loop is faster:

```sh
cargo run -p sui-lint --example check -- <file>      # grammar + lint; exits 1 on findings
cargo run -p sui-compiler -- build <file>            # full compile; aborts on errors
```

### 1.2 Scaffolding

```sh
deck scaffold view <name>         # new views/<name>/ skeleton (page.sui, controller.ts, manifest.jsonc)
deck scaffold component <tag>     # new component under packages/components/ui-components/src/<tag>/
deck scaffold pack <pack>         # new independent component pack
```

### 1.3 Daemon ops (`ship ops`)

```sh
ship ops list                          # print the full read + command catalog
ship ops call <op-name> --json '{…}'  # invoke any op by dotted name
```

Agent-relevant ops (call `ship ops list` for the live set):

| Op | Use |
|---|---|
| `pipelines.step_done` | Signal that the current pipeline step is complete (canonical agent completion signal). |
| `pipelines.update_job` | Update whole-job status (final step or explicit lifecycle transition). |
| `pipelines.log_progress` | Emit a progress note against the active job. |
| `git.status` | Git status of the active workspace. |
| `workspaces.list` | Active project workspaces. |
| `sessions.list` | Active sessions. |
| `skills.list` | Skill catalog. |

The op catalog is generated from the daemon's proto registry — run
`ship ops list` for the current set.

### 1.4 Component packs (post-decomposition)

Components are decomposed into independent workspace packs under
`packages/components/<pack>/`. Two published packs today:

| Pack | Tags | Bundle |
|---|---|---|
| `@ship/primitives` | ~80 generic `<sui-*>` (button, input, dialog, popover, …) | `/_deck/packs/primitives/sui-primitives.js` |
| `@ship/charts` | 8 chart components | `/_deck/packs/charts/sui-charts.js` |

Chrome + domain components (sui-code-editor, sui-terminal, picker,
nav, devtools, …) stay in the `@ship/ui-components` umbrella and ship
via the daemon's `sui-components.js` aggregate.

The `@ship/sui-catalog` walker produces two projections under
`.deck/catalogs/`:

| File | Contains | Agent use |
|---|---|---|
| `workspace.json` | Every component in the repo (public + private) | Authoring agents — full surface. |
| `published.json` | `!private` packs only | Library-consumer agents — what npm publishers see. |

Each catalog entry gains a `pack` field so filters can target one
pack (`entries.filter(e => e.pack === '@ship/primitives')`). The
daemon serves per-pack bundles at `/_deck/packs/<pack>/<bundle>.js`.

---

## 2. `sui-lsp` binary (editors + ship-agent tooling)

The native Rust language server (`crates/sui/lsp`, on PATH after
`just install-dev`). Speaks LSP over stdio — the ship-agent launches
it via the repo's `.lsp.json`, VS Code via `sui.serverPath`, any
editor via the `sui` language id. Same `sui-compiler` grammar as
the build.

The old one-shot TypeScript CLI (`packages/deck/lsp/src/cli.ts`) is gone. Agents
without an editor use the binary loop: `cargo run --manifest-path Cargo.toml -p sui-lint --example check -- <file>` for

Output shape (diagnostics):

```json
[
  { "line": 12, "col": 4, "severity": "error", "message": "unknown modifier \"enter\"", "source": "sui-compiler" },
  { "line": 18, "col": 9, "severity": "warning", "message": "<data> directive without a `name` attr", "source": "sui-lint" }
]
```

The LSP binary uses the **same analyser** as the binary lint/compile loop — one Rust grammar, multiple delivery surfaces.

---

## 3. `@ship/sui-eval` — regression scoring

When an agent authors a `.sui` view, `sui-eval` runs it against a spec
and scores the output. Source: `packages/deck/eval/src/index.ts`.

```ts
import { runEval, type EvalTask } from '@ship/sui-eval'

const task: EvalTask = {
  description: 'A counter with +1/-1 buttons that resets to 0.',
  catalog: await loadCatalog(),                    // ship://components dump
  spec: {
    mount: (el) => { /* assertions here */ },
    interactions: [
      { action: 'click', selector: 'button[data-action="bump-up"]' },
      { assert: 'text', selector: 'strong', value: '1' },
    ],
  },
}

const result = await runEval(task, { model: 'sonnet' })
// → { score: 0.92, sub: { compile: 1, scope: 1, behaviour: 0.83 }, report: '…' }
```

Sub-scores:
- **compile** — does it parse + lower without errors?
- **scope** — does every signal / directive / tag resolve?
- **behaviour** — does it pass the spec's interaction script in jsdom?

`sniffEscapeHatches(output)` flags illegal escapes (raw `<script>`,
`@ts-ignore`, `eval`, custom imports outside the importmap).

---

## 4. Skills

Two files own the agent-facing prose:

| File | Audience | Use |
|---|---|---|
| `docs/ship-sui/SKILL.md` | Loaded automatically when the `ship-sui` skill activates | Architecture, hard rules, the four layers. |
| `docs/ship-sui/references/docs/agent-authoring.md` | The do-now checklist | Step-by-step view scaffold, common mistakes, the green-path workflow. |

Load the file directly or activate from the Ship agent configuration — there is no `set_skill_var` tool.

---

## 5. Gaps + non-surfaces

What an agent might reasonably expect but does NOT exist:

- **No formatter for `.sui`.** `prettier --plugin sui` is not implemented. Agents emit hand-formatted output.
- **Browser diagnostics via WASM.** `@ship/sui-lsp-browser` (the docs-playground worker) calls the WASM `analyze` from `@ship/sui-compiler-wasm` — the same Rust grammar as the build — for parse + semantic + lint diagnostics. Hover/autocomplete in the browser are still the static-dictionary sources, not yet identifier-aware.
- **No multi-file scaffold mutation tool.** `deck scaffold view` produces the skeleton once; subsequent multi-file edits are direct filesystem writes.
- **No live-edit collision detection.** Two agents writing to the same view concurrently get last-writer-wins. Coordinate via mesh status ops or job-level sequencing.
- **No assertion-style step gate for `sui-eval`.** The scoring is informational; you have to read the score yourself and decide whether to retry.
