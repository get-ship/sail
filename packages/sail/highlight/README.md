# @ship/sail-highlight

Tree-sitter-sail highlighter. **Not a parser.** Compiler / `sail lsp` / fmt stay Rust.

One query (`packages/sail/grammar/queries/highlights.scm`) + one wasm
(`tree-sitter-sail.wasm`, built from that grammar). Consumers:

- `packages/sail/vscode` — semantic tokens
- `<sail-code-editor>` — CodeMirror decorations

```sh
bun run build-wasm   # real node, not bun's node symlink
bun run test
```

A new `@capture` in `highlights.scm` without a row in `src/scopes.ts`
and `bindings/rust/highlight.rs` `SCOPES` fails CI (`scopes-drift.test.ts`,
`cargo test -p tree-sitter-sail --test highlight`).
