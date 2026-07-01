// /deck/hmr — adapted from packages/deck/docs/routes/hmr.
import { controller, type ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'
import { devCmd } from './snippets'

export default controller({
  scope: (_ctx: ViewScope<Scope>) => ({ devCmd }),
})
