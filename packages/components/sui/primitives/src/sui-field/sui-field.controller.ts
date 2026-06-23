import { controller, effect } from '@ship/sui-runtime'
import type { Scope } from './sui-field.sui'

let _idSeq = 0

export default controller<Scope>({
  mount(state, host) {
    const hintId = `sui-field-hint-${++_idSeq}`
    const errorId = `sui-field-error-${++_idSeq}`

    // Assign stable IDs to hint and error containers.
    const stopIds = effect(() => {
      const hint = state.$signals.hintEl()
      const error = state.$signals.errorEl()
      if (hint) hint.id = hintId
      if (error) error.id = errorId
    })

    const wireControl = () => {
      const slot = state.$signals.controlSlot() as HTMLSlotElement | null
      const first = slot?.assignedElements({ flatten: true })[0] as HTMLElement | undefined
      if (!first) return
      const inv = state.$signals.invalid()
      const targetId = inv ? errorId : hintId
      const prior = (first.getAttribute('aria-describedby') || '')
        .split(/\s+/)
        .filter(id => id && id !== hintId && id !== errorId)
      const final = [...prior, targetId].join(' ').trim()
      if (final) first.setAttribute('aria-describedby', final)
      if ('invalid' in first) (first as unknown as { invalid: boolean }).invalid = inv
      else first.setAttribute('aria-invalid', inv ? 'true' : 'false')
    }

    // Re-wire whenever invalid changes.
    const stopInvalid = effect(() => {
      state.$signals.invalid() // track dependency
      wireControl()
    })

    // Re-wire when slotted elements change.
    const stopSlot = effect(() => {
      const slot = state.$signals.controlSlot() as HTMLSlotElement | null
      if (!slot) return
      slot.addEventListener('slotchange', wireControl)
      return () => slot.removeEventListener('slotchange', wireControl)
    })

    return () => {
      stopIds()
      stopInvalid()
      stopSlot()
    }
  },
})
