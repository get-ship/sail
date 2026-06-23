import { controller, effect } from '@ship/sui-runtime'
import type { Scope } from './sui-toggle.sui'

export default controller<Scope>({
  mount(state, host) {

    const stop = effect(() => {
      const btn = state.$signals.buttonEl() as HTMLButtonElement | null
      if (!btn) return
      const onClick = () => {
        if (state.$signals.disabled()) return
        state.$signals.pressed.set(!state.$signals.pressed())
        host.dispatchEvent(new CustomEvent('sui-toggle', {
          detail: { pressed: state.$signals.pressed(), value: state.$signals.value() },
          bubbles: true,
          composed: true,
        }))
      }
      btn.addEventListener('click', onClick)
      return () => btn.removeEventListener('click', onClick)
    })

    // Group calls this to move keyboard focus.
    ;(host as unknown as { focusButton(): void }).focusButton = () => {
      const btn = host.shadowRoot?.querySelector('button') as HTMLButtonElement | null
      btn?.focus()
    }

    return () => stop()
  },
})
