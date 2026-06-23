import { controller, derived, effect, ref, signal } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './sui-combobox.sui'

interface OptionRecord { value: string; label: string }
interface VisibleOption extends OptionRecord { id: string; idx: number }

let listIdSeq = 0

const parseJsonOptions = (raw: string): OptionRecord[] => {
  if (!raw) return []
  try {
    const parsed = JSON.parse(raw) as unknown
    if (!Array.isArray(parsed)) return []
    return parsed
      .map((entry) => {
        if (entry && typeof entry === 'object') {
          const o = entry as { value?: unknown; label?: unknown }
          const value = o.value != null ? String(o.value) : ''
          const label = o.label != null ? String(o.label) : value
          return value ? { value, label } : null
        }
        if (typeof entry === 'string') return { value: entry, label: entry }
        return null
      })
      .filter((o): o is OptionRecord => o !== null)
  } catch {
    return []
  }
}

const readLightOptions = (host: HTMLElement): OptionRecord[] => {
  const els = Array.from(host.querySelectorAll(':scope > option')) as HTMLOptionElement[]
  return els.map((el) => ({
    value: el.value || el.textContent?.trim() || '',
    label: el.textContent?.trim() || el.value || '',
  })).filter((o) => o.value !== '')
}

export default controller({
  scope: ({ options }: ViewScope<Scope>) => {
    const open = signal(false)
    const filterText = signal('')
    const highlight = signal(0)
    const slotOptions = signal<OptionRecord[]>([])
    const inputEl = ref<HTMLInputElement>(null)
    const listEl = ref<HTMLUListElement>(null)
    const optionsSlot = ref<HTMLSlotElement>(null)
    const listId = signal(`sui-combobox-list-${++listIdSeq}`)

    const parsedOptions = derived<OptionRecord[]>(() => {
      const fromAttr = parseJsonOptions(options())
      if (fromAttr.length > 0) return fromAttr
      return slotOptions()
    })
    const visibleOptions = derived<VisibleOption[]>(() => {
      const q = filterText().trim().toLowerCase()
      const all = parsedOptions()
      const matched = q
        ? all.filter((o) => o.label.toLowerCase().includes(q))
        : all
      return matched.map((o, idx) => ({ ...o, idx, id: `sui-combobox-opt-${idx}` }))
    })
    const activeId = derived<string>(() => {
      if (!open()) return ''
      const list = visibleOptions()
      const h = highlight()
      return list[h]?.id ?? ''
    })

    return {
      open, filterText, highlight, slotOptions,
      inputEl, listEl, optionsSlot, listId,
      parsedOptions, visibleOptions, activeId,
    }
  },

  mount(state, host) {
    let lastEmitted: string | null = null

    const refreshSlotOptions = () => {
      state.slotOptions = readLightOptions(host)
    }
    refreshSlotOptions()
    const slot = state.optionsSlot as HTMLSlotElement | null
    slot?.addEventListener('slotchange', refreshSlotOptions)

    const labelFor = (value: string): string => {
      const opts = state.parsedOptions as OptionRecord[]
      return opts.find((o) => o.value === value)?.label ?? value
    }

    // value → input text. When closed, the input mirrors the selected
    // option's label. While typing, the user's filter takes over (see
    // onInput below).
    const stopValueSync = effect(() => {
      const v = state.value
      if (!state.open) state.filterText = labelFor(v as string)
    })

    // Initial sync of input element after first paint.
    state.filterText = labelFor(state.value as string)

    // Reflect open state to a host attribute so the chevron CSS can flip.
    const stopOpenAttr = effect(() => {
      if (state.open) host.setAttribute('data-open', '')
      else host.removeAttribute('data-open')
    })

    const inputEl = state.inputEl as HTMLInputElement | null
    if (!inputEl) return

    const openList = () => {
      if (state.disabled) return
      if (!state.open) {
        state.open = true
        state.highlight = 0
        // On open, clear filter so the user sees the full list.
        state.filterText = ''
      }
    }

    const closeList = (commitLabelFromValue = true) => {
      if (!state.open) return
      state.open = false
      if (commitLabelFromValue) state.filterText = labelFor(state.value as string)
    }

    const commit = (rec: OptionRecord | undefined) => {
      if (!rec) return
      state.value = rec.value
      state.open = false
      state.filterText = rec.label
      if (lastEmitted !== rec.value) {
        lastEmitted = rec.value
        host.dispatchEvent(
          new CustomEvent('sui-change', {
            detail: { value: rec.value, label: rec.label },
            bubbles: true,
            composed: true,
          }),
        )
      }
    }

    const onFocus = () => openList()
    const onClick = () => openList()
    const onInput = () => {
      if (!state.open) state.open = true
      state.highlight = 0
    }
    const onKeydown = (e: KeyboardEvent) => {
      const list = state.visibleOptions as VisibleOption[]
      if (e.key === 'ArrowDown') {
        e.preventDefault()
        if (!state.open) { openList(); return }
        if (list.length === 0) return
        state.highlight = (state.highlight + 1) % list.length
      } else if (e.key === 'ArrowUp') {
        e.preventDefault()
        if (!state.open) { openList(); return }
        if (list.length === 0) return
        const h = state.highlight
        state.highlight = h <= 0 ? list.length - 1 : h - 1
      } else if (e.key === 'Enter') {
        if (!state.open) return
        e.preventDefault()
        const sel = list[state.highlight]
        if (sel) commit(sel)
      } else if (e.key === 'Escape') {
        if (!state.open) return
        e.preventDefault()
        closeList(true)
      } else if (e.key === 'Tab') {
        closeList(true)
      }
    }
    const onBlur = (e: FocusEvent) => {
      // Don't close when the new focus target is inside our shadow tree
      // (e.g. clicking a listbox row before mousedown→click completes).
      const next = e.relatedTarget as Node | null
      if (next && host.shadowRoot?.contains(next)) return
      setTimeout(() => closeList(true), 0)
    }

    inputEl.addEventListener('focus', onFocus)
    inputEl.addEventListener('click', onClick)
    inputEl.addEventListener('input', onInput)
    inputEl.addEventListener('keydown', onKeydown)
    inputEl.addEventListener('blur', onBlur)

    // Mousedown on a listbox row commits before blur fires.
    const onShadowMousedown = (e: MouseEvent) => {
      const target = e.target as HTMLElement | null
      const li = target?.closest?.('li.opt[data-value]') as HTMLElement | null
      if (!li) return
      e.preventDefault()
      const value = li.dataset.value ?? ''
      const list = state.visibleOptions as VisibleOption[]
      commit(list.find((o) => o.value === value))
    }
    host.shadowRoot?.addEventListener('mousedown', onShadowMousedown as EventListener)

    host.expose!({
      focus: () => inputEl.focus(),
      blur: () => inputEl.blur(),
      open: openList,
      close: () => closeList(true),
    })

    return () => {
      stopValueSync()
      stopOpenAttr()
      slot?.removeEventListener('slotchange', refreshSlotOptions)
      inputEl.removeEventListener('focus', onFocus)
      inputEl.removeEventListener('click', onClick)
      inputEl.removeEventListener('input', onInput)
      inputEl.removeEventListener('keydown', onKeydown)
      inputEl.removeEventListener('blur', onBlur)
      host.shadowRoot?.removeEventListener('mousedown', onShadowMousedown as EventListener)
    }
  },
})
