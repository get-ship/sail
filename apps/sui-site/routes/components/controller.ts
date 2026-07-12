import { controller, signal, type ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

export default controller({
  scope: (_ctx: ViewScope<Scope>) => {
    return {
      agreed: signal(true),
      volume: signal(60),
      name: signal(''),
      notify: signal(true),
    }
  },
})
