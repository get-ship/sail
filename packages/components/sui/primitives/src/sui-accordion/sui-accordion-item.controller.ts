import { controller, effect } from '@ship/sui-runtime'
import type { Scope } from './sui-accordion-item.sui'

export default controller<Scope>({
  mount(state, host) {
    // Guards against the 'toggle' event our own open-prop sync causes
    // (native <details> fires 'toggle' whenever its `open` attribute
    // changes, whether the browser or we did it) from being mistaken
    // for a user-driven click that should ask the parent to coordinate.
    let suppressToggle = false

    const stopOpenSync = effect(() => {
      const details = state.$signals.detailsEl() as HTMLDetailsElement | null
      const isOpen = state.$signals.open()
      if (details && details.open !== isOpen) {
        suppressToggle = true
        details.open = isOpen
      }
    })

    const onToggle = () => {
      if (suppressToggle) {
        suppressToggle = false
        return
      }
      host.dispatchEvent(
        new CustomEvent('sui-accordion-item-toggle', {
          detail: { value: state.$signals.value() },
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
      if (state.$signals.disabled() && (e.key === 'Enter' || e.key === ' ')) {
        e.preventDefault()
        return
      }
      if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
        e.preventDefault()
        host.dispatchEvent(
          new CustomEvent('sui-accordion-nav', {
            detail: { direction: e.key === 'ArrowDown' ? 'next' : 'prev' },
            bubbles: true,
            composed: true,
          }),
        )
      } else if (e.key === 'Home' || e.key === 'End') {
        e.preventDefault()
        host.dispatchEvent(
          new CustomEvent('sui-accordion-nav', {
            detail: { direction: e.key === 'Home' ? 'first' : 'last' },
            bubbles: true,
            composed: true,
          }),
        )
      }
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
