// /deck/routing — adapted from packages/deck/docs/routes/routing.
import { controller, type ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'
import { routeTree, colocated } from './snippets'

export default controller({
  scope: (_ctx: ViewScope<Scope>) => ({ routeTree, colocated }),
})
