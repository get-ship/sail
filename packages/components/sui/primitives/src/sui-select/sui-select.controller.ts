import { controller, effect } from '@ship/sui-runtime'
import type { Scope } from './sui-select.sui'

export default controller<Scope>({
  mount(state, host) {

    // Form association is wired by the compiler via `form-value` on the
    // value prop and `form-disabled` on the disabled prop in the .sui
    // file — the runtime calls setFormValue + installs reset / restore /
    // disabled callbacks before this mount runs. This controller only
    // owns the shadow-select mirror and option/optgroup cloning.

    const lightOptions = (): HTMLOptionElement[] =>
      Array.from(
        host.querySelectorAll(':scope > option, :scope > optgroup > option'),
      ) as HTMLOptionElement[]

    const currentLabel = (): string => {
      const v = state.$signals.value()
      const opts = lightOptions()
      if (opts.length === 0) return v
      const match =
        opts.find((o) => o.value === v) ??
        opts.find((o) => o.hasAttribute('selected')) ??
        opts[0]
      const text = (match?.textContent ?? '').trim()
      return text || match?.value || v
    }

    const updatePlaceholder = () => {
      const sel = state.$signals.selectEl() as HTMLSelectElement | null
      if (!sel) {
        const opts = lightOptions()
        const match =
          opts.find((o) => o.value === state.$signals.value()) ??
          opts.find((o) => o.hasAttribute('selected')) ??
          opts[0]
        if (!match || match.value === '') host.setAttribute('data-placeholder', '')
        else host.removeAttribute('data-placeholder')
        return
      }
      const opt = sel.selectedOptions?.[0] ?? sel.options[sel.selectedIndex]
      if (!opt || opt.value === '') host.setAttribute('data-placeholder', '')
      else host.removeAttribute('data-placeholder')
    }

    const syncOptions = () => {
      const sel = state.$signals.selectEl() as HTMLSelectElement | null
      if (!sel) return
      const sourceOpts = host.querySelectorAll(
        ':scope > option, :scope > optgroup',
      )
      sel.replaceChildren(...Array.from(sourceOpts, (o) => o.cloneNode(true)))
      sel.value = state.$signals.value()
    }

    const syncLabel = () => {
      const labelEl = state.$signals.valueLabel() as HTMLSpanElement | null
      if (labelEl) labelEl.textContent = currentLabel()
    }

    // Keep shadow <select>.value in sync with the prop signal and
    // refresh visible label + placeholder state on every change.
    // Covers programmatic writes (host.value = 'x') and attribute-driven
    // updates uniformly.
    const stopValueSync = effect(() => {
      const v = state.$signals.value()
      const sel = state.$signals.selectEl() as HTMLSelectElement | null
      if (sel) sel.value = v
      syncLabel()
      updatePlaceholder()
    })

    // Native change on the shadow <select> → write back to prop +
    // dispatch sui-change. The effect above will then re-run the
    // label/placeholder cascade and the compiler-installed
    // setFormValue effect picks up the new value.
    const onNativeChange = (e: Event) => {
      const v = (e.target as HTMLSelectElement).value
      state.$signals.value.set(v)
      host.dispatchEvent(
        new CustomEvent('sui-change', {
          detail: { value: v },
          bubbles: true,
          composed: true,
        }),
      )
    }

    const selectEl = state.$signals.selectEl() as HTMLSelectElement | null
    if (selectEl) selectEl.addEventListener('change', onNativeChange)

    // Light-DOM option/optgroup children → shadow select. Re-run on
    // every slotchange so consumers can add/remove options dynamically.
    const optionsSlot = state.$signals.optionsSlot() as HTMLSlotElement | null
    const onSlotChange = () => {
      syncOptions()
      syncLabel()
      updatePlaceholder()
    }
    if (optionsSlot) optionsSlot.addEventListener('slotchange', onSlotChange)

    // Initial pass — slotchange may or may not fire on first mount
    // depending on whether the slot's assignedNodes were ready, so we
    // do it explicitly.
    syncOptions()
    syncLabel()
    updatePlaceholder()

    return () => {
      if (selectEl) selectEl.removeEventListener('change', onNativeChange)
      if (optionsSlot) optionsSlot.removeEventListener('slotchange', onSlotChange)
      stopValueSync()
    }
  },
})
