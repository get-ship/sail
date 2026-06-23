import { controller, effect } from '@ship/sui-runtime'
import type { Scope } from './sui-textarea.sui'

export default controller<Scope>({
  mount(state, host) {
    let blurValue = state.$signals.value()

    const stopListeners = effect(() => {
      const el = state.$signals.textareaEl()
      if (!el) return

      const onNativeInput = () => {
        host.dispatchEvent(new CustomEvent('sui-input', {
          detail: { value: el.value },
          bubbles: true,
          composed: true,
        }))
      }

      const onNativeBlur = () => {
        const current = state.$signals.value()
        if (current !== blurValue) {
          blurValue = current
          host.dispatchEvent(new CustomEvent('sui-change', {
            detail: { value: current },
            bubbles: true,
            composed: true,
          }))
        }
      }

      el.addEventListener('input', onNativeInput)
      el.addEventListener('blur', onNativeBlur)
      return () => {
        el.removeEventListener('input', onNativeInput)
        el.removeEventListener('blur', onNativeBlur)
      }
    })

    return () => stopListeners()
  },
})
