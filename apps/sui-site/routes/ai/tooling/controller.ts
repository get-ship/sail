// /ai/tooling — the LSP / CLI surface, adapted from the 10-lsp-cli tutorial
// lesson. Snippets live in ./snippets so literal braces (e.g. the
// `.lsp.json` payload) never leak into the inferred scope.
import { controller, type ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'
import { lintCmd, lintOutput, compileCmd, lspJson } from './snippets'

export default controller({
  scope: (_ctx: ViewScope<Scope>) => ({ lintCmd, lintOutput, compileCmd, lspJson }),
})
