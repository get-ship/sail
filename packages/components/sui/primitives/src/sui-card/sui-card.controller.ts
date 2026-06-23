import { controller, effect } from '@ship/sui-runtime'
import type { Scope } from './sui-card.sui'

export default controller<Scope>({
  mount(state, host) {

    const onKeydown = (e: KeyboardEvent) => {
      if (e.key === 'Enter' || e.key === ' ') {
        e.preventDefault()
        host.click()
      }
    }

    const stopSync = effect(() => {
      if (state.$signals.interactive()) {
        if (!host.hasAttribute('role')) host.setAttribute('role', 'button')
        if (!host.hasAttribute('tabindex')) host.setAttribute('tabindex', '0')
        host.addEventListener('keydown', onKeydown)
        return () => host.removeEventListener('keydown', onKeydown)
      } else {
        if (host.getAttribute('role') === 'button') host.removeAttribute('role')
        if (host.getAttribute('tabindex') === '0') host.removeAttribute('tabindex')
      }
    })

    return () => {
      stopSync()
    }
  },
})
