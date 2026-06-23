// sui-calendar engine — shared shapes (Slice 1a).
//
// PURE TypeScript — type-only. The event union, the view-model render shapes,
// and the reactive-state slice handles. `contract.ts` carries the public
// `CalendarOptions`/`Calendar` surface and re-exports these so consumers have
// one import site (mirrors sui-table's types/contract split).

import type { Signal } from '@ship/sui-runtime'
import type { ISODate, ISODateTime } from './date-math'

export type { ISODate, ISODateTime } from './date-math'
export type ViewMode = 'month' | 'week' | 'day'
export type WeekStart = 0 | 1 | 2 | 3 | 4 | 5 | 6
/** Range-selection granularity. Mirrors the `range-mode` prop. */
export type RangeMode = 'none' | 'date' | 'datetime'

// ── Events model — a DISCRIMINATED UNION on `allDay` ──────────────────────────

export interface EventBase {
  id: string
  title: string
  meta?: Record<string, unknown>
  editable?: boolean
}

export interface AllDayEvent extends EventBase {
  allDay: true
  start: ISODate
  /** [start, end) inclusive-start exclusive-end multi-day span. */
  end?: ISODate
  startTime?: never
  endTime?: never
}

export interface TimedEvent extends EventBase {
  allDay?: false
  start: ISODateTime
  /** end > start enforced; zero-length rejected. */
  end: ISODateTime
}

export type CalendarEvent = AllDayEvent | TimedEvent

// ── View-model render shapes (the thing consumers read) ───────────────────────

export interface MonthCell {
  iso: ISODate
  day: number
  inMonth: boolean
  today: boolean
  isWeekend: boolean
  events: CalendarEvent[]
  overflowCount: number
  selected: boolean
  focused: boolean
}

export interface PositionedEvent {
  event: TimedEvent
  laneIndex: number
  laneCount: number
  topPct: number
  heightPct: number
}

export interface DayColumn {
  iso: ISODate
  today: boolean
  allDayEvents: CalendarEvent[]
  laidOut: PositionedEvent[]
}

export interface TimeSlot {
  key: string
  index: number
  minutes: number
  label: string
}

export interface AllDaySpan {
  event: AllDayEvent
  startCol: number
  endCol: number
}
export interface AllDayLane {
  laneIndex: number
  spans: AllDaySpan[]
}

export interface AgendaRow {
  key: string
  iso: ISODate
  kind: 'day-header' | 'event'
  event?: CalendarEvent
}

export interface DateRange {
  start: ISODate | ISODateTime
  end: ISODate | ISODateTime
}

// ── Reactive state slices (each is a Signal<…>) ───────────────────────────────

export interface ReactiveCalendarState {
  events: Signal<CalendarEvent[]>
  view: Signal<ViewMode>
  focusDate: Signal<ISODate>
  range: Signal<DateRange | null>
  selectedEventIds: Signal<string[]>
}
