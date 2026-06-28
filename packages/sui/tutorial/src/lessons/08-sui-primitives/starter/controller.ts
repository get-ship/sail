import { controller, signal, derived } from '@ship/sui-runtime'
import type { Scope } from './template.sui'

export default controller<Scope>({
  scope: () => {
    const errorRate = signal(2.3)
    const rateText = derived(() => errorRate().toFixed(1))

    return {
      errorRate, rateText,
      bumpRate: (n: number): void => errorRate.set(Math.max(0, errorRate() + n)),
      reset: (): void => errorRate.set(0),
    }
  },
})
