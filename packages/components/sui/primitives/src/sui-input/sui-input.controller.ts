import { bindAttr, bindNativeEvent, controller, onCleanup } from '@ship/sui-runtime'
import type { Scope } from './sui-input.sui'

let _idSeq = 0

export default controller<Scope>({
  mount(state, host) {
    const inputEl = state.$signals.inputEl()
    if (!inputEl) return
    host.expose!({
      focus: () => inputEl.focus(),
      blur: () => inputEl.blur(),
      clear: () => { inputEl.value = ''; state.$signals.value.set('') },
      reportValidity: () => inputEl.reportValidity(),
    })
    bindNativeEvent(inputEl, 'input', () =>
      host.emit!('sui-input', { value: inputEl.value }))
    bindNativeEvent(inputEl, 'change', () =>
      host.emit!('sui-input-change', { value: inputEl.value }))
    const errId = `sui-input-err-${++_idSeq}`
    onCleanup(bindAttr(inputEl, 'aria-describedby', () => {
      const parts: string[] = []
      if (state.$signals.invalid() && state.$signals.errorMessage()) {
        const errorEl = state.$signals.errorEl()
        if (errorEl) { errorEl.id = errId; parts.push(errId) }
      }
      const extra = state.$signals.describedBy()
      if (extra) parts.push(extra)
      return parts.length > 0 ? parts.join(' ') : null
    }))
  },
})
