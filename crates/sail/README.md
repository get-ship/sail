# crates/sail/

The **SAIL view framework** (Rust side) — the `.sail` language, its ahead-of-time
compiler, lint rules, and the signal-graph runtime. SAIL is the "React" of the
stack; the meta-framework that hosts, serves, and bundles SAIL views lives next
door in [`crates/deck/`](../deck/). See the ADR
[`sail-deck-crate-boundary`](../../docs/ship-adrs/references/docs/sail-deck-crate-boundary.md).

`.sail` files compile **ahead-of-time to ESM JavaScript** attached to a
signal-graph runtime — not Jinja, not Datastar. The Rust crates here are the
pure, portable leaves of that pipeline.

## Crates

| Crate | Workspace member? | Role |
|---|---|---|
| `compiler/` | yes | `.sail` → ESM JavaScript transform (`sail-compiler`). Pure — no FS, no env; WASM-safe. Also exposes `check_all` / `analyze` for the LSP + agent tooling. |
| `lint/` | yes | Lint rules for `.sail` templates + controller TypeScript (oxc-parsed). Findings shared with the compiler's build-time checks. |
| `signals/` | yes | Signal-graph runtime — the fine-grained reactivity layer the compiled output binds to. |
| `lsp/` | yes | `sail-lsp` — a thin LSP/stdio server over `compiler`'s analysis facade. One grammar for editors + agents; no TypeScript, no WASM. |

## Layer convention

- `compiler`, `lint`, `signals` are **pure leaves** — no native I/O, WASM-safe.
- `lsp` is a thin transport binary over `compiler` (stdio LSP loop only).
- Anything that *resolves*, *SSRs*, *serves over HTTP*, or *bundles* SAIL views
  is **Deck**, not SAIL — it lives in [`crates/deck/`](../deck/)
  (`deck-host`, `deck-server`, `deck`). Those crates depend on `sail-compiler` +
  `sail-signals`; the dependency only ever points sail ← deck, never the reverse.

## Why this split

The Rust workspace mirrors the JS package split so the mental model is stated
once and both agree:

```
crates/sail/   = SAIL view framework (Rust)   ↔  packages/sail/   = SAIL (JS)
crates/deck/  = Deck meta-framework (Rust)  ↔  packages/deck/  = Deck (JS)
crates/core/  = substrate + platform tier (kernel, pipelines, supervisor, …)
```

The host/server/facade crates used to sit in `crates/ship/` under misleading
`ship-sail-*` names; the `sail-deck-crate-boundary` ADR moved them to
`crates/deck/` and renamed them to `deck-*`.
