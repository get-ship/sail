import { controller, effect } from '@ship/sui-runtime'
import type { Scope } from './sui-chip.sui'

export default controller<Scope>({
  mount(state, host: HTMLElement) {

    if (!host.hasAttribute('role')) host.setAttribute('role', 'button')
    if (!host.hasAttribute('tabindex') && !state.disabled) {
      host.setAttribute('tabindex', '0')
    }

    const stopSync = effect(() => {
      const sel = state.selected
      const dis = state.disabled
      host.setAttribute('aria-pressed', sel ? 'true' : 'false')
      if (dis) {
        host.setAttribute('aria-disabled', 'true')
        if (host.getAttribute('tabindex') !== '-1') host.setAttribute('tabindex', '-1')
      } else {
        host.removeAttribute('aria-disabled')
        if (host.getAttribute('tabindex') === '-1') host.setAttribute('tabindex', '0')
      }
    })

    const onKeydown = (e: KeyboardEvent) => {
      if (state.disabled) return
      if (e.key === 'Enter' || e.key === ' ') {
        e.preventDefault()
        host.click()
      } else if (state.removable && (e.key === 'Backspace' || e.key === 'Delete')) {
        e.preventDefault()
        host.dispatchEvent(new CustomEvent('ship-remove', { detail: {}, bubbles: true, composed: true }))
      }
    }
    host.addEventListener('keydown', onKeydown)

    const stopRemoveBtn = effect(() => {
      const btn = state.removeBtn as HTMLElement | null
      if (!btn) return
      const onClick = (e: MouseEvent) => {
        e.stopPropagation()
        if (state.disabled) return
        host.dispatchEvent(new CustomEvent('ship-remove', { detail: {}, bubbles: true, composed: true }))
      }
      btn.addEventListener('click', onClick)
      return () => btn.removeEventListener('click', onClick)
    })

    return () => {
      stopSync()
      host.removeEventListener('keydown', onKeydown)
      stopRemoveBtn()
    }
  },
})
