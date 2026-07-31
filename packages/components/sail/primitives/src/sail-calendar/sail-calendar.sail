<!--
  <sail-calendar> — month / week / day calendar surface (Slice 2, the VIEW).
  Pure-framework component over the COMMITTED headless engine
  (`./engine/createCalendar`), which owns all reactive state + view-model
  (monthCells / dayColumns / timeSlots / agenda / selection / navigation). The
  engine arrives via `calendar` IDENTITY; `events`/`focus-date` (type=string)
  are the SSR/declarative fallback (JSON.parse in the controller — `type="json"`
  does NOT compile). All emits go through host.emit! (see controller).

  A11y: month grid is a FIXED 6×7 role=grid/row/gridcell tree (never virtualized
  → NO windowed-count ARIA); event chips are role=button DESCENDANTS inside a
  gridcell; today → aria-current="date". Density: NO density/dense prop — rows
  read the --density-* channel off <html data-density>.

  Range selection (range-mode date|datetime): pointer drag across month-grid day
  cells, or Shift+Arrow from the focus anchor, sets the engine `range` signal and
  emits sail-range-change. Drag-to-create (empty time slot → sail-event-create-intent)
  and drag-to-move (timed chip → sail-event-move) ride a background slot grid that
  carries data-iso + data-minutes so the pointer→time map is element-addressed.
  The virtualized agenda remains an explicit GAP.
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
// <sail-calendar> controller — Slice 2 (the VIEW). Glue-only / tier-2.
//
// The headless engine (`./engine/createCalendar`) owns ALL reactive state +
// view-model derivation (month/week/day, agenda, selection, navigation). This
// controller is the thin seam: it routes the live `Calendar` in via the
// `calendar` IDENTITY path (a JS reference, never a stringified attribute),
// falls back to building an engine from the `events` `type="string"` SSR prop
// (sail-combobox JSON.parse pattern), re-exposes the engine's `Derived` surfaces
// as scope signals the template reads, and emits `sail-*` events via the runtime
// `host.emit!` helper (NOT open-coded `dispatchEvent`).
//
// Cardinals honored: mutations only in on:* / mount; no fetch / ship.* / query()
// (engine purity asserted by engine/purity.test.ts). Range selection +
// drag-to-create/move ride the engine's range/intent mutators; the DOM bridge
// (pointer drag + Shift+Arrow → engine + sail-* emits) lives in `./sail-calendar.interactions`.

import type { ControllerScope } from '@ship/sail-runtime'
import { createCalendar } from './engine'
import type { Calendar, CalendarEvent, DateRange, MonthCell, ViewMode } from './engine'
import { wireCalendarInteractions } from './sail-calendar.interactions'
import type { Scope } from './sail-calendar.sail'

const EMPTY: CalendarEvent[] = []
const DOW_LABELS = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat']

/** SSR/declarative fallback decoder: parse the `events` JSON-string prop.
 *  Bad JSON degrades to an empty list (never throws into the render). */
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

/** Weekday header labels rotated to honor `weekStartsOn`. */
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
    // The time-grid tracks — a constant per skin, exposed as a derived so
    // sx/use:gridTracks={...} can stay expression-bound. The language's
    // layout element only accepts `--var` for `tracks`, so this lives as
    // a custom property read at runtime, not a static literal.
    const timeGridTracks = derived<string>(() => 'auto 1fr')
    // `use:eventSpan={eventRect(pe)}` — convert a PositionedEvent's per-cent
    // geometry into a Rect the action writes as four custom properties.
    // topPct/heightPct/laneIndex/laneCount are runtime decimals from the
    // layouter (not design); we stringify each as `N%` so the action reads
    // it straight through.
    const eventRect = (pe: { topPct: number; heightPct: number; laneIndex: number; laneCount: number }) => ({
      top: `${pe.topPct}%`,
      height: `${pe.heightPct}%`,
      left: `${(pe.laneIndex / pe.laneCount) * 100}%`,
      width: `${(1 / pe.laneCount) * 100}%`,
    })
    // The live engine. `calendar` identity (set in mount) supersedes; until
    // then we seed from the SSR `events` string + the focus-date attr so a
    // declarative (no-parent-controller) mount still renders.
    const engine = signal<Calendar | null>(null)

    // exactOptionalPropertyTypes: omit initialDate when empty rather than
    // passing `undefined` to the optional field.
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

    // Re-expose the engine view-model as scope deriveds the template reads.
    const view = derived<ViewMode>(() => cal().view())
    const title = derived(() => cal().title())
    const monthCells = derived(() => cal().monthCells())
    // Each cell carries a content-sensitive key so a keyed {#for} re-renders the
    // cell (and its inner event {#for}) when the engine swaps in / events change —
    // a stable `iso`-only key would let the runtime reuse the row DOM and skip
    // the inner loop (the nested-for-over-a-plain-array reuse trap).
    // A cell is in-range when the range (day part) covers its day. The cell key
    // is content-sensitive but EXCLUDES the range, so `in-range` re-binds on the
    // REUSED cell as the range changes (a re-key would detach drag nodes mid-drag).
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
          // data-state is the component-state channel the skin keys on
          // (state-out:/state-today:/state-weekend:/state-range:).
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
    // Day columns with a content-sensitive key (same reuse-trap guard as the
    // month grid): the laid-out timed events change when the engine swaps.
    const dayColumns = derived(() => {
      const sel = cal().selectedEventIds()
      return cal()
        .dayColumns()
        .map((col) => ({
          key: `${col.iso}|${col.laidOut.map((p) => p.event.id).join(',')}|${col.allDayEvents.map((e) => e.id).join(',')}|${sel.join(',')}`,
          col,
        }))
    })
    const timeSlots = derived(() => cal().timeSlots())
    const agenda = derived(() => cal().agenda())
    const isEmpty = derived(() => cal().isEmpty())
    const selectedIds = derived(() => cal().selectedEventIds())
    // view-mode branch flags the template reads (CSS also branches via the
    // reflected `view` host attribute synced in mount).
    const isMonth = derived(() => view() === 'month')
    const isWeek = derived(() => view() === 'week')
    const isDay = derived(() => view() === 'day')
    // The keyboard focus anchor — drives the month grid's roving tabindex
    // (focused cell tabindex=0, all others -1) so the grid is reachable + operable.
    const focusedIso = derived(() => cal().focusDate())

    mount((host) => {
      // calendar identity — a live `Calendar` reference (the engine path),
      // routed into the scope signal so every derived re-resolves onto it.
      Object.defineProperty(host, 'calendar', {
        configurable: true,
        get: () => engine(),
        set: (c: Calendar | null) => {
          engine.set(c)
        },
      })

      // Keep the reflected `view` attribute synced to the engine view so the
      // host CSS layout branches follow programmatic setView() too.
      const stopView = effect(() => {
        const v = view()
        if (host.getAttribute('view') !== v) host.setAttribute('view', v)
      })

      const active = (): Calendar => cal()

      // Deduped emit (sail-combobox lastEmitted pattern) over host.emit!. When
      // calendar swaps the engine identity we re-seed from the NEW engine
      // (no emit) instead of diffing its view/date against the prior engine's —
      // otherwise the stale closure fires a spurious sail-view/date-change.
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

      const onShadowClick = (e: Event) => {
        const t = e.target as HTMLElement | null
        if (!t) return
        const cal = active()

        const viewBtn = t.closest?.('[data-view]') as HTMLElement | null
        if (viewBtn) {
          cal.setView(viewBtn.dataset.view as ViewMode)
          return
        }
        const navBtn = t.closest?.('[data-nav]') as HTMLElement | null
        if (navBtn) {
          const act = navBtn.dataset.nav
          if (act === 'next') cal.next()
          else if (act === 'prev') cal.prev()
          else if (act === 'today') cal.today()
          return
        }
        const chip = t.closest?.('[data-event-id]') as HTMLElement | null
        if (chip) {
          const id = chip.dataset.eventId!
          cal.selectEvent(id, { additive: (e as MouseEvent).metaKey })
          host.emit!('sail-event-click', { id })
          host.emit!('sail-event-select', {
            id,
            selected: cal.selectedEventIds().includes(id),
            selectedIds: cal.selectedEventIds(),
          })
          return
        }
        const dayCell = t.closest?.('[role=gridcell][data-iso]') as HTMLElement | null
        if (dayCell) cal.goToDate(dayCell.dataset.iso!)
      }

      // Month-grid keyboard operability — the grid advertises role=grid/gridcell
      // with a roving tabindex, so the focused day cell must respond to arrow /
      // Home / End / PageUp-Down / Enter-Space. All moves route through the engine
      // (which owns the date math); DOM focus follows the new focus anchor.
      const onShadowKey = (e: KeyboardEvent) => {
        const cell = (e.target as HTMLElement | null)?.closest?.(
          '[role=gridcell][data-iso]',
        ) as HTMLElement | null
        if (!cell) return
        // Shift+Arrow is range extension — owned by the interactions wiring (which
        // moves focus AND extends the range as one move). Skip it here so focus is
        // not moved twice.
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
          ) as HTMLElement | null
          next?.focus()
        })
      }

      ;(host.shadowRoot ?? host).addEventListener('click', onShadowClick)
      ;(host.shadowRoot ?? host).addEventListener('keydown', onShadowKey as EventListener)

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
        ;(host.shadowRoot ?? host).removeEventListener('keydown', onShadowKey as EventListener)
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
      eventRect,
    }
  })

</controller>
<div sx={style.display(block).fontFamily(body).fontSize(density).bg(background).fg(foreground).borderWidth(1).border(border).radius(lg).shadow(xs).overflow(hidden)}>
  <row align="center" gap="density" sx={style.padY(density-row-py).padX(density-row-px).borderBWidth(1).borderB(border)}>
    <row gap="1" align="center">
      <button type="button" variant="outline" size="sm" data-nav="prev" aria-label="Previous">‹</button>
      <button type="button" variant="outline" size="sm" data-nav="today">Today</button>
      <button type="button" variant="outline" size="sm" data-nav="next" aria-label="Next">›</button>
    </row>
    <h2 grow sx={style.fontWeight(semibold).fontSize(base)}>{title()}</h2>
    <row role="group" aria-label="Calendar view" gap="1" align="center">
      <button type="button" variant="outline" size="sm" data-view="month" aria-pressed={isMonth() ? 'true' : 'false'}>Month</button>
      <button type="button" variant="outline" size="sm" data-view="week" aria-pressed={isWeek() ? 'true' : 'false'}>Week</button>
      <button type="button" variant="outline" size="sm" data-view="day" aria-pressed={isDay() ? 'true' : 'false'}>Day</button>
    </row>
  </row>

  <if {isEmpty()}>
    <div role="status" sx={style.fg(muted-foreground).padY(6).padX(density-row-px).textAlign(center)}>
      <slot name="empty">No events</slot>
    </div>
  </if>

  <if {isMonth()}>
  <div role="grid" aria-label={label() || 'Calendar month'}>
    <grid cols="7" gap="0" role="presentation">
      <for {weekdays()} as="d" key={d.key}>
        <span role="columnheader" sx={style.fg(muted-foreground).fontWeight(medium).padY(density-row-py).textAlign(center).fontSize(density).borderBWidth(1).borderB(border)}>{d.label}</span>
      </for>
    </grid>
    <for {weekRows()} as="wk" key={wk.key}>
      <grid cols="7" gap="0" role="row">
        <for {wk.cells} as="cr" key={cr.key}>
          <column
            role="gridcell"
            data-iso={cr.cell.iso}
            tabindex={cr.cell.iso === focusedIso() ? '0' : '-1'}
            data-state={cr.state}
            aria-current={cr.cell.today ? 'date' : null}
            aria-label={cr.cell.iso}
            gap="1"
            sx={style.group().padY(density-row-py).padX(density-row-px).minH(panel-min-h-cell).borderRWidth(1).borderR(border).borderBWidth(1).borderB(border).when({ stateOut: style.fg(muted-foreground).bg(muted), stateWeekend: style.bg(muted), stateRange: style.bg(selected-bg) })}
          >
            <row align="center" justify="center" gap="0" sx={style.fontSize(density).minW(control-h-2xs).h(control-h-2xs).radius(full).when({ groupStateToday: style.fg(primary).fontWeight(semibold).outline(primary) })}>{cr.cell.day}</row>
            <for {cr.cell.events} as="ev" key={ev.id}>
              <button
                type="button"
                data-event-id={ev.id}
                data-state={ev.allDay ? 'allday' : ''}
                aria-selected={selectedIds().includes(ev.id) ? 'true' : 'false'}
                title={ev.title}
                sx={style.position(relative).textAlign(start).cursor(pointer).fontSize(density).fg(foreground).bg(hover-bg).radius(sm).padY(half).padX(1_5).padL(3).truncate().when({ stateAllday: style.bg(hover-bg), selected: style.outline(ring) })}
              >
                <span aria-hidden="true" sx={style.position(absolute).left(half).top(1).bottom(1).w(half).radius(sm).bg(primary).when({ groupStateAllday: style.bg(accent) })}></span>
                {ev.title}
              </button>
            </for>
            <if {cr.cell.overflowCount > 0}>
              <span sx={style.fontSize(xs).fontWeight(semibold).fg(muted-foreground).padX(1)}>+{cr.cell.overflowCount} more</span>
            </if>
          </column>
        </for>
      </grid>
    </for>
  </div>
</if>

<if {isWeek() || isDay()}>
  <grid use:gridTracks={timeGridTracks()} tracks="--cal-time-tracks" gap="0" role="grid" aria-label={label() || 'Calendar time grid'}>
    <column role="rowgroup" gap="0" sx={style.borderRWidth(1).borderR(border)}>
      <for {timeSlots()} as="slot" key={slot.key}>
        <div data-slot={slot.minutes} role="rowheader" sx={style.h(control-h-md).fg(muted-foreground).padX(density-row-px).fontSize(density).borderBWidth(1).borderB(border)}>{slot.label}</div>
      </for>
    </column>
    <row gap="0" grow>
      <for {dayColumns()} as="dc" key={dc.key}>
        <div data-state={dc.col.today ? 'today' : ''} data-iso={dc.col.iso} grow sx={style.position(relative).minH(panel-min-h-daycol).borderRWidth(1).borderR(border).when({ stateToday: style.bg(selected-bg) })}>
          <column gap="0" sx={style.position(absolute).top(0).right(0).bottom(0).left(0)}>
            <for {timeSlots()} as="slot" key={slot.key}>
              <div data-iso={dc.col.iso} data-minutes={slot.minutes} grow sx={style.minH(control-h-md).cursor(cell).borderBWidth(1).borderB(border).when({ hover: style.bg(hover-bg) })}></div>
            </for>
          </column>
          <column gap="1" sx={style.position(relative).z(elevated).pad(half)}>
            <for {dc.col.allDayEvents} as="ev" key={ev.id}>
              <button
                type="button"
                data-event-id={ev.id}
                data-state="allday"
                aria-selected={selectedIds().includes(ev.id) ? 'true' : 'false'}
                title={ev.title}
                sx={style.position(relative).textAlign(start).cursor(pointer).fontSize(density).fg(foreground).bg(hover-bg).radius(sm).padY(half).padX(1_5).truncate().when({ selected: style.outline(ring) })}
              >{ev.title}</button>
            </for>
          </column>
          <for {dc.col.laidOut} as="pe" key={pe.event.id}>
            <button
              type="button"
              data-event-id={pe.event.id}
              aria-selected={selectedIds().includes(pe.event.id) ? 'true' : 'false'}
              use:eventSpan={eventRect(pe)}
              title={pe.event.title}
              sx={style.position(absolute).top(bind(evtTop)).h(bind(evtH)).left(bind(evtLeft)).w(bind(evtW)).cursor(pointer).textAlign(start).fontSize(density).fg(primary).bg(hover-bg).radius(sm).padY(half).padX(1).overflow(hidden).when({ selected: style.outline(ring) })}
            >{pe.event.title}</button>
          </for>
        </div>
      </for>
    </row>
  </grid>
</if>
</div>
