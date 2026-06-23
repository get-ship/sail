import { controller, effect } from '@ship/sui-runtime'
import type { Scope } from './sui-switch.sui'

export default controller<Scope>({
  mount(state, host) {

    // Dispatch sui-change after bind:checked has updated the signal.
    const stopChange = effect(() => {
      const el = state.$signals.inputEl()
      if (!el) return
      const onNativeChange = () => {
        host.dispatchEvent(new CustomEvent('sui-change', {
          detail: { checked: el.checked },
          bubbles: true,
          composed: true,
        }))
      }
      el.addEventListener('change', onNativeChange)
      return () => el.removeEventListener('change', onNativeChange)
    })

    return () => stopChange()
  },
})
