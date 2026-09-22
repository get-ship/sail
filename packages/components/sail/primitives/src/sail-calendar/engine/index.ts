// sail-calendar engine — public barrel (Slice 1). Pure reactive state factory +
// its types. The view (Slice 5) consumes `createCalendar` via prop:calendar.

export { createCalendar } from './create-calendar'
export type { Calendar, CalendarOptions } from './contract'
export type {
  AgendaRow,
  AllDayEvent,
  AllDayLane,
  AllDaySpan,
  CalendarEvent,
  DateRange,
  DayColumn,
  EventBase,
  MonthCell,
  PositionedEvent,
  ReactiveCalendarState,
  TimeSlot,
  TimedEvent,
  ViewMode,
  WeekStart,
} from './types'
export { isAllDay, validateEvent, partitionEvents, touchesDay, eventsInRange } from './events'
export { layoutDay } from './layout'
export * from './date-math'
