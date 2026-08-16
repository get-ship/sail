# tree-sitter-sail

Tree-sitter grammar for `.sail`. **Highlighter**, not the product parser.
Compiler / `sail lsp` / `sail fmt` stay Rust.

Intelligence in every editor is `sail lsp` (`sail` on PATH). Color is
this package: `grammar.js` + `src/parser.c` + `queries/`.

## Editors

Helix, nvim, and Zed load tree-sitter grammars natively. Point the
editor at this directory (or a clone of it) and start `sail lsp`.

### Helix

```sh
sail editor setup helix
```

Writes `~/.config/helix/languages.toml` (`sail lsp` on PATH), extracts the
grammar under `~/.local/share/sail/grammar`, and compiles `sail.so` into
the existing Helix runtime. Does **not** run `hx --grammar build`.

Or by hand — `sail` must be on PATH:

```toml
[language-server.sail]
command = "sail"
args = ["lsp"]

[[language]]
name = "sail"
scope = "source.sail"
file-types = ["sail"]
comment-token = "<!--"
indent = { tab-width = 2, unit = "  " }
language-servers = ["sail"]
grammar = "sail"

[[grammar]]
name = "sail"
source = { git = "https://github.com/get-ship/sail", subpath = "packages/sail/grammar", rev = "main" }
```

Until the git URL is the public repo, use `source = { path = "<clone>/packages/sail/grammar" }`
and `hx --grammar build` **only for `sail`**. Bare `hx --grammar build`
rebuilds every language and can drop stock `.so` files.

Controller bodies inject TypeScript. Helix needs its own `typescript.so`.

### Neovim

Same grammar directory. Register `sail` with nvim-treesitter (path/git)
and `vim.lsp.config` / lspconfig: `cmd = { "sail", "lsp" }`.

### Zed

Declare language `sail` for `*.sail`, grammar source = this directory,
language server `{ "command": "sail", "args": ["lsp"] }`. A Zed
extension that bundles those three facts is the store-shaped install;
unpublished until identity is picked.

### VS Code / Cursor

`packages/sail/vscode` is an LSP client (`sail lsp`) plus semantic tokens
from `@ship/sail-highlight` (this grammar compiled to wasm). Not a
TextMate grammar.

### Studio / docs

`<sail-code-editor>` paints from `@ship/sail-highlight` (same wasm +
query). Docs `<code-block>` uses the same package via
`registerSailHighlighter` (TS / shell snippets stay on the runtime
regex). The generated vocab table is leftover generate output, not
the paint engine.

## Generate (maintainers)

```sh
bun run generate        # compiler vocab → grammar.js + highlights.scm + overlay
bun run generate-check  # drift vs committed
tree-sitter generate    # grammar.js → src/parser.c  (real node, not bun)
# then, from packages/sail/highlight:
bun run build-wasm      # parser.c → tree-sitter-sail.wasm
```

`cargo test -p tree-sitter-sail` fails if `grammar.js` / queries drift
from the compiler, or if `src/grammar.json` does not list the same
rules as `grammar.js` (forgot `tree-sitter generate`).
