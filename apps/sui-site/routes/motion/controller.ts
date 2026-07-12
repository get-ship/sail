import { controller, signal, type ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'
import { code } from './snippets'

export default controller({
  scope: (_ctx: ViewScope<Scope>) => {
    return {
      code,
      open: signal(true),
      shown: signal(true),
      items: signal([1, 2, 3, 4, 5, 6, 7, 8]),
    }
  },
})
