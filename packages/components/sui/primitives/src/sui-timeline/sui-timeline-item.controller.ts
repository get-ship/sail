import { controller, effect } from '@ship/sui-runtime'
import type { Scope } from './sui-timeline-item.sui'

export default controller<Scope>({
  mount(state, host) {

    if (!host.hasAttribute('role')) host.setAttribute('role', 'listitem')

    const stopRole = effect(() => {
      if (state.$signals.interactive()) {
        host.setAttribute('role', 'button')
        if (!host.hasAttribute('tabindex')) host.setAttribute('tabindex', '0')
      } else {
        host.setAttribute('role', 'listitem')
        host.removeAttribute('tabindex')
      }
    })

    const emit = () => {
      host.dispatchEvent(
        new CustomEvent('sui-select', { bubbles: true, composed: true, detail: {} }),
      )
    }

    const onClick = () => { if (state.$signals.interactive()) emit() }

    const onKeydown = (e: KeyboardEvent) => {
      if (!state.$signals.interactive()) return
      if (e.key === 'Enter' || e.key === ' ') {
        e.preventDefault()
        emit()
      }
    }

    host.addEventListener('click', onClick)
    host.addEventListener('keydown', onKeydown)

    return () => {
      stopRole()
      host.removeEventListener('click', onClick)
      host.removeEventListener('keydown', onKeydown)
    }
  },
})
