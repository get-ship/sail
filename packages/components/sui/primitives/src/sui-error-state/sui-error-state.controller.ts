import { controller, effect } from '@ship/sui-runtime'
import type { Scope } from './sui-error-state.sui'

export default controller<Scope>({
  mount(state, host) {
    let detailsOpen = false

    const updateToggleLabel = (btn: HTMLButtonElement) => {
      btn.textContent = detailsOpen ? 'Hide details' : 'Show details'
      btn.setAttribute('aria-expanded', detailsOpen ? 'true' : 'false')
    }

    const stopDetailsSlot = effect(() => {
      const slot = state.$signals.detailsSlot() as HTMLSlotElement | null
      if (!slot) return
      const check = () => {
        const hasContent = slot.assignedNodes({ flatten: true }).some(n =>
          n.nodeType === Node.ELEMENT_NODE ||
          (n.nodeType === Node.TEXT_NODE && (n.textContent ?? '').trim().length > 0),
        )
        if (hasContent) {
          host.setAttribute('data-has-details', '')
        } else {
          host.removeAttribute('data-has-details')
          host.removeAttribute('data-details-open')
          detailsOpen = false
          const btn = state.$signals.detailsToggleBtn() as HTMLButtonElement | null
          if (btn) updateToggleLabel(btn)
        }
      }
      slot.addEventListener('slotchange', check)
      check()
      return () => slot.removeEventListener('slotchange', check)
    })

    const stopToggleInit = effect(() => {
      const btn = state.$signals.detailsToggleBtn() as HTMLButtonElement | null
      if (!btn) return
      updateToggleLabel(btn)
      const onClick = () => {
        detailsOpen = !detailsOpen
        if (detailsOpen) {
          host.setAttribute('data-details-open', '')
        } else {
          host.removeAttribute('data-details-open')
        }
        updateToggleLabel(btn)
      }
      btn.addEventListener('click', onClick)
      return () => btn.removeEventListener('click', onClick)
    })

    const stopActionSlot = effect(() => {
      const slot = state.$signals.actionSlot() as HTMLSlotElement | null
      if (!slot) return
      const onSlotChange = () => {
        const retry = slot.assignedElements().find(
          el => el.getAttribute('slot') === 'retry',
        ) as HTMLElement | undefined
        if (retry) requestAnimationFrame(() => retry.focus?.())
      }
      slot.addEventListener('slotchange', onSlotChange)
      return () => slot.removeEventListener('slotchange', onSlotChange)
    })

    return () => {
      stopDetailsSlot()
      stopToggleInit()
      stopActionSlot()
    }
  },
})
