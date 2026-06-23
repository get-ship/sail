import { controller, effect } from '@ship/sui-runtime'
import type { Scope } from './sui-label.sui'

export default controller<Scope>({
  mount(state, _host) {

    const stopFor = effect(() => {
      const labelEl = state.$signals.labelEl() as HTMLLabelElement | null
      if (!labelEl) return
      // bracket notation avoids reserved-word TS parsing issues
      labelEl.htmlFor = (state.$signals as Record<string, () => string>)['for']?.() ?? ''
    })

    return () => {
      stopFor()
    }
  },
})
