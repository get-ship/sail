// /deck/rendering — adapted from packages/deck/docs/routes/rendering.
import { controller, type ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'
import { ssrConfig, buildCmd } from './snippets'

export default controller({
  scope: (_ctx: ViewScope<Scope>) => ({ ssrConfig, buildCmd }),
})
