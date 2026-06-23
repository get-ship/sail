import { controller, effect } from '@ship/sui-runtime'
import type { Scope } from './sui-toggle-group.sui'

type ToggleEl = HTMLElement & { focusButton?(): void }

export default controller<Scope>({
  mount(state, host) {

    if (!host.hasAttribute('role')) host.setAttribute('role', 'group')

    const stopAriaOrientation = effect(() => {
      host.setAttribute('aria-orientation', state.$signals.orientation() as string)
    })

    // Light-DOM hidden input for form participation.
    const hiddenInput = document.createElement('input')
    hiddenInput.type = 'hidden'
    host.appendChild(hiddenInput)

    const getToggles = (): ToggleEl[] => {
      const slot = state.$signals.defaultSlotEl() as HTMLSlotElement | null
      if (!slot) return Array.from(host.querySelectorAll(':scope > sui-toggle')) as ToggleEl[]
      return slot.assignedElements()
        .filter(el => el.tagName.toLowerCase() === 'sui-toggle') as ToggleEl[]
    }

    const enabledToggles = (): ToggleEl[] => getToggles().filter(t => !t.hasAttribute('disabled'))

    const getSelectedValues = (): Set<string> =>
      new Set((state.$signals.value() as string).split(',').filter(Boolean))

    const setSelectedValues = (vals: Set<string>) => {
      const joined = Array.from(vals).join(',')
      state.$signals.value.set(joined)
      syncToggles()
      hiddenInput.name = state.$signals.name() as string
      hiddenInput.value = joined
      host.dispatchEvent(new CustomEvent('sui-change', {
        detail: { value: joined },
        bubbles: true,
        composed: true,
      }))
    }

    const syncToggles = () => {
      const selected = getSelectedValues()
      const size = state.$signals.size() as string
      const variant = state.$signals.variant() as string
      for (const toggle of getToggles()) {
        const val = toggle.getAttribute('value') ?? ''
        toggle.toggleAttribute('pressed', selected.has(val))
        if (size) toggle.setAttribute('size', size)
        if (variant) toggle.setAttribute('variant', variant)
      }
    }

    const onToggle = (e: Event) => {
      const ce = e as CustomEvent
      ce.stopPropagation()
      const toggle = (ce.composedPath() as Element[]).find(
        el => el.tagName?.toLowerCase() === 'sui-toggle',
      ) as ToggleEl | null
      if (!toggle) return
      const val = toggle.getAttribute('value') ?? ''
      const selected = getSelectedValues()

      if ((state.$signals.type() as string) === 'single') {
        if (selected.has(val)) setSelectedValues(new Set())
        else setSelectedValues(new Set([val]))
      } else {
        if (selected.has(val)) selected.delete(val)
        else selected.add(val)
        setSelectedValues(selected)
      }
    }

    const onKeydown = (e: KeyboardEvent) => {
      const vertical = (state.$signals.orientation() as string) === 'vertical'
      const nextKey = vertical ? 'ArrowDown' : 'ArrowRight'
      const prevKey = vertical ? 'ArrowUp' : 'ArrowLeft'
      if (e.key !== nextKey && e.key !== prevKey) return
      const items = enabledToggles()
      if (!items.length) return
      const current = (e.composedPath() as Element[]).find(
        el => items.includes(el as ToggleEl),
      ) as ToggleEl | null
      const i = current ? items.indexOf(current) : -1
      const next = e.key === nextKey
        ? (i + 1) % items.length
        : (i - 1 + items.length) % items.length
      items[next]?.focusButton?.()
      e.preventDefault()
    }

    const stopSync = effect(() => {
      state.$signals.value()
      state.$signals.size()
      state.$signals.variant()
      syncToggles()
    })

    const stopSlot = effect(() => {
      const slot = state.$signals.defaultSlotEl() as HTMLSlotElement | null
      if (!slot) return
      const onSlotChange = () => syncToggles()
      slot.addEventListener('slotchange', onSlotChange)
      return () => slot.removeEventListener('slotchange', onSlotChange)
    })

    const stopHidden = effect(() => {
      hiddenInput.name = state.$signals.name() as string
      hiddenInput.value = state.$signals.value() as string
    })

    host.addEventListener('sui-toggle', onToggle)
    host.addEventListener('keydown', onKeydown)

    return () => {
      stopSync()
      stopSlot()
      stopHidden()
      stopAriaOrientation()
      host.removeEventListener('sui-toggle', onToggle)
      host.removeEventListener('keydown', onKeydown)
      hiddenInput.remove()
    }
  },
})
