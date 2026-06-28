import { controller, effect } from '@ship/sui-runtime'
import type { Scope } from './sui-label.sui'

export default controller<Scope>({
  mount(state, _host) {

    const stopFor = effect(() => {
      const labelEl = state.$signals.labelEl() as HTMLLabelElement | null
      if (!labelEl) return
      // bracket notation handles the reserved-word key without a cast
      labelEl.htmlFor = state.$signals['for']?.() ?? ''
    })

    return () => {
      stopFor()
    }
  },
})
