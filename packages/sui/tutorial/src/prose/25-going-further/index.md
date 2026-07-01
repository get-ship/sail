---
title: "Going further"
description: "Where to take SUI after the tutorial."
group: "Tutorial"
order: 25
slug: "25-going-further"
prev: "24-morphs"
next: null
chapter: "views"
---

You've covered the SUI surface end-to-end. Twenty-five steps across
seven chapters — Reactivity, Templating, Components, Agents,
Composition, Lifecycle, and Views — that ran the full client-server
loop against a mock daemon. From here:

## Build something real

1. **`scaffold_view`** — the MCP tool that emits a fresh
   `views/<name>/` skeleton. Or `deck scaffold view <name>`
   from a shell.
2. **`docs/ship-sui/references/docs/agent-authoring.md`** — the do-now
   checklist. File layout, the `<data>` directive's real shape,
   common failure modes, the verification ladder.
3. **`docs/ship-sui/references/docs/cookbook.md`** — ~30 recipes covering
   patterns you'll hit (form validation flows, optimistic UI,
   pagination, infinite scroll, error boundaries, theme switching).

## Tooling reach

- **LSP**: the native `sui-lsp` binary (on your PATH after install) —
  diagnostics/hover/completion over stdio, the same `sui-compiler`
  grammar the build uses.
- **Binary lint/compile loop**: `cargo run -p sui-lint --example check -- <file>` for
  fast per-edit lint; `cargo run -p sui-compiler -- build <file>` for the full compile.
  Every authoring surface is documented in `docs/ship-sui/references/docs/agent-surfaces.md`.
- **`@ship/sui-eval`**: programmatic behavioural scoring. Wire into
  CI to gate PRs that touch `.sui` files.
- **Editor LSP** — register `sui-lsp` via `.lsp.json` and any LSP-capable
  editor (VS Code, Neovim, Helix) attaches on `.sui` open.

## Where to read next

| Doc | When |
|---|---|
| `docs/ship-sui/references/docs/authoring-model.md` | The line between `.sui` and `controller.ts` — pure expressions vs. capabilities, why it's not JSX, with a worked before/after. |
| `docs/ship-sui/references/docs/template.md` | Full template surface — every directive, every binding, every block keyword. |
| `docs/ship-sui/references/docs/controller.md` | Three-tier controller contract. Scope rules, hook composition, mutation discipline. |
| `docs/ship-sui/references/docs/hooks.md` | Template vs controller hooks — when to reach for which. |
| `docs/ship-sui/references/docs/data-subscriptions.md` | `<data live>` deep-dive: envelope, patch protocol, SSR seeding. |
| `docs/ship-sui/references/docs/effect-scope.md` | Cleanup ownership on unmount. |
| `docs/ship-sui/references/docs/ssr-hydration.md` | Server-render + attach handshake. |
| `docs/ship-sui/references/docs/agent-surfaces.md` | The agent-facing tool inventory you've been using. |

## What the playground doesn't cover

The docs site isn't coupled to a daemon. Things you'll see in a real
Ship project but not here:

- **Routing**: route params, `<Link>`, redirects, route-level data
  preloads. Routes and navigation arrive through the host context
  (`ctx.route`, `ctx.navigate`) — routing lives in **deck**, not SUI.
- **SSR**: the server pre-renders `<data live>` to HTML, the client
  attach takes over. Covered in `ssr-hydration.md`.
- **Manifest**: `manifest.jsonc` declares routes + auth + theme +
  packs. Read `agent-authoring.md` § Manifest.
- **Mesh**: multiple daemons sharing live signals. Architecture-only
  reading; not exercised in single-tab tutorials.

## Try it

The preview shows a tiny "what's next" card with a link to each of
the docs above — kept as a useful starting point you can come back to.
