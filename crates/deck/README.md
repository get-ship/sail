# crates/deck/

Deck host / meta-framework for SAIL routes: routing, SSR, bundling, CLI.

| Crate | Role |
|---|---|
| `deck-core/` | Transport-agnostic serving |
| `deck-axum/` | axum adapter |
| `deck-host/` | Host integration |
| `deck-bundler/` | In-process Rolldown bundler |
| `deck-config/` | serve config + host layout types |
| `deck-cli/` | `sail` binary |
| `deck/` | Facade |
| `deck-conformance/` | Standalone harness app |
| `deck-ssr/` | V8 SSR path (opt-in member) |

Feature inventory (drift-gated): [`FEATURES.md`](./FEATURES.md).

```sh
cargo run -p deck-cli -- build-assets
cargo run -p deck-cli --bin sail -- dev --project apps/sail-site
cargo test -p deck-core
```
