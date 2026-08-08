<!--
  <sail-date-picker> — ISO date field + month-grid popover (YYYY-MM-DD).
  Input is the trigger; panel is absolute under the host (not native popover —
  trigger is not a button, so popovertarget does not apply).
-->

<scope>
  props: {
    name = ''
    value = ''
    min = ''
    max = ''
    placeholder = 'YYYY-MM-DD'
    disabled = false
    tone: 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted'
    size: 'md' | 'sm' | 'lg' = 'md'
  }
</scope>
<controller>

import type { Action, ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './sail-date-picker.sail'
import {
  DOW,
  DOW_SHORT,
  MONTHS,
  addDays,
  clampIso,
  daysInMonth,
  isoFor,
  parseIso,
  todayIso,
} from './date-math'

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
  /** data-state: empty | today (selected/focused ride ARIA). */
  state: string
}
interface DayHeader { key: string; label: string }

export default controller(({ value, min, max, disabled }: ControllerScope<Scope>) => {
    const open = signal(false)
    const today = signal(todayIso())
    const focused = signal(todayIso())
    const viewYear = signal(new Date().getFullYear())
    const viewMonth = signal(new Date().getMonth())
    const inputEl = signal<HTMLInputElement | null>(null)

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
          disabled: false, label: '', state: 'empty',
        })
      }
      for (let d = 1; d <= totalDays; d++) {
        const iso = isoFor(year, month, d)
        const dt = new Date(year, month, d)
        const dayName = DOW[dt.getDay()] ?? ''
        cells.push({
          key: `${iso}|${sel === iso ? 's' : ''}${foc === iso ? 'f' : ''}${td === iso ? 't' : ''}`,
          iso, day: d,
          empty: false,
          selected: sel === iso,
          focused: foc === iso,
          today: td === iso,
          disabled: Boolean((mn && iso < mn) || (mx && iso > mx)),
          label: `${dayName}, ${MONTHS[month]} ${d}, ${year}`,
          state: td === iso ? 'today' : '',
        })
      }
      return cells
    })

    mount((host) => {
      // Writes go through the prop accessor so external `el.value = …` stays in sync.
      const hostEl = host as HTMLElement & { value: string }

      const moveView = (delta: number) => {
        let y = viewYear()
        let m = viewMonth() + delta
        while (m < 0) { m += 12; y -= 1 }
        while (m > 11) { m -= 12; y += 1 }
        viewYear.set(y)
        viewMonth.set(m)
      }

      const syncViewToValue = () => {
        const seed = (value() as string) || (focused() as string) || todayIso()
        const parts = parseIso(seed)
        if (!parts) return
        viewYear.set(parts.y)
        viewMonth.set(parts.m)
        focused.set(seed)
      }

      const commit = (iso: string) => {
        const parts = parseIso(iso)
        if (!parts) return
        if (min() && iso < min()) return
        if (max() && iso > max()) return
        hostEl.value = iso
        focused.set(iso)
        open.set(false)
        ;(host as any).onChange?.({ value: iso })
      }

      const openPanel = () => {
        if (disabled()) return
        if (open()) return
        syncViewToValue()
        open.set(true)
      }

      const closePanel = () => {
        if (!open()) return
        open.set(false)
      }

      // captureInput may lag first paint — require the action ref before wiring.
      const inputElement = inputEl()
      if (!inputElement) return

      const onInputClick = () => openPanel()
      const onInputFocus = () => openPanel()
      const onInputChange = () => {
        const v = inputElement.value.trim()
        if (v === '') {
          hostEl.value = ''
          ;(host as any).onChange?.({ value: '' })
          return
        }
        const parts = parseIso(v)
        if (parts) commit(v)
      }
      const onInputKeydown = (e: KeyboardEvent) => {
        if (e.key === 'Escape' && open()) {
          e.preventDefault()
          closePanel()
        } else if (e.key === 'ArrowDown' && !open()) {
          e.preventDefault()
          openPanel()
        }
      }
      inputElement.addEventListener('click', onInputClick)
      inputElement.addEventListener('focus', onInputFocus)
      inputElement.addEventListener('change', onInputChange)
      inputElement.addEventListener('keydown', onInputKeydown)

      const onPanelKeydown = (e: KeyboardEvent) => {
        const cur = (focused() as string) || (value() as string) || todayIso()
        let next: string | null = null
        if (e.key === 'ArrowLeft') next = addDays(cur, -1)
        else if (e.key === 'ArrowRight') next = addDays(cur, 1)
        else if (e.key === 'ArrowUp') next = addDays(cur, -7)
        else if (e.key === 'ArrowDown') next = addDays(cur, 7)
        else if (e.key === 'PageUp') next = addDays(cur, -30)
        else if (e.key === 'PageDown') next = addDays(cur, 30)
        else if (e.key === 'Enter') { e.preventDefault(); commit(cur); return }
        else if (e.key === 'Escape') { e.preventDefault(); closePanel(); inputElement.focus(); return }
        if (next != null) {
          e.preventDefault()
          const clamped = clampIso(next, min() as string, max() as string)
          focused.set(clamped)
          const parts = parseIso(clamped)
          if (parts) {
            viewYear.set(parts.y)
            viewMonth.set(parts.m)
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
        const day = t.closest?.('button[data-iso]') as HTMLButtonElement | null
        if (day && !day.disabled) commit(day.dataset.iso ?? '')
      }

      const onShadowKeydown = (e: KeyboardEvent) => {
        if (!open()) return
        const inPanel = (e.target as HTMLElement | null)?.closest?.('[role="dialog"]')
        if (inPanel) onPanelKeydown(e)
      }

      const onDocPointer = (e: Event) => {
        if (!open()) return
        const path = e.composedPath()
        if (path.includes(host)) return
        closePanel()
      }

      ;(host.shadowRoot ?? host).addEventListener('click', onShadowClick)
      ;(host.shadowRoot ?? host).addEventListener('keydown', onShadowKeydown as EventListener)
      document.addEventListener('pointerdown', onDocPointer)

      host.expose!({
        focus: () => inputElement.focus(),
        blur: () => inputElement.blur(),
        open: openPanel,
        close: closePanel,
      })

      return () => {
        inputElement.removeEventListener('click', onInputClick)
        inputElement.removeEventListener('focus', onInputFocus)
        inputElement.removeEventListener('change', onInputChange)
        inputElement.removeEventListener('keydown', onInputKeydown)
        ;(host.shadowRoot ?? host).removeEventListener('click', onShadowClick)
        ;(host.shadowRoot ?? host).removeEventListener('keydown', onShadowKeydown as EventListener)
        document.removeEventListener('pointerdown', onDocPointer)
      }
    })

    const captureInput: Action<void> = (node) => {
      inputEl.set(node as HTMLInputElement)
      return { destroy: () => inputEl.set(null) }
    }
    return { open, today, focused, viewYear, viewMonth, captureInput, monthLabel, dayHeaders, monthCells }
  })

</controller>
<column gap="0" style({ box: position(relative).w(full) })>
  <row gap="2" align="center" style({
    box: w(full).h(control-h-md).padX(2_5),
    paint: bg(background).fg(foreground).borderWidth(1).border(border).radius(md),
    when: { focusWithin: { paint: outline(ring) } },
  })>
    <input
      type="text"
      flush
      name={name()}
      placeholder={placeholder()}
      disabled={disabled()}
      autocomplete="off"
      inputmode="numeric"
      aria-haspopup="dialog"
      aria-expanded={open() ? 'true' : 'false'}
      bind:value={value}
      use:captureInput
      style({
        box: grow(1).minW(0),
        text: fontFamily(body),
        when: { disabled: { paint: cursor(not-allowed) } },
      })
    >
    <svg
      viewBox="0 0 16 16"
      aria-hidden="true"
      fill="none"
      stroke="currentColor"
      stroke-width="1.5"
      stroke-linecap="round"
      stroke-linejoin="round"
      style({
        box: w(icon-size-sm).h(icon-size-sm).minW(icon-size-sm),
        paint: fg(muted-foreground).pointerEvents(none),
      })
    >
      <rect x="2" y="3" width="12" height="11" rx="1.5"></rect>
      <path d="M2 6h12 M5 1.5v3 M11 1.5v3"></path>
    </svg>
  </row>
  <if {open() && !disabled()}>
    <div
      role="dialog"
      aria-label="Choose date"
      style({
        box: position(absolute).left(0).w(panel-max-w-sm).pad(2).mt(1),
        paint: z(popover).bg(surface-1).fg(foreground).borderWidth(1).border(border).radius(md).shadow(md),
      })
    >
      <span style({ box: display(contents), text: fontFamily(body) })>
        <row align="center" justify="between" style({ box: mb(1) })>
          <button
            type="button"
            data-act="prev"
            aria-label="Previous month"
            style({
              box: w(icon-size-2xl).h(icon-size-2xl),
              paint: radius(sm).fg(muted-foreground).cursor(pointer),
              when: { hover: { paint: bg(muted).fg(foreground) } },
            })
          >‹</button>
          <span style({ box: grow(1), text: fontWeight(semibold).textAlign(center) })>{monthLabel()}</span>
          <button
            type="button"
            data-act="next"
            aria-label="Next month"
            style({
              box: w(icon-size-2xl).h(icon-size-2xl),
              paint: radius(sm).fg(muted-foreground).cursor(pointer),
              when: { hover: { paint: bg(muted).fg(foreground) } },
            })
          >›</button>
        </row>
        <grid role="grid" cols="7" gap="1">
          <for {dayHeaders() as d} key={d.key}>
            <span
              role="columnheader"
              style({
                box: padY(1),
                paint: fg(muted-foreground),
                text: fontSize(sm).fontWeight(medium).textAlign(center),
              })
            >{d.label}</span>
          </for>
          <for {monthCells() as cell} key={cell.key}>
            <button
              type="button"
              disabled={cell.disabled}
              aria-label={cell.label}
              aria-selected={cell.selected ? 'true' : 'false'}
              tabindex={cell.focused ? '0' : '-1'}
              data-state={cell.state || null}
              data-iso={cell.iso}
              style({
                box: h(control-h-sm),
                paint: radius(sm).cursor(pointer).fg(foreground),
                text: textAlign(center),
                when: {
                  hover: { paint: bg(hover-bg) },
                  selected: { paint: bg(primary).fg(primary-foreground) },
                  disabled: { paint: fg(muted-foreground).opacity(50).cursor(not-allowed) },
                  stateEmpty: { paint: visibility(hidden) },
                  stateToday: { paint: outline(border-strong) },
                  focus: { paint: outline(ring) },
                },
              })
            >{cell.empty ? '' : cell.day}</button>
          </for>
        </grid>
      </span>
    </div>
  </if>
</column>
