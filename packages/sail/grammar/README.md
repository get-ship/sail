# tree-sitter-sail

Tree-sitter grammar for `.sail`. **Highlighter**, not the product parser
(compiler / `sail lsp` / `sail fmt` stay Rust).

## Helix (first editor)

From this repo, with `sail` on PATH (`cargo install --path crates/deck/deck-cli --bin sail`):

```sh
cd packages/sail/grammar
bun run helix-setup
```

That links `queries/` into `~/.config/helix/runtime/queries/sail`, appends
an absolute `[[grammar]]` path to `~/.config/helix/languages.toml`, and
builds `sail.so`. This checkout’s `.helix/languages.toml` wires `sail lsp`.

Does **not** replace `~/.config/helix/runtime` (a symlink to the install
must stay, or stock themes disappear). Open a `.sail` from the repo root.

After `bun run generate`, run `helix-setup` again so queries and the
compiled grammar match.

## Generate

```sh
bun run generate        # vocab dump → grammar.js + highlights.scm + overlay table
bun run generate-check  # drift vs committed (overlay included)
tree-sitter generate    # grammar.js → src/parser.c (Helix / nvim runtime)
```
