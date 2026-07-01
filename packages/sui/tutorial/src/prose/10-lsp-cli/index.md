---
title: "The LSP — diagnostics on tap"
description: "One Rust grammar, two binary delivery channels — the sui-lint fast check and the sui-compiler full compile."
group: "Tutorial"
order: 10
slug: "10-lsp-cli"
prev: "09-agent-loop"
next: "11-sui-eval"
chapter: "agents"
---

SUI's language intelligence is the Rust compiler — `sui_compiler::analyze`
(diagnostics + scope symbols + dependencies) and `check_all` (every diagnostic,
not just the first). It's the same grammar the build uses, exposed to agents
through two binary commands.

## Fast per-edit lint

```sh
cargo run -p sui-lint --example check -- views/tasks/page.sui
```

Exits 1 on findings. Covers grammar errors, missing bindings, unstable-key
warnings, purity violations, CSS token errors, and prop-value mismatches. Run
this in your edit loop — it's the cheapest check and catches the most
common authoring errors.

Output on a file with issues:

```
views/tasks/page.sui:12:4 error: `new` is not allowed in renderable expressions [expr-subset]
views/tasks/page.sui:18:9 warning: <data> directive without `name` attr [sui-data-name]
```

`severity` is `error` (compile-blocking) or `warning` (compiles but suspicious).
The rule id is in brackets — useful for filtering or mapping to your editor's
diagnostic provider. Positions are 1-based.

## Full compile

```sh
cargo run -p sui-compiler -- build views/tasks/page.sui
```

Runs the full pipeline (parse → check → lint → lower → emit) and aborts on
errors. More thorough than the lint check — catches lowering-level issues the
fast pass misses. Also emits the compiled JS, `.sui.d.ts`, and SSR HTML into
the project mirror tree. Use this after the lint check is clean.

## The `sui-lsp` binary (editors)

For editors — VS Code, Neovim, Helix, or any LSP-capable client — the
`sui-lsp` binary provides the same intelligence over stdio. Same Rust
grammar; hover, completion, and
go-to-definition on top. Register it for `.sui` files with a `.lsp.json`
at your project root and any LSP-capable editor attaches automatically:

```json
{ "sui": { "command": "sui-lsp", "extensionToLanguage": { ".sui": "sui" } } }
```

## Why this matters

There is **one** source of truth — the Rust `sui_compiler` grammar — flowing
through multiple surfaces:

- **`sui-lint`** — fast per-edit binary for agents and CI
- **`sui-compiler`** — full compile with emit, used by the deck bundler
- **`sui-lsp`** — LSP server for editors over stdio
- **WASM compiler** (`@ship/sui-compiler-wasm`) — in-browser diagnostics for the docs and any embedded editor

A human typing in the docs site, an agent running `sui-lint`, and an editor
on your laptop all see the *same* diagnostics — because they're all the same
compiler. No second grammar to drift out of sync.
