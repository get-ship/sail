import { controller, effect } from '@ship/sui-runtime'
import type { Scope } from './sui-collapsible.sui'

export default controller<Scope>({
  mount(state, host) {
    const stopOpenSync = effect(() => {
      const details = state.$signals.detailsEl() as HTMLDetailsElement | null
      const isOpen = state.$signals.open()
      if (details && details.open !== isOpen) details.open = isOpen
    })

    const onToggle = (e: Event) => {
      const details = e.target as HTMLDetailsElement
      const isOpen = details.open
      if (state.$signals.open() === isOpen) return
      state.$signals.open.set(isOpen)
      host.dispatchEvent(
        new CustomEvent('sui-toggle', {
          detail: { open: isOpen },
          bubbles: true,
          composed: true,
        }),
      )
    }

    // <details> has no native `disabled` state — the one bit of JS
    // native HTML can't express. Block activation ourselves.
    const onClickCapture = (e: MouseEvent) => {
      if (state.$signals.disabled()) e.preventDefault()
    }
    const onKeydownCapture = (e: KeyboardEvent) => {
      if (state.$signals.disabled() && (e.key === 'Enter' || e.key === ' ')) e.preventDefault()
    }

    const stopListeners = effect(() => {
      const details = state.$signals.detailsEl() as HTMLDetailsElement | null
      if (!details) return
      details.addEventListener('toggle', onToggle)
      details.addEventListener('click', onClickCapture, true)
      details.addEventListener('keydown', onKeydownCapture, true)
      return () => {
        details.removeEventListener('toggle', onToggle)
        details.removeEventListener('click', onClickCapture, true)
        details.removeEventListener('keydown', onKeydownCapture, true)
      }
    })

    return () => {
      stopOpenSync()
      stopListeners()
    }
  },
})
