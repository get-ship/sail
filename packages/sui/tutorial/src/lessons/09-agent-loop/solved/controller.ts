import { controller, signal, derived } from '@ship/sui-runtime'
import type { Scope } from './template.sui'

export default controller<Scope>({
  scope: () => {
    const rate = signal(2.34)
    const prevRate = signal(0)
    const query = signal('')
    const rateText = derived(() => rate().toFixed(2))

    return {
      rate, prevRate, query, rateText,
      init: (): void => {
        prevRate.set(rate())
        rate.set(1.0)
      },
    }
  },
})
