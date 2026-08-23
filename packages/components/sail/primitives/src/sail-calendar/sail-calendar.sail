<!--
  <sail-calendar> — month / week / day view over `./engine/createCalendar`.
  Engine via `calendar` identity; `events`/`focus-date` JSON strings are SSR fallback.
  Interactions (range drag, create/move) in sail-calendar.interactions.
-->

<scope>
  props: {
    view: 'month' | 'week' | 'day' = 'month'
    events = ''
    'focus-date' = ''
    'week-starts-on' = 0
    'slot-minutes' = 30
    'day-start-hour' = 0
    'day-end-hour' = 24
    'range-mode': 'none' | 'date' | 'datetime' = 'none'
    editable = false
    label = ''
  }
</scope>
<controller>
// Thin seam: identity → engine surfaces; DOM bridge in sail-calendar.interactions.

import type { ControllerScope } from '@ship/sail-runtime'
import { createCalendar } from './engine'
import type { Calendar, CalendarEvent, DateRange, MonthCell, ViewMode } from './engine'
import { wireCalendarInteractions } from './sail-calendar.interactions'
import type { Scope } from './sail-calendar.sail'

const EMPTY: CalendarEvent[] = []
const DOW_LABELS = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat']

function parseJsonEvents(raw: string): CalendarEvent[] {
  const s = raw.trim()
  if (!s) return EMPTY
  try {
    const parsed = JSON.parse(s)
    return Array.isArray(parsed) ? (parsed as CalendarEvent[]) : EMPTY
  } catch {
    return EMPTY
  }
}

function weekdayLabels(weekStartsOn: number): { key: string; label: string }[] {
  const out: { key: string; label: string }[] = []
  for (let i = 0; i < 7; i++) {
    const idx = (weekStartsOn + i) % 7
    out.push({ key: `dow-${idx}`, label: DOW_LABELS[idx]! })
  }
  return out
}

export default controller((ctx: ControllerScope<Scope>) => {
    const { events, focusDate, weekStartsOn, slotMinutes, dayStartHour, dayEndHour, rangeMode } = ctx
    // tracks attr only accepts --var.
    const timeGridTracks = derived<string>(() => 'auto 1fr')
    // calendar identity (mount) supersedes; seed from SSR events until then.
    const engine = signal<Calendar | null>(null)

    // omit initialDate when empty (exactOptionalPropertyTypes).
    const initialDate = focusDate()
    const fallback = createCalendar({
      events: derived(() => parseJsonEvents(events())),
      ...(initialDate ? { initialDate } : {}),
      weekStartsOn: (weekStartsOn() || 0) as 0,
      slotMinutes: slotMinutes() || 30,
      dayStartHour: dayStartHour() ?? 0,
      dayEndHour: dayEndHour() ?? 24,
      rangeMode: (rangeMode() || 'none') as 'none',
    })

    /** The active engine — the calendar identity if present, else fallback. */
    const cal = derived<Calendar>(() => engine() ?? fallback)


    const view = derived<ViewMode>(() => cal().view())
    const title = derived(() => cal().title())
    const monthCells = derived(() => cal().monthCells())
    // Content-sensitive cell keys (not iso-only) so nested event for re-renders;
    // range excluded so drag nodes stay attached while in-range rebinds.
    const inRangeIso = (iso: string, r: DateRange | null): boolean =>
      r != null && iso >= r.start.slice(0, 10) && iso <= r.end.slice(0, 10)
    type CellRow = { key: string; iso: string; cell: MonthCell; inRange: boolean; state: string }
    const weekRows = derived<{ key: string; cells: CellRow[] }[]>(() => {
      const cells = cal().monthCells()
      const sel = cal().selectedEventIds()
      const range = cal().range()
      const sig = (c: MonthCell): string =>
        `${c.iso}|${c.today ? 't' : ''}|${c.inMonth ? '' : 'o'}${c.isWeekend ? 'w' : ''}|${c.events.map((e) => e.id).join(',')}|${c.overflowCount}|${sel.join(',')}`
      const rows: { key: string; cells: CellRow[] }[] = []
      for (let row = 0; row < 6; row++) {
        const slice = cells.slice(row * 7, row * 7 + 7)
        const cellRows = slice.map((c) => {
          const inRange = inRangeIso(c.iso, range)

          const state = [
            c.inMonth ? '' : 'out',
            c.today ? 'today' : '',
            c.isWeekend ? 'weekend' : '',
            inRange ? 'range' : '',
          ]
            .filter(Boolean)
            .join(' ')
          return { key: sig(c), iso: c.iso, cell: c, inRange, state }
        })
        rows.push({ key: cellRows.map((cr) => cr.key).join('::'), cells: cellRows })
      }
      return rows
    })
    const weekdays = derived(() => weekdayLabels(weekStartsOn() || 0))

    const dayColumns = derived(() => {
      const sel = cal().selectedEventIds()
      return cal()
        .dayColumns()
        .map((col) => ({
          key: `${col.iso}|${col.laidOut.map((p) => p.event.id).join(',')}|${col.allDayEvents.map((e) => e.id).join(',')}|${sel.join(',')}`,
          col: {
            ...col,
            // Lane geometry as ready CSS values — row member binds paint these.
            laidOut: col.laidOut.map((p) => ({
              ...p,
              top: `${p.topPct}%`,
              height: `${p.heightPct}%`,
              left: `${(p.laneIndex / p.laneCount) * 100}%`,
              width: `${(1 / p.laneCount) * 100}%`,
            })),
          },
        }))
    })
    const timeSlots = derived(() => cal().timeSlots())
    const agenda = derived(() => cal().agenda())
    const isEmpty = derived(() => cal().isEmpty())
    const selectedIds = derived(() => cal().selectedEventIds())

    const isMonth = derived(() => view() === 'month')
    const isWeek = derived(() => view() === 'week')
    const isDay = derived(() => view() === 'day')
    // Month grid roving tabindex anchor.
    const focusedIso = derived(() => cal().focusDate())

    mount((host) => {

      defineHostProperty(asSailHost(host) as HostBag, 'calendar', {
        get: () => engine(),
        set: (c: Calendar | null) => {
          engine.set(c)
        },
      })


      const stopView = effect(() => {
        const v = view()
        if (hostGetAttr(host, 'view') !== v) hostSetAttr(host, 'view', v)
      })

      const active = (): Calendar => cal()

      // Deduped emit; engine identity swap reseeds without emitting.
      let lastEngine: Calendar | null = null
      let lastView: ViewMode | null = null
      let lastDate: string | null = null
      const stopEmit = effect(() => {
        const cal = active()
        const v = cal.view()
        const d = cal.focusDate()
        if (cal === lastEngine) {
          if (lastView !== null && v !== lastView) host.emit!('sail-view-change', { view: v })
          if (lastDate !== null && d !== lastDate) host.emit!('sail-date-change', { date: d })
        }
        lastEngine = cal
        lastView = v
        lastDate = d
      })

      const onShadowClick = (e: DomEv) => {
        const t = e.target as HostBag | null
        if (!t) return
        const cal = active()

        const viewBtn = t.closest?.('[data-view]') as HostBag | null
        if (viewBtn) {
          cal.setView(viewBtn.dataset.view as ViewMode)
          return
        }
        const navBtn = t.closest?.('[data-nav]') as HostBag | null
        if (navBtn) {
          const act = navBtn.dataset.nav
          if (act === 'next') cal.next()
          else if (act === 'prev') cal.prev()
          else if (act === 'today') cal.today()
          return
        }
        const chip = t.closest?.('[data-event-id]') as HostBag | null
        if (chip) {
          const id = chip.dataset.eventId!
          cal.selectEvent(id, { additive: (e as DomEv).metaKey })
          host.emit!('sail-event-click', { id })
          host.emit!('sail-event-select', {
            id,
            selected: cal.selectedEventIds().includes(id),
            selectedIds: cal.selectedEventIds(),
          })
          return
        }
        const dayCell = t.closest?.('[role=gridcell][data-iso]') as HostBag | null
        if (dayCell) cal.goToDate(dayCell.dataset.iso!)
      }

      // Month-grid keyboard: arrows/Home/End/Page*/Enter → engine; DOM focus follows.
      const onShadowKey = (e: DomEv) => {
        const cell = (e.target as HostBag | null)?.closest?.(
          '[role=gridcell][data-iso]',
        ) as HostBag | null
        if (!cell) return
        // Shift+Arrow = range extension (interactions module owns both).
        if (e.shiftKey) return
        const cal = active()
        switch (e.key) {
          case 'ArrowRight': cal.moveFocus(1); break
          case 'ArrowLeft': cal.moveFocus(-1); break
          case 'ArrowDown': cal.moveFocus(7); break
          case 'ArrowUp': cal.moveFocus(-7); break
          case 'Home': cal.focusWeekEdge(false); break
          case 'End': cal.focusWeekEdge(true); break
          case 'PageDown': cal.next(); break
          case 'PageUp': cal.prev(); break
          case 'Enter':
          case ' ':
            cal.goToDate(cell.dataset.iso!)
            break
          default:
            return
        }
        e.preventDefault()
        // Move DOM focus onto the newly-anchored cell after the grid re-renders.
        const iso = active().focusDate()
        queueMicrotask(() => {
          const next = (host.shadowRoot ?? host).querySelector(
            `[role=gridcell][data-iso="${iso}"]`,
          ) as HostBag | null
          next?.focus()
        })
      }

      ;(host.shadowRoot ?? host).addEventListener('click', onShadowClick)
      ;(host.shadowRoot ?? host).addEventListener('keydown', onShadowKey)

      // Range selection (month drag + Shift+Arrow) + drag-to-create/move (time-
      // grid slot drag). The DOM↔engine bridge lives in the interactions module;
      // it reads the engine fresh per event and emits sail-range-change /
      // sail-event-create-intent / sail-event-move, auto-cleaning against the mount
      // scope. The slot span is the step between consecutive slots (the engine
      // builds them by stepping `slotMinutes`, so the gap recovers it).
      const slotSpan = (): number => {
        const s = active().timeSlots()
        return s.length > 1 ? s[1]!.minutes - s[0]!.minutes : 30
      }
      wireCalendarInteractions(host, { calOf: active, slotMinutes: slotSpan })

      host.expose!({
        next: () => active().next(),
        prev: () => active().prev(),
        today: () => active().today(),
        setView: (v: ViewMode) => active().setView(v),
      })

      return () => {
        stopView()
        stopEmit()
        ;(host.shadowRoot ?? host).removeEventListener('click', onShadowClick)
        ;(host.shadowRoot ?? host).removeEventListener('keydown', onShadowKey)
        delete (host as { calendar?: unknown }).calendar
      }
    })

    return {
      engine,
      cal,
      view,
      title,
      monthCells,
      weekRows,
      weekdays,
      dayColumns,
      timeSlots,
      agenda,
      isEmpty,
      selectedIds,
      isMonth,
      isWeek,
      isDay,
      focusedIso,
      timeGridTracks,
    }
  })

</controller>
<div style({ display(block).overflow({ all: hidden }).bg(background).fg(foreground).border({ color: border, width: 1 }).radius(lg).elevation(xs) })>  <span style({ display(contents).type({ family: body, size: density }) })>    <row align="center" gap="density" style({ pad({ x: density-row-px, y: density-row-py }).border(default-b) })>
    <row gap="1" align="center">
      <button type="button" variant="outline" size="sm" data-nav="prev" aria-label="Previous">‹</button>
      <button type="button" variant="outline" size="sm" data-nav="today">Today</button>
      <button type="button" variant="outline" size="sm" data-nav="next" aria-label="Next">›</button>
    </row>
    <h2 style({ grow(1).type({ size: base, weight: semibold }) })>{title()}</h2>
    <row role="group" aria-label="Calendar view" gap="1" align="center">
      <button type="button" variant="outline" size="sm" data-view="month" aria-pressed={isMonth() ? 'true' : 'false'}>Month</button>
      <button type="button" variant="outline" size="sm" data-view="week" aria-pressed={isWeek() ? 'true' : 'false'}>Week</button>
      <button type="button" variant="outline" size="sm" data-view="day" aria-pressed={isDay() ? 'true' : 'false'}>Day</button>
    </row>
  </row>

  <if {isEmpty()}>
    <div role="status" style({ pad({ x: density-row-px, y: 6 }).fg(muted-foreground) })>      <span style({ display(contents).type({ align: center }) })>        <slot name="empty">No events</slot>
      </span>
</div>
  </if>

  <if {isMonth()}>
  <div role="grid" aria-label={label() || 'Calendar month'}>
    <grid cols="7" gap="0" role="presentation">
      <for {weekdays() as d} key={d.key}>
        <span role="columnheader" style({ pad({ y: density-row-py }).fg(muted-foreground).border(default-b).type({ size: density, weight: medium, align: center }) })>{d.label}</span>
      </for>
    </grid>
    <for {weekRows() as wk} key={wk.key}>
      <grid cols="7" gap="0" role="row">
        <for {wk.cells as cr} key={cr.key}>
          <column
            role="gridcell"
            data-iso={cr.cell.iso}
            tabindex={cr.cell.iso === focusedIso() ? '0' : '-1'}
            data-state={cr.state}
            aria-current={cr.cell.today ? 'date' : null}
            aria-label={cr.cell.iso}
            gap="1" style({ pad({ x: density-row-px, y: density-row-py }).size({ min-h: 84px }).group().border(default-b), when: { stateOut: fg(muted-foreground).bg(muted), stateWeekend: bg(muted), stateRange: bg(selected-bg) } })>
            <row align="center" justify="center" gap="0" style({ size({ h: control-h-2xs, min-w: control-h-2xs }).radius(full), when: { groupStateToday: fg(primary).border({ kind: outline, color: primary }) } })>              <span style({ display(contents).type({ size: density }), when: { groupStateToday: type({ weight: semibold }) } })>                {cr.cell.day}
              </span>
</row>
            <for {cr.cell.events as ev} key={ev.id}>
              <button
                type="button"
                data-event-id={ev.id}
                data-state={ev.allDay ? 'allday' : ''}
                aria-selected={selectedIds().includes(ev.id) ? 'true' : 'false'}
                title={ev.title} style({ position(relative).pad({ x: 1_5, y: half, l: 3 }).cursor(pointer).fg(foreground).bg(hover-bg).radius(sm).type({ size: density, align: start, wrap: truncate }), when: { stateAllday: bg(hover-bg), selected: border({ kind: outline, color: ring }) } })>
                <span aria-hidden="true" style({ position({ scheme: absolute, t: 1, b: 1, l: half }).size({ w: half }).radius(sm).bg(primary), when: { groupStateAllday: bg(accent) } })></span>
                {ev.title}
              </button>
            </for>
            <if {cr.cell.overflowCount > 0}>
              <span style({ pad({ x: 1 }).fg(muted-foreground).type({ size: xs, weight: semibold }) })>+{cr.cell.overflowCount} more</span>
            </if>
          </column>
        </for>
      </grid>
    </for>
  </div>
</if>

<if {isWeek() || isDay()}>
  <grid use:gridTracks={timeGridTracks()} tracks="--cal-time-tracks" gap="0" role="grid" aria-label={label() || 'Calendar time grid'}>
    <column role="rowgroup" gap="0" style({ border(default-r) })>
      <for {timeSlots() as slot} key={slot.key}>
        <div data-slot={slot.minutes} role="rowheader" style({ size({ h: control-h-md }).pad({ x: density-row-px }).fg(muted-foreground).border(default-b) })>          <span style({ display(contents).type({ size: density }) })>            {slot.label}
          </span>
</div>
      </for>
    </column>
    <row gap="0" style({ grow(1) })>
      <for {dayColumns() as dc} key={dc.key}>
        <div data-state={dc.col.today ? 'today' : ''} data-iso={dc.col.iso} style({ grow(1).position(relative).size({ min-h: 864px }).border(default-r), when: { stateToday: bg(selected-bg) } })>
          <column gap="0" style({ position({ scheme: absolute, t: 0, b: 0, l: 0, r: 0 }) })>
            <for {timeSlots() as slot} key={slot.key}>
              <div data-iso={dc.col.iso} data-minutes={slot.minutes} style({ grow(1).size({ min-h: control-h-md }).cursor(cell).border(default-b), when: { hover: bg(hover-bg) } })></div>
            </for>
          </column>
          <column gap="1" style({ position(relative).pad(half).position({ z: elevated }) })>
            <for {dc.col.allDayEvents as ev} key={ev.id}>
              <button
                type="button"
                data-event-id={ev.id}
                data-state="allday"
                aria-selected={selectedIds().includes(ev.id) ? 'true' : 'false'}
                title={ev.title} style({ position(relative).pad({ x: 1_5, y: half }).cursor(pointer).fg(foreground).bg(hover-bg).radius(sm).type({ size: density, align: start, wrap: truncate }), when: { selected: border({ kind: outline, color: ring }) } })>{ev.title}</button>
            </for>
          </column>
          <for {dc.col.laidOut as pe} key={pe.event.id}>
            <button
              type="button"
              data-event-id={pe.event.id}
              aria-selected={selectedIds().includes(pe.event.id) ? 'true' : 'false'}
              title={pe.event.title} style({ position({ scheme: absolute, t: bind(pe.top), l: bind(pe.left) }).size({ w: bind(pe.width), h: bind(pe.height) }).pad({ x: 1, y: half }).overflow({ all: hidden }).cursor(pointer).fg(primary).bg(hover-bg).radius(sm).type({ size: density, align: start }), when: { selected: border({ kind: outline, color: ring }) } })>{pe.event.title}</button>
          </for>
        </div>
      </for>
    </row>
  </grid>
</if>
  </span>
</div>
