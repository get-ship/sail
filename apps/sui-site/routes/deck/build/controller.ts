// /deck/build — adapted from packages/deck/docs/routes/build.
import { controller, type ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'
import { buildCmd, buildOut, baseCmd } from './snippets'

export default controller({
  scope: (_ctx: ViewScope<Scope>) => ({ buildCmd, buildOut, baseCmd }),
})
