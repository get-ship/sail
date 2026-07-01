// /deck/layouts — adapted from packages/deck/docs/routes/layouts.
import { controller, type ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'
import { layoutFile, layoutChain } from './snippets'

export default controller({
  scope: (_ctx: ViewScope<Scope>) => ({ layoutFile, layoutChain }),
})
