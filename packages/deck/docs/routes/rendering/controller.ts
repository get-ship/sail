// Rendering controller. Snippets live in ./snippets (a plain data module) so
// example code containing `{ ... }` is never scope-scanned by the compiler.
import { controller, type ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'
import { ssrConfig, buildCmd } from './snippets'

export default controller({
  scope: (_ctx: ViewScope<Scope>) => ({ ssrConfig, buildCmd }),
})
