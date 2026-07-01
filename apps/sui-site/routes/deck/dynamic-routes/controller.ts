// /deck/dynamic-routes — adapted from packages/deck/docs/routes/dynamic-routes.
import { controller, type ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'
import { routeExample, readExample } from './snippets'

export default controller({
  scope: (_ctx: ViewScope<Scope>) => ({ routeExample, readExample }),
})
