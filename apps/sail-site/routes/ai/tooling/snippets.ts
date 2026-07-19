// Code snippets for the /ai/tooling page.

export const lintCmd = `cargo run -p sail-lint --example check -- views/tasks/page.sail`

export const lintOutput = `views/tasks/page.sail:12:4 error: \`new\` is not allowed in renderable expressions [expr-subset]
views/tasks/page.sail:18:9 warning: {#for} block is missing a \`key={EXPR}\` [sail-for-key]`

export const compileCmd = `cargo run -p sail-compiler -- build views/tasks/page.sail`

export const lspJson = `{ "sail": { "command": "sail-lsp", "extensionToLanguage": { ".sail": "sail" } } }`
