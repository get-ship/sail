import { controller } from '@ship/sui-runtime'
import type { Scope } from './sui-timeline.sui'

export default controller<Scope>({
  mount(_state, host) {
    if (!host.hasAttribute('role')) host.setAttribute('role', 'list')
  },
})
