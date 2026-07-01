import { controller, signal, derived } from '@ship/sui-runtime'
import type { Scope } from './template.sui'

export default controller<Scope>({
  scope: () => {
    const rate = signal(2.34)
    const prevRate = signal(0)
    const query = signal('')

    return {
      rate, prevRate, query,
      init: (): void => {
        prevRate.set(rate())
        rate.set(1.0)
      },
    }
  },
})
