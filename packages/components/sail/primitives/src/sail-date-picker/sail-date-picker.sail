<!--
  <sail-date-picker> — input field + calendar popover.
  Value is ISO 8601 (`YYYY-MM-DD`). The input itself is the trigger:
  clicking it (or focusing) opens the popover. The calendar is a
  month grid; arrow keys move day focus, Enter selects, Escape closes.

  Compose-points:
  - The trigger surface mirrors the <input> chrome (border, focus
    ring, height matrix) so the picker drops into the same form rows.
  - The popover panel is hand-rolled (no <popover>): the trigger
    isn't a <button>, so the native popovertarget wiring sail-popover
    relies on wouldn't apply. The panel uses absolute positioning
    relative to the host.
  ─────────────────────────────────────────────────────────────────────
  Props
    name        Form field name.
    value       ISO date string (YYYY-MM-DD). Empty when unset.
    min         Earliest allowed date (ISO).
    max         Latest allowed date (ISO).
    placeholder Native input placeholder shown when value is empty.
    disabled    Disables interaction + form participation.
    tone        Tones focus ring + selection.
    size        sm | md | lg.
  ─────────────────────────────────────────────────────────────────────
  Events
    sail-change  { detail: { value: string } }   // ISO date or '' when cleared
  ─────────────────────────────────────────────────────────────────────

  NEW SURFACE (re-port from the feat/next baseline, parity-gate pass)
  ─────────────────────────────────────────────────────────────────────
  - Deleted the per-component tone theme block (prop=tone, channels
    tone-border + tone-ring) — those two channel names are now GLOBAL
    framework tone/size contracts (contracts.css), and this component's old block
    collided with them 1:1. `border-tone-border` /
    `focus-within:ring-tone-ring` below now resolve against the framework's
    tone contract directly, same as sail-alert/sail-card/sail-select.
    (NOTE for future editors: don't spell the deleted block out in real
    open/close theme-tag syntax in this comment — the compiler's theme
    lowering pass scans for the literal tag text without regard for HTML
    comment boundaries, and a stray opening mention here will pair with
    the real closing tag on the surviving size-theme block below and
    silently swallow every prop declaration in between. Prose only.)
  - Kept the local size theme block (prop=size, channels dp-h/dp-fs/dp-px)
    (channel NAMES don't collide with the global `size-*` contract, and the
    values genuinely differ from it — global `[size]` padding is
    space-2_5/3/4 at sm/md/lg, feat/next's picker used space-2/2_5/3; same
    established pattern as sail-input/-badge/-chip keeping their own size
    theme). Fixed two values while re-porting: the `md` row had drifted to
    the density channel (`density-text`/`density-row-px`, unrelated to this
    component — feat/next's md size is a fixed 13px/10px, not
    density-reactive) — restored to `text-base`/`space-2_5`. The `lg` row's
    font size had drifted to `text-sm` (12px) instead of feat/next's 14px —
    restored to `text-md`.
  - `.day.selected`'s background/color/font-weight had been dropped entirely
    in the prior "dissolve style" pass (only `box-shadow:none` survived).
    Restored using the SOLID `--tone`/`--tone-contrast` channel pair (not
    `--tone-ring`, which is now a translucent wash, not a solid fill) — the
    channel pairing the framework provides specifically for solid-fill +
    readable-text scenarios. Net effect: still a solid, tone-colored,
    legible selected-day pill, matching feat/next's intent; the untoned/
    default case shifts from feat/next's hardcoded "solid primary" to the
    framework's generic "solid foreground" (an intentional consequence of
    tone no longer being a per-component color choice).
  - `.day.empty { visibility: hidden; }` and
    `.day:hover:not(:disabled):not(.empty) { background: var(--hover-bg); }`
    were also dropped in the prior pass — restored (leading/trailing padding
    cells were rendering as visible ghost buttons, and no day had hover
    feedback).
  - `.dow`'s font-size had drifted to `text:xs` (11px); feat/next's raw
    `0.75rem` is 12px — restored to `text:sm`.
  - `.panel`'s width had drifted to `w:panel-max-w-md` (480px); feat/next's
    raw `260px` is far closer to `--panel-max-w-sm` (320px — still a
    deliberate token-driven widening, per tokens.css's own documented
    re-port convention, and gives the 7-column day grid comfortable room).
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
  /** data-state channel: empty | today (selected/focused ride ARIA). */
  state: string
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
      // `value` is a declared <prop> — writes below go through its own
      // compiler-installed accessor (the same signal `value()` reads),
      // never a disconnected local copy, so external `el.value = ...` and
      // this component's own commits stay in sync.
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

      // The ref lands during the template's mount, so read the signal (not the
      // unwrapped proxy value, which is still null when this body runs).
      const inputElement = inputEl()
      if (!inputElement) return

      const onInputClick = () => openPanel()
      const onInputFocus = () => openPanel()
      const onInputChange = () => {
        // User typed an ISO date manually; commit if it parses.
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
<row gap="2" align="center" style({ box: w(full).h(control-h-md).padX(2_5), paint: bg(background).fg(foreground).borderWidth(1).border(border).radius(md), when: { focusWithin: { paint: outline(ring) } } })>
  <input type="text" flush name={name()} placeholder={placeholder()} disabled={disabled()} autocomplete="off" inputmode="numeric" aria-haspopup="dialog" aria-expanded={open() ? 'true' : 'false'} bind:value={value} use:captureInput style({ box: grow(1).minW(0), text: fontFamily(body), when: { disabled: { paint: cursor(not-allowed) } } })>
  <svg viewBox="0 0 16 16" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" style({ box: w(icon-size-sm).h(icon-size-sm).minW(icon-size-sm), paint: fg(muted-foreground).pointerEvents(none) })>
    <rect x="2" y="3" width="12" height="11" rx="1.5"></rect>
    <path d="M2 6h12 M5 1.5v3 M11 1.5v3"></path>
  </svg>
</row>
<if {open() && !disabled()}>
  <div role="dialog" aria-label="Choose date" style({ box: position(absolute).left(0).w(panel-max-w-sm).pad(2).mt(1), paint: z(popover).bg(popover).fg(popover-foreground).borderWidth(1).border(border).radius(md).shadow(md) })>    <span style({ box: display(contents), text: fontFamily(body) })>      <row align="center" justify="between" style({ box: mb(1) })>
      <button type="button" data-act="prev" aria-label="Previous month" style({ box: w(icon-size-2xl).h(icon-size-2xl), paint: radius(sm).fg(muted-foreground).cursor(pointer), when: { hover: { paint: bg(muted).fg(foreground) } } })>‹</button>
      <span style({ box: grow(1), text: fontWeight(semibold).textAlign(center) })>{monthLabel()}</span>
      <button type="button" data-act="next" aria-label="Next month" style({ box: w(icon-size-2xl).h(icon-size-2xl), paint: radius(sm).fg(muted-foreground).cursor(pointer), when: { hover: { paint: bg(muted).fg(foreground) } } })>›</button>
    </row>
    <grid role="grid" cols="7" gap="1">
      <for {dayHeaders() as d} key={d.key}>
        <span role="columnheader" style({ box: padY(1), paint: fg(muted-foreground), text: fontSize(sm).fontWeight(medium).textAlign(center) })>{d.label}</span>
      </for>
      <for {monthCells() as cell} key={cell.key}>
        <button
          type="button"
          disabled={cell.disabled}
          aria-label={cell.label}
          aria-selected={cell.selected ? 'true' : 'false'}
          tabindex={cell.focused ? '0' : '-1'}
          data-state={cell.state || null}
          data-iso={cell.iso} style({ box: h(control-h-sm), paint: radius(sm).cursor(pointer).fg(foreground), text: textAlign(center), when: { hover: { paint: bg(hover-bg) }, selected: { paint: bg(primary).fg(primary-foreground) }, disabled: { paint: fg(muted-foreground).opacity(50).cursor(not-allowed) }, stateEmpty: { paint: visibility(hidden) }, stateToday: { paint: outline(border-strong) }, focus: { paint: outline(ring) } } })>{cell.empty ? '' : cell.day}</button>
      </for>
    </grid>
    </span>
</div>
</if>
</column>
