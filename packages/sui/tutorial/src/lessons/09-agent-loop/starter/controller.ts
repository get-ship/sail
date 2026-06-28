import { controller, signal, derived } from '@ship/sui-runtime'
import type { Scope } from './template.sui'

export default controller<Scope>({
  scope: () => {
    const rate = signal(2.34)
    // Renamed from `oldRate` to `prevRate` but the template still reads oldRate().
    const prevRate = signal(0)

    return {
      rate, prevRate,
      init: (): void => rate.set(1.0),
    }
  },
})
