import { controller, signal, derived } from '@ship/sui-runtime'
import type { Scope } from './template.sui'

export default controller<Scope>({
  scope: () => {
    const count = signal(0)
    const label = derived(() => count() === 0 ? 'draft' : count() > 5 ? 'over' : 'active')
    return {
      count, label,
      bump: (n: number): void => count.set(count() + n),
    }
  },
})
