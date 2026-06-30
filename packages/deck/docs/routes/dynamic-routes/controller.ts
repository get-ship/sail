import { controller, type ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'
import { routeExample, readExample } from './snippets'

export default controller({
  scope: (_ctx: ViewScope<Scope>) => ({ routeExample, readExample }),
})
