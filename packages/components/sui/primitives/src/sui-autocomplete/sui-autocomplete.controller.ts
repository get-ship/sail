import { controller, derived, ref, signal } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './sui-autocomplete.sui'

interface Suggestion { value: string; label: string }
interface FilteredSuggestion extends Suggestion { id: string; idx: number }

let listIdSeq = 0

const parseJsonSuggestions = (raw: string): Suggestion[] => {
  if (!raw) return []
  try {
    const parsed = JSON.parse(raw) as unknown
    if (!Array.isArray(parsed)) return []
    return parsed
      .map((entry) => {
        if (typeof entry === 'string') return { value: entry, label: entry }
        if (entry && typeof entry === 'object') {
          const o = entry as { value?: unknown; label?: unknown }
          const value = o.value != null ? String(o.value) : ''
          const label = o.label != null ? String(o.label) : value
          return value ? { value, label } : null
        }
        return null
      })
      .filter((s): s is Suggestion => s !== null)
  } catch {
    return []
  }
}

const readLightSuggestions = (host: HTMLElement): Suggestion[] => {
  const els = Array.from(host.querySelectorAll(':scope > option')) as HTMLOptionElement[]
  return els.map((el) => ({
    value: el.value || el.textContent?.trim() || '',
    label: el.textContent?.trim() || el.value || '',
  })).filter((s) => s.value !== '')
}

export default controller({
  scope: ({ suggestions, value }: ViewScope<Scope>) => {
    const open = signal(false)
    const highlight = signal(0)
    const slotSuggestions = signal<Suggestion[]>([])
    const inputEl = ref<HTMLInputElement>(null)
    const listEl = ref<HTMLUListElement>(null)
    const suggestionsSlot = ref<HTMLSlotElement>(null)
    const listId = signal(`sui-autocomplete-list-${++listIdSeq}`)

    const parsedSuggestions = derived<Suggestion[]>(() => {
      const fromAttr = parseJsonSuggestions(suggestions())
      if (fromAttr.length > 0) return fromAttr
      return slotSuggestions()
    })
    const filtered = derived<FilteredSuggestion[]>(() => {
      const q = value().trim().toLowerCase()
      const all = parsedSuggestions()
      const matched = q
        ? all.filter((s) => s.value.toLowerCase().includes(q) || s.label.toLowerCase().includes(q))
        : all
      return matched.map((s, idx) => ({
        ...s,
        idx,
        id: `sui-autocomplete-opt-${idx}`,
      }))
    })
    const activeId = derived<string>(() => {
      if (!open()) return ''
      const list = filtered()
      const h = highlight()
      return list[h]?.id ?? ''
    })

    return {
      open, highlight, slotSuggestions,
      inputEl, listEl, suggestionsSlot, listId,
      parsedSuggestions, filtered, activeId,
    }
  },

  mount(state, host) {
    let lastEmitted: string | null = null

    const refreshSlotSuggestions = () => {
      state.slotSuggestions = readLightSuggestions(host)
    }
    refreshSlotSuggestions()
    const slot = state.suggestionsSlot as HTMLSlotElement | null
    slot?.addEventListener('slotchange', refreshSlotSuggestions)

    const inputEl = state.inputEl as HTMLInputElement | null
    if (!inputEl) return

    const openList = () => {
      if (state.disabled) return
      if (!state.open) {
        state.open = true
        state.highlight = 0
      }
    }

    const closeList = () => {
      if (!state.open) return
      state.open = false
    }

    const commit = (text: string, label?: string) => {
      state.value = text
      state.open = false
      if (lastEmitted !== text) {
        lastEmitted = text
        host.dispatchEvent(
          new CustomEvent('sui-change', {
            detail: { value: text, label: label ?? text },
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
      const list = state.filtered as FilteredSuggestion[]
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
        e.preventDefault()
        if (state.open && list.length > 0) {
          const sel = list[state.highlight]
          if (sel) { commit(sel.value, sel.label); return }
        }
        commit(state.value as string)
      } else if (e.key === 'Escape') {
        if (!state.open) return
        e.preventDefault()
        closeList()
      } else if (e.key === 'Tab') {
        closeList()
      }
    }
    const onBlur = (e: FocusEvent) => {
      const next = e.relatedTarget as Node | null
      if (next && host.shadowRoot?.contains(next)) return
      setTimeout(() => closeList(), 0)
    }

    inputEl.addEventListener('focus', onFocus)
    inputEl.addEventListener('click', onClick)
    inputEl.addEventListener('input', onInput)
    inputEl.addEventListener('keydown', onKeydown)
    inputEl.addEventListener('blur', onBlur)

    const onShadowMousedown = (e: MouseEvent) => {
      const target = e.target as HTMLElement | null
      const li = target?.closest?.('li.opt[data-value]') as HTMLElement | null
      if (!li) return
      e.preventDefault()
      const value = li.dataset.value ?? ''
      const list = state.filtered as FilteredSuggestion[]
      const match = list.find((s) => s.value === value)
      commit(value, match?.label)
    }
    host.shadowRoot?.addEventListener('mousedown', onShadowMousedown as EventListener)

    host.expose!({
      focus: () => inputEl.focus(),
      blur: () => inputEl.blur(),
      open: openList,
      close: () => closeList(),
    })

    return () => {
      slot?.removeEventListener('slotchange', refreshSlotSuggestions)
      inputEl.removeEventListener('focus', onFocus)
      inputEl.removeEventListener('click', onClick)
      inputEl.removeEventListener('input', onInput)
      inputEl.removeEventListener('keydown', onKeydown)
      inputEl.removeEventListener('blur', onBlur)
      host.shadowRoot?.removeEventListener('mousedown', onShadowMousedown as EventListener)
    }
  },
})
