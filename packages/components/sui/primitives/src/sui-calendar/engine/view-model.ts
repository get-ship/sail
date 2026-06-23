// sui-calendar engine — pure view-model builders (Slice 1b/2). No signals/DOM.
//
// monthCells (fixed 6×7), timeSlots (the virtualizer count source), dayColumns
// (per-day overlap lane layout), the all-day band (multi-day span packing), and
// the header title. Each is a pure function of (focus, events, options) so the
// derived chain in create-calendar.ts only re-runs the relevant suffix.

import {
  addDays,
  dayOfWeek,
  monthGrid,
  startOfWeek,
  ymd,
  type ISODate,
  type WeekStart,
} from './date-math'
import { partitionEvents, touchesDay } from './events'
import { layoutDay } from './layout'
import type {
  AllDayEvent,
  AllDayLane,
  CalendarEvent,
  DayColumn,
  MonthCell,
  TimeSlot,
  TimedEvent,
  ViewMode,
} from './types'

const MONTHS = [
  'January', 'February', 'March', 'April', 'May', 'June',
  'July', 'August', 'September', 'October', 'November', 'December',
]
const MON_SHORT = MONTHS.map((m) => m.slice(0, 3))
const DOW_SHORT = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat']

const pad2 = (n: number) => (n < 10 ? `0${n}` : String(n))

// ── Month grid ────────────────────────────────────────────────────────────────

export interface MonthCellInput {
  year: number
  month: number // 0-based
  weekStartsOn: WeekStart
  today: ISODate
  events: CalendarEvent[]
  selectedIds: string[]
  focusedIso: ISODate | null
  cap: number
}

export function buildMonthCells(input: MonthCellInput): MonthCell[] {
  const { year, month, weekStartsOn, today, events, selectedIds, focusedIso, cap } = input
  const sel = new Set(selectedIds)
  return monthGrid(year, month, weekStartsOn).map((g) => {
    const touching = events.filter((e) => touchesDay(e, g.iso))
    const shown = cap > 0 ? touching.slice(0, cap) : touching
    return {
      iso: g.iso,
      day: g.day,
      inMonth: g.inMonth,
      today: g.iso === today,
      isWeekend: g.isWeekend,
      events: shown,
      overflowCount: Math.max(0, touching.length - shown.length),
      selected: shown.some((e) => sel.has(e.id)),
      focused: focusedIso === g.iso,
    }
  })
}

// ── Time slots ────────────────────────────────────────────────────────────────

export function buildTimeSlots(
  dayStartHour: number,
  dayEndHour: number,
  slotMinutes: number,
): TimeSlot[] {
  const startM = dayStartHour * 60
  const endM = dayEndHour * 60
  const step = Math.max(1, slotMinutes)
  const out: TimeSlot[] = []
  let index = 0
  for (let m = startM; m < endM; m += step) {
    const hh = Math.floor(m / 60)
    const mm = m % 60
    out.push({
      key: `slot-${m}`,
      index,
      minutes: m,
      label: `${pad2(hh)}:${pad2(mm)}`,
    })
    index++
  }
  return out
}

// ── Day columns (week/day) ────────────────────────────────────────────────────

export interface DayColumnInput {
  days: ISODate[]
  today: ISODate
  events: CalendarEvent[]
  windowStart: number // minutes from midnight
  windowEnd: number
}

/** Clip a (possibly midnight-spanning) timed event to one calendar day so
 *  layoutDay sees a same-day interval. The id is preserved (selection keys on
 *  it); the day-N+1 portion of a multi-day event becomes a 00:00 start, the
 *  day-N portion a 23:59 end. */
function clipTimedToDay(e: TimedEvent, iso: ISODate): TimedEvent {
  const start = e.start.slice(0, 10) === iso ? e.start : `${iso}T00:00`
  const end = e.end.slice(0, 10) === iso ? e.end : `${iso}T23:59`
  return { ...e, start, end }
}

export function buildDayColumns(input: DayColumnInput): DayColumn[] {
  const { days, today, events, windowStart, windowEnd } = input
  const { allDayEvents, timedEvents } = partitionEvents(events)
  return days.map((iso) => {
    const dayTimed = timedEvents.filter((e) => touchesDay(e, iso)).map((e) => clipTimedToDay(e, iso))
    const dayAllDay = allDayEvents.filter((e) => touchesDay(e, iso))
    return {
      iso,
      today: iso === today,
      allDayEvents: dayAllDay,
      laidOut: layoutDay(dayTimed as TimedEvent[], windowStart, windowEnd),
    }
  })
}

// ── All-day band (multi-day span lane packing) ────────────────────────────────

export interface AllDayBandInput {
  days: ISODate[]
  events: CalendarEvent[]
}

export function buildAllDayBand(input: AllDayBandInput): AllDayLane[] {
  const { days, events } = input
  const allDay = events.filter((e): e is AllDayEvent => e.allDay === true)

  // Compute each span's [startCol, endCol] within the visible day window.
  interface Span {
    event: AllDayEvent
    startCol: number
    endCol: number
  }
  const spans: Span[] = []
  for (const e of allDay) {
    let startCol = days.length
    let endCol = -1
    for (let i = 0; i < days.length; i++) {
      if (touchesDay(e, days[i]!)) {
        if (i < startCol) startCol = i
        if (i > endCol) endCol = i
      }
    }
    if (endCol >= 0) spans.push({ event: e, startCol, endCol })
  }
  // Deterministic order: earliest start, then widest, then id.
  spans.sort(
    (a, b) =>
      a.startCol - b.startCol ||
      b.endCol - b.startCol - (a.endCol - a.startCol) ||
      cmp(a.event.id, b.event.id),
  )

  const laneLastCol: number[] = [] // laneLastCol[k] = last occupied col in lane k
  const lanes: AllDayLane[] = []
  for (const s of spans) {
    let lane = laneLastCol.findIndex((last) => last < s.startCol)
    if (lane === -1) {
      lane = laneLastCol.length
      laneLastCol.push(s.endCol)
      lanes.push({ laneIndex: lane, spans: [] })
    } else {
      laneLastCol[lane] = s.endCol
    }
    lanes[lane]!.spans.push({ event: s.event, startCol: s.startCol, endCol: s.endCol })
  }
  return lanes
}

// ── Title ─────────────────────────────────────────────────────────────────────

export function titleFor(view: ViewMode, focusIso: ISODate, weekStartsOn: WeekStart): string {
  const [y, m, d] = ymd(focusIso)
  if (view === 'month') return `${MONTHS[m - 1]} ${y}`
  if (view === 'day') {
    const dow = DOW_SHORT[dayOfWeek(focusIso)]
    return `${dow}, ${MON_SHORT[m - 1]} ${d}, ${y}`
  }
  // week
  const start = startOfWeek(focusIso, weekStartsOn)
  const end = addDays(start, 6)
  const [sy, sm, sd] = ymd(start)
  const [ey, em, ed] = ymd(end)
  const left = `${MON_SHORT[sm - 1]} ${sd}`
  const right = sm === em ? `${ed}` : `${MON_SHORT[em - 1]} ${ed}`
  const year = ey === sy ? ey : `${sy}–${ey}`
  return `${left} – ${right}, ${year}`
}

const cmp = (a: string, b: string): number => (a < b ? -1 : a > b ? 1 : 0)
