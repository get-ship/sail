import { controller, effect } from '@ship/sui-runtime'
import type { Scope } from './sui-accordion-item.sui'

let _seq = 0

export default controller<Scope>({
  mount(state, host) {
    const uid = ++_seq
    const triggerId = `sui-acc-trigger-${uid}`
    const contentId = `sui-acc-content-${uid}`

    const stopIds = effect(() => {
      const btn = state.$signals.triggerBtn() as HTMLButtonElement | null
      const wrap = state.$signals.contentWrap() as HTMLElement | null
      if (btn) {
        btn.id = triggerId
        btn.setAttribute('aria-controls', contentId)
      }
      if (wrap) {
        wrap.id = contentId
        wrap.setAttribute('aria-labelledby', triggerId)
      }
    })

    const stopAriaExpanded = effect(() => {
      const btn = state.$signals.triggerBtn() as HTMLButtonElement | null
      if (btn) btn.setAttribute('aria-expanded', state.$signals.open() ? 'true' : 'false')
    })

    const onClick = () => {
      if (state.$signals.disabled()) return
      host.dispatchEvent(
        new CustomEvent('sui-accordion-item-toggle', {
          detail: { value: state.$signals.value() },
          bubbles: true,
          composed: true,
        }),
      )
    }

    const onKeydown = (e: KeyboardEvent) => {
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

    const stopBtn = effect(() => {
      const btn = state.$signals.triggerBtn() as HTMLButtonElement | null
      if (!btn) return
      btn.addEventListener('click', onClick)
      btn.addEventListener('keydown', onKeydown)
      return () => {
        btn.removeEventListener('click', onClick)
        btn.removeEventListener('keydown', onKeydown)
      }
    })

    return () => {
      stopIds()
      stopAriaExpanded()
      stopBtn()
    }
  },
})
