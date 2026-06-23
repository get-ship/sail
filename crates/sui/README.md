# crates/sui/

The **SUI view framework** (Rust side) — the `.sui` language, its ahead-of-time
compiler, lint rules, and the signal-graph runtime. SUI is the "React" of the
stack; the meta-framework that hosts, serves, and bundles SUI views lives next
door in [`crates/deck/`](../deck/). See the ADR
[`sui-deck-crate-boundary`](../../docs/ship-adrs/references/docs/sui-deck-crate-boundary.md).

`.sui` files compile **ahead-of-time to ESM JavaScript** attached to a
signal-graph runtime — not Jinja, not Datastar. The Rust crates here are the
pure, portable leaves of that pipeline.

## Crates

| Crate | Workspace member? | Role |
|---|---|---|
| `compiler/` | yes | `.sui` → ESM JavaScript transform (`sui-compiler`). Pure — no FS, no env; WASM-safe. Also exposes `check_all` / `analyze` for the LSP + agent tooling. |
| `lint/` | yes | Lint rules for `.sui` templates + paired `.controller.ts` (oxc-parsed). Findings shared with the compiler's build-time checks. |
| `signals/` | yes | Signal-graph runtime — the fine-grained reactivity layer the compiled output binds to. |
| `lsp/` | yes | `sui-lsp` — a thin LSP/stdio server over `compiler`'s analysis facade. One grammar for editors + agents; no TypeScript, no WASM. |

## Layer convention

- `compiler`, `lint`, `signals` are **pure leaves** — no native I/O, WASM-safe.
- `lsp` is a thin transport binary over `compiler` (stdio LSP loop only).
- Anything that *resolves*, *SSRs*, *serves over HTTP*, or *bundles* SUI views
  is **Deck**, not SUI — it lives in [`crates/deck/`](../deck/)
  (`deck-host`, `deck-server`, `deck`). Those crates depend on `sui-compiler` +
  `sui-signals`; the dependency only ever points sui ← deck, never the reverse.

## Why this split

The Rust workspace mirrors the JS package split so the mental model is stated
once and both agree:

```
crates/sui/   = SUI view framework (Rust)   ↔  packages/sui/   = SUI (JS)
crates/deck/  = Deck meta-framework (Rust)  ↔  packages/deck/  = Deck (JS)
crates/core/  = substrate + platform tier (kernel, pipelines, supervisor, …)
```

The host/server/facade crates used to sit in `crates/ship/` under misleading
`ship-sui-*` names; the `sui-deck-crate-boundary` ADR moved them to
`crates/deck/` and renamed them to `deck-*`.
