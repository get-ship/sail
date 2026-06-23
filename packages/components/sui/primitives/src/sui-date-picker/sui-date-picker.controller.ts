import { controller, derived, ref, signal } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './sui-date-picker.sui'

interface DayCell {
  key: string
  iso: string
  day: number
  empty: boolean
  selected: boolean
  focused: boolean
  today: boolean
  disabled: boolean
  label: string
}
interface DayHeader { key: string; label: string }

const DOW = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat']
const DOW_SHORT = ['S', 'M', 'T', 'W', 'T', 'F', 'S']
const MONTHS = [
  'January', 'February', 'March', 'April', 'May', 'June',
  'July', 'August', 'September', 'October', 'November', 'December',
]

const pad2 = (n: number) => (n < 10 ? `0${n}` : String(n))
const isoFor = (year: number, month: number, day: number): string =>
  `${year}-${pad2(month + 1)}-${pad2(day)}`

const parseIso = (s: string): { y: number; m: number; d: number } | null => {
  const m = /^(\d{4})-(\d{2})-(\d{2})$/.exec(s)
  if (!m) return null
  const y = Number(m[1])
  const mo = Number(m[2]) - 1
  const d = Number(m[3])
  if (mo < 0 || mo > 11 || d < 1 || d > 31) return null
  return { y, m: mo, d }
}

const daysInMonth = (year: number, month: number): number =>
  new Date(year, month + 1, 0).getDate()

const todayIso = (): string => {
  const d = new Date()
  return isoFor(d.getFullYear(), d.getMonth(), d.getDate())
}

const addDays = (iso: string, delta: number): string => {
  const parts = parseIso(iso)
  if (!parts) return iso
  const d = new Date(parts.y, parts.m, parts.d + delta)
  return isoFor(d.getFullYear(), d.getMonth(), d.getDate())
}

const clampIso = (iso: string, min: string, max: string): string => {
  if (min && iso < min) return min
  if (max && iso > max) return max
  return iso
}

export default controller({
  scope: ({ value, min, max }: ViewScope<Scope>) => {
    const open = signal(false)
    const today = signal(todayIso())
    const focused = signal(todayIso())
    const viewYear = signal(new Date().getFullYear())
    const viewMonth = signal(new Date().getMonth())
    const inputEl = ref<HTMLInputElement>(null)
    const panelEl = ref<HTMLDivElement>(null)

    const monthLabel = derived<string>(() =>
      `${MONTHS[viewMonth()]} ${viewYear()}`,
    )
    const dayHeaders = derived<DayHeader[]>(() =>
      DOW.map((label, idx) => ({ key: String(idx), label: DOW_SHORT[idx] ?? label })),
    )
    const monthCells = derived<DayCell[]>(() => {
      const year = viewYear()
      const month = viewMonth()
      const sel = value()
      const foc = focused()
      const td = today()
      const mn = min()
      const mx = max()
      const firstDow = new Date(year, month, 1).getDay()
      const totalDays = daysInMonth(year, month)
      const cells: DayCell[] = []
      for (let i = 0; i < firstDow; i++) {
        cells.push({
          key: `e-${i}`, iso: '', day: 0,
          empty: true, selected: false, focused: false, today: false,
          disabled: false, label: '',
        })
      }
      for (let d = 1; d <= totalDays; d++) {
        const iso = isoFor(year, month, d)
        const dt = new Date(year, month, d)
        const dayName = DOW[dt.getDay()] ?? ''
        cells.push({
          key: iso,
          iso, day: d,
          empty: false,
          selected: sel === iso,
          focused: foc === iso,
          today: td === iso,
          disabled: Boolean((mn && iso < mn) || (mx && iso > mx)),
          label: `${dayName}, ${MONTHS[month]} ${d}, ${year}`,
        })
      }
      return cells
    })

    return { open, today, focused, viewYear, viewMonth, inputEl, panelEl, monthLabel, dayHeaders, monthCells }
  },

  mount(state, host) {
    const moveView = (delta: number) => {
      let y = state.viewYear
      let m = state.viewMonth + delta
      while (m < 0) { m += 12; y -= 1 }
      while (m > 11) { m -= 12; y += 1 }
      state.viewYear = y
      state.viewMonth = m
    }

    const syncViewToValue = () => {
      const seed = (state.value as string) || (state.focused as string) || todayIso()
      const parts = parseIso(seed)
      if (!parts) return
      state.viewYear = parts.y
      state.viewMonth = parts.m
      state.focused = seed
    }

    const commit = (iso: string) => {
      const parts = parseIso(iso)
      if (!parts) return
      if (state.min && iso < state.min) return
      if (state.max && iso > state.max) return
      state.value = iso
      state.focused = iso
      state.open = false
      host.dispatchEvent(
        new CustomEvent('sui-change', {
          detail: { value: iso },
          bubbles: true,
          composed: true,
        }),
      )
    }

    const openPanel = () => {
      if (state.disabled) return
      if (state.open) return
      syncViewToValue()
      state.open = true
    }

    const closePanel = () => {
      if (!state.open) return
      state.open = false
    }

    const inputEl = state.inputEl as HTMLInputElement | null
    if (!inputEl) return

    const onInputClick = () => openPanel()
    const onInputFocus = () => openPanel()
    const onInputChange = () => {
      // User typed an ISO date manually; commit if it parses.
      const v = inputEl.value.trim()
      if (v === '') {
        state.value = ''
        host.dispatchEvent(
          new CustomEvent('sui-change', {
            detail: { value: '' },
            bubbles: true,
            composed: true,
          }),
        )
        return
      }
      const parts = parseIso(v)
      if (parts) commit(v)
    }
    const onInputKeydown = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && state.open) {
        e.preventDefault()
        closePanel()
      } else if (e.key === 'ArrowDown' && !state.open) {
        e.preventDefault()
        openPanel()
      }
    }
    inputEl.addEventListener('click', onInputClick)
    inputEl.addEventListener('focus', onInputFocus)
    inputEl.addEventListener('change', onInputChange)
    inputEl.addEventListener('keydown', onInputKeydown)

    const onPanelKeydown = (e: KeyboardEvent) => {
      const cur = (state.focused as string) || (state.value as string) || todayIso()
      let next: string | null = null
      if (e.key === 'ArrowLeft') next = addDays(cur, -1)
      else if (e.key === 'ArrowRight') next = addDays(cur, 1)
      else if (e.key === 'ArrowUp') next = addDays(cur, -7)
      else if (e.key === 'ArrowDown') next = addDays(cur, 7)
      else if (e.key === 'PageUp') next = addDays(cur, -30)
      else if (e.key === 'PageDown') next = addDays(cur, 30)
      else if (e.key === 'Enter') { e.preventDefault(); commit(cur); return }
      else if (e.key === 'Escape') { e.preventDefault(); closePanel(); inputEl.focus(); return }
      if (next != null) {
        e.preventDefault()
        const clamped = clampIso(next, state.min as string, state.max as string)
        state.focused = clamped
        const parts = parseIso(clamped)
        if (parts) {
          state.viewYear = parts.y
          state.viewMonth = parts.m
        }
      }
    }

    const onShadowClick = (e: Event) => {
      const t = e.target as HTMLElement | null
      if (!t) return
      const navBtn = t.closest?.('button[data-act]') as HTMLElement | null
      if (navBtn) {
        const act = navBtn.dataset.act
        if (act === 'prev') moveView(-1)
        else if (act === 'next') moveView(1)
        return
      }
      const day = t.closest?.('button.day[data-iso]') as HTMLButtonElement | null
      if (day && !day.disabled) commit(day.dataset.iso ?? '')
    }

    const onShadowKeydown = (e: KeyboardEvent) => {
      if (!state.open) return
      const inPanel = (e.target as HTMLElement | null)?.closest?.('.panel')
      if (inPanel) onPanelKeydown(e)
    }

    const onDocPointer = (e: Event) => {
      if (!state.open) return
      const path = e.composedPath()
      if (path.includes(host)) return
      closePanel()
    }

    host.shadowRoot?.addEventListener('click', onShadowClick)
    host.shadowRoot?.addEventListener('keydown', onShadowKeydown as EventListener)
    document.addEventListener('pointerdown', onDocPointer)

    host.expose!({
      focus: () => inputEl.focus(),
      blur: () => inputEl.blur(),
      open: openPanel,
      close: closePanel,
    })

    return () => {
      inputEl.removeEventListener('click', onInputClick)
      inputEl.removeEventListener('focus', onInputFocus)
      inputEl.removeEventListener('change', onInputChange)
      inputEl.removeEventListener('keydown', onInputKeydown)
      host.shadowRoot?.removeEventListener('click', onShadowClick)
      host.shadowRoot?.removeEventListener('keydown', onShadowKeydown as EventListener)
      document.removeEventListener('pointerdown', onDocPointer)
    }
  },
})
