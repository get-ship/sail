// /deck/data — adapted from packages/deck/docs/routes/data. The live `visits`
// demo is omitted here because the sui-site doesn't expose a `store` client
// (its clients seam is intentionally empty). The config + usage snippets
// still document the seam.
import { controller, type ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'
import { configExample, usageExample } from './snippets'

export default controller({
  scope: (_ctx: ViewScope<Scope>) => ({ configExample, usageExample }),
})
