// Code snippets for the /ai/tooling page.

export const lintCmd = `cargo run -p sui-lint --example check -- views/tasks/page.sui`

export const lintOutput = `views/tasks/page.sui:12:4 error: \`new\` is not allowed in renderable expressions [expr-subset]
views/tasks/page.sui:18:9 warning: <data> directive without \`name\` attr [sui-data-name]`

export const compileCmd = `cargo run -p sui-compiler -- build views/tasks/page.sui`

export const lspJson = `{ "sui": { "command": "sui-lsp", "extensionToLanguage": { ".sui": "sui" } } }`
