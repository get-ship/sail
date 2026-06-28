import { controller, signal } from '@ship/sui-runtime'
import type { Scope } from './template.sui'

export default controller<Scope>({
  scope: () => {
    const count = signal(0)
    return {
      count,
      bump: (n: number): void => count.set(count() + n),
      reset: (): void => count.set(0),
    }
  },
})
