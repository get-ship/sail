import { controller, effect } from '@ship/sui-runtime'
import type { Scope } from './sui-brand-mark.sui'

export default controller<Scope>({
  mount(state, host) {
    const sig = state.$signals

    const stopAria = effect(() => {
      const label = sig.label()
      if (label) {
        host.setAttribute('role', 'img')
        host.setAttribute('aria-label', label)
        host.removeAttribute('aria-hidden')
      } else {
        host.setAttribute('aria-hidden', 'true')
        host.removeAttribute('role')
        host.removeAttribute('aria-label')
      }
    })

    return () => stopAria()
  },
})
