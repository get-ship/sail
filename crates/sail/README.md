# crates/sail/

Rust side of SAIL: `.sail` compiler, lint, LSP, layout, ops, MCP.

| Crate | Role |
|---|---|
| `compiler/` | `.sail` → ESM (`sail-compiler`); pure / WASM-safe |
| `compiler-napi/` | Node N-API binding |
| `lint/` | Template + controller lint rules |
| `lsp/` | stdio language server |
| `layout/` | Static layout diagnostics |
| `ops/` | Ops data model for tooling |
| `mcp/` | MCP server |
| `render/` | Static render support |

Host / serve / bundle lives in [`crates/deck/`](../deck/). Dependency direction:
**deck → sail**, never reverse.

```sh
cargo test -p sail-compiler
cargo run -p sail-compiler -- skill    # regenerate authoring skill
```
