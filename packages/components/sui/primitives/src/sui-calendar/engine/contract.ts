// sui-calendar engine — the createCalendar() public contract (Backbone #1).
//
// `CalendarOptions` (input) + the `Calendar` facade the assembler returns.
// PURE TypeScript — type-only. Imports the Signal/Derived handles from
// `@ship/sail-runtime` and the shared shapes from `./types`. `types.ts` and
// `contract.ts` are the one source of truth for the surface (no parallel defn).

import type { Derived, Signal } from '@ship/sail-runtime'
import type {
  AgendaRow,
  AllDayLane,
  CalendarEvent,
  DateRange,
  DayColumn,
  ISODate,
  ISODateTime,
  MonthCell,
  RangeMode,
  ReactiveCalendarState,
  TimeSlot,
  ViewMode,
  WeekStart,
} from './types'

/** A drag-to-create intent — the consumer decides whether to add the event. */
export interface EventCreateIntent {
  start: ISODateTime
  end: ISODateTime
}
/** A drag-to-move intent — the consumer decides whether to commit the move. */
export interface EventMoveIntent {
  id: string
  start: ISODateTime
  end: ISODateTime
}

export interface CalendarOptions {
  /** Raw array OR a live Signal/Derived identity (growing it adds events). */
  events: CalendarEvent[] | Signal<CalendarEvent[]> | Derived<CalendarEvent[]>
  /** default `e => e.id`; selection/{#for}-key/drag identity all key off it. */
  getEventId?: (e: CalendarEvent) => string
  initialView?: ViewMode
  /** focus anchor; defaults to today. */
  initialDate?: ISODate
  weekStartsOn?: WeekStart
  /** day/week grid granularity in minutes; default 30. */
  slotMinutes?: number
  dayStartHour?: number
  dayEndHour?: number
  /** max events shown per month cell before an "+N more" overflow. */
  monthCellCap?: number
  /** range-selection granularity. `none` (default) disables the range mutators;
   *  `date` stores ISODate bounds (month/week day-grid drag); `datetime` stores
   *  ISODateTime bounds (week/day time-grid drag). */
  rangeMode?: RangeMode
  // callbacks (engine → consumer; consumer writes its own arg signals).
  onViewChange?: (v: ViewMode) => void
  onDateChange?: (d: ISODate) => void
  onSelectionChange?: (ids: string[]) => void
  /** the committed range changed (drag end / Shift+Arrow / clear). */
  onRangeChange?: (range: DateRange | null) => void
  /** drag-on-empty-time produced a create intent (consumer adds the event). */
  onEventCreateIntent?: (intent: EventCreateIntent) => void
  /** drag-an-event produced a move intent (consumer commits the new bounds). */
  onEventMoveIntent?: (intent: EventMoveIntent) => void
}

export interface Calendar {
  // View-model render surfaces (Derived — read in template/derived).
  view: Derived<ViewMode>
  focusDate: Derived<ISODate>
  title: Derived<string>
  monthCells: Derived<MonthCell[]>
  dayColumns: Derived<DayColumn[]>
  timeSlots: Derived<TimeSlot[]>
  allDayBand: Derived<AllDayLane[]>
  agenda: Derived<AgendaRow[]>
  range: Derived<DateRange | null>
  /** The resolved range-selection mode (the view gates its range wiring on it). */
  rangeMode: RangeMode
  selectedEventIds: Derived<string[]>
  isEmpty: Derived<boolean>
  // Centralized reactive state (each slice a Signal).
  state: ReactiveCalendarState
  // Mutators — call ONLY from on:* / mount (Cardinal 3).
  setView(v: ViewMode): void
  goToDate(d: ISODate): void
  /** Move the focus anchor by whole days (keyboard grid nav). */
  moveFocus(deltaDays: number): void
  /** Move the focus anchor to the start (false) or end (true) of its week. */
  focusWeekEdge(end: boolean): void
  today(): void
  next(): void
  prev(): void
  selectEvent(id: string, opts?: { additive?: boolean }): void
  clearSelection(): void
  /** Set the range to [start, end] (chronologically normalized). No-op when
   *  `rangeMode` is `none`. Emits `onRangeChange`. */
  selectRange(start: ISODate | ISODateTime, end: ISODate | ISODateTime): void
  /** Start a drag/keyboard range from `anchor` (range = [anchor, anchor]). */
  beginRange(anchor: ISODate | ISODateTime): void
  /** Extend the active range to `to`, pivoting on the begin anchor. */
  extendRange(to: ISODate | ISODateTime): void
  /** Clear the range (sets the signal to null, emits `onRangeChange(null)`). */
  clearRange(): void
  /** Emit a drag-to-create intent for [start, end] (normalized). Engine never
   *  mutates its events array — the consumer decides whether to add. */
  createIntent(start: ISODateTime, end: ISODateTime): void
  /** Emit a drag-to-move intent for an existing event's new [start, end]. */
  moveEvent(id: string, start: ISODateTime, end: ISODateTime): void
}

export type {
  AgendaRow,
  AllDayLane,
  CalendarEvent,
  DateRange,
  DayColumn,
  ISODate,
  ISODateTime,
  MonthCell,
  RangeMode,
  ReactiveCalendarState,
  TimeSlot,
  ViewMode,
  WeekStart,
} from './types'
