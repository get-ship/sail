# Sail — VS Code / Cursor language client

Local package (not a marketplace publish). It registers language `sail` for
`*.sail` and starts **`sail lsp`** over stdio. Hover, complete, definition,
diagnostics, format, and code actions come from that server — this extension
does not invent a second protocol.

Colorization is **tree-sitter-sail** via `@ship/sail-highlight`
(`tree-sitter-sail.wasm` + `queries/highlights.scm`) as VS Code semantic
tokens. This client does not ship a TextMate grammar and does not teach
retired syntax.

## Prerequisite

The editor process must be able to spawn:

```sh
sail lsp
```

Install the CLI from this repo (`sail lsp` is a peer on that binary):

```sh
cargo install --path crates/deck/deck-cli --bin sail
```

Confirm with `sail lsp --help` or by opening a `.sail` file after install.
Override the executable with `sail.lsp.path` (absolute path to `sail`). The
client always appends `lsp`. It never runs `cargo run`.

## Install from Location (unpacked)

From this directory:

```sh
cd packages/sail/vscode
bun install
bun run compile
```

`compile` copies wasm + `highlights.scm` into `vendor/` so the highlighter
loads from `context.extensionUri`.

Then in **Cursor** or **VS Code**:

1. Command Palette → **Extensions: Install from Location…**
2. Choose this folder (`packages/sail/vscode`).

If that command is missing, symlink the folder into the editor’s extensions
directory and reload:

```sh
ln -s "$(pwd)" "$HOME/.cursor/extensions/ship.sail-0.1.0"
# VS Code: ~/.vscode/extensions/ship.sail-0.1.0
```

Open a `*.sail` file. The language mode is **Sail**. Output channel
**Sail Language Server** is the stdio session.

## Install from VSIX (local only)

```sh
cd packages/sail/vscode
bun install
bun run compile
bunx @vscode/vsce package --no-dependencies --allow-missing-repository --skip-license
```

Command Palette → **Extensions: Install from VSIX…** and pick the generated
`.vsix`. Do **not** `vsce publish`.

## Settings

| Setting | Default | Meaning |
|---|---|---|
| `sail.lsp.path` | empty (`sail` on PATH) | Path to the `sail` executable. Client runs `<path> lsp`. |

## Activation

`onLanguage:sail` — the client starts when you open a `.sail` file.
