import { controller, signal } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

export default controller<Scope>({
  scope: () => ({ count: signal(0) }),
})
