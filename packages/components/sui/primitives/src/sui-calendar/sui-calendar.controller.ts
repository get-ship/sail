// <sui-calendar> controller — Slice 2 (the VIEW). Glue-only / tier-2.
//
// The headless engine (`./engine/createCalendar`) owns ALL reactive state +
// view-model derivation (month/week/day, agenda, selection, navigation). This
// controller is the thin seam: it routes the live `Calendar` in via the
// `prop:calendar` IDENTITY path (a JS reference, never a stringified attribute),
// falls back to building an engine from the `events` `type="string"` SSR prop
// (sui-combobox JSON.parse pattern), re-exposes the engine's `Derived` surfaces
// as scope signals the template reads, and emits `sui-*` events via the runtime
// `host.emit!` helper (NOT open-coded `dispatchEvent`).
//
// Cardinals honored: mutations only in on:* / mount; no fetch / ship.* / <data>
// (engine purity asserted by engine/purity.test.ts). Range selection +
// drag-to-create/move ride the engine's range/intent mutators; the DOM bridge
// (pointer drag + Shift+Arrow → engine + sui-* emits) lives in `./sui-calendar.interactions`.

import { controller, derived, effect, signal } from '@ship/sui-runtime'
import { createCalendar } from './engine'
import type { Calendar, CalendarEvent, DateRange, MonthCell, ViewMode } from './engine'
import { wireCalendarInteractions } from './sui-calendar.interactions'
import type { Scope } from './sui-calendar.sui'

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

export default controller<Scope>({
  scope: ({
    events,
    focusDate,
    weekStartsOn,
    slotMinutes,
    dayStartHour,
    dayEndHour,
    rangeMode,
  }) => {
    // The live engine. `prop:calendar` identity (set in mount) supersedes; until
    // then we seed from the SSR `events` string + the focus-date attr so a
    // declarative (no-parent-controller) mount still renders.
    const engine = signal<Calendar | null>(null)

    const fallback = createCalendar({
      events: derived(() => parseJsonEvents(events())),
      initialDate: focusDate() || undefined,
      weekStartsOn: (weekStartsOn() || 0) as 0,
      slotMinutes: slotMinutes() || 30,
      dayStartHour: dayStartHour() ?? 0,
      dayEndHour: dayEndHour() ?? 24,
      rangeMode: (rangeMode() || 'none') as 'none',
    })

    /** The active engine — the prop:calendar identity if present, else fallback. */
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
    type CellRow = { key: string; iso: string; cell: MonthCell; inRange: boolean }
    const weekRows = derived<{ key: string; cells: CellRow[] }[]>(() => {
      const cells = cal().monthCells()
      const sel = cal().selectedEventIds()
      const range = cal().range()
      const sig = (c: MonthCell): string =>
        `${c.iso}|${c.today ? 't' : ''}|${c.events.map((e) => e.id).join(',')}|${c.overflowCount}|${sel.join(',')}`
      const rows: { key: string; cells: CellRow[] }[] = []
      for (let row = 0; row < 6; row++) {
        const slice = cells.slice(row * 7, row * 7 + 7)
        const cellRows = slice.map((c) => ({
          key: sig(c),
          iso: c.iso,
          cell: c,
          inRange: inRangeIso(c.iso, range),
        }))
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
    }
  },

  mount(state, host) {
    // prop:calendar identity — a live `Calendar` reference (the engine path),
    // routed into the scope signal so every derived re-resolves onto it.
    Object.defineProperty(host, 'calendar', {
      configurable: true,
      get: () => (state as unknown as { $signals: { engine: () => Calendar | null } }).$signals.engine(),
      set: (c: Calendar | null) => {
        (state as unknown as { engine: Calendar | null }).engine = c
      },
    })

    // Keep the reflected `view` attribute synced to the engine view so the
    // host CSS layout branches follow programmatic setView() too.
    const stopView = effect(() => {
      const v = (state as unknown as { $signals: { view: () => ViewMode } }).$signals.view()
      if (host.getAttribute('view') !== v) host.setAttribute('view', v)
    })

    const active = (): Calendar => (state as unknown as { $signals: { cal: () => Calendar } }).$signals.cal()

    // Deduped emit (sui-combobox lastEmitted pattern) over host.emit!. When
    // prop:calendar swaps the engine identity we re-seed from the NEW engine
    // (no emit) instead of diffing its view/date against the prior engine's —
    // otherwise the stale closure fires a spurious sui-view/date-change.
    let lastEngine: Calendar | null = null
    let lastView: ViewMode | null = null
    let lastDate: string | null = null
    const stopEmit = effect(() => {
      const cal = active()
      const v = cal.view()
      const d = cal.focusDate()
      if (cal === lastEngine) {
        if (lastView !== null && v !== lastView) host.emit!('sui-view-change', { view: v })
        if (lastDate !== null && d !== lastDate) host.emit!('sui-date-change', { date: d })
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
        host.emit!('sui-event-click', { id })
        host.emit!('sui-event-select', {
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
        const next = host.shadowRoot?.querySelector(
          `[role=gridcell][data-iso="${iso}"]`,
        ) as HTMLElement | null
        next?.focus()
      })
    }

    host.shadowRoot?.addEventListener('click', onShadowClick)
    host.shadowRoot?.addEventListener('keydown', onShadowKey as EventListener)

    // Range selection (month drag + Shift+Arrow) + drag-to-create/move (time-
    // grid slot drag). The DOM↔engine bridge lives in the interactions module;
    // it reads the engine fresh per event and emits sui-range-change /
    // sui-event-create-intent / sui-event-move, auto-cleaning against the mount
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
      host.shadowRoot?.removeEventListener('click', onShadowClick)
      host.shadowRoot?.removeEventListener('keydown', onShadowKey as EventListener)
      delete (host as { calendar?: unknown }).calendar
    }
  },
})
