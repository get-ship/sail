// Build & deploy controller. Snippets live in ./snippets (a plain data module)
// so example code containing `{ ... }` is never scope-scanned by the compiler.
import { controller, type ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'
import { buildCmd, buildOut, baseCmd } from './snippets'

export default controller({
  scope: (_ctx: ViewScope<Scope>) => ({ buildCmd, buildOut, baseCmd }),
})
