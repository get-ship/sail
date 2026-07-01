// /compare controller. No signals; static page.

import { controller, type ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

export default controller({
  scope: (_ctx: ViewScope<Scope>) => ({}),
})
