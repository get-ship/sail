// Code snippets for the /docs/ai/tooling page.

export const lintCmd = `cargo run -p sail-lint --example check -- routes/tasks/page.sail`

export const lintOutput = `routes/tasks/page.sail:12:4 error: \`new\` is not allowed in renderable expressions [expr-subset]
routes/tasks/page.sail:18:9 warning: {#for} block is missing a \`key={EXPR}\` [sail-for-key]`

export const compileCmd = `cargo run -p sail-compiler -- build routes/tasks/page.sail`

export const lspJson = `{ "sail": { "command": "sail-lsp", "extensionToLanguage": { ".sail": "sail" } } }`
