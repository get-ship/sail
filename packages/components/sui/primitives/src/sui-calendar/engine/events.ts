// sui-calendar engine — events model (Slice 1b). PURE transforms.
//
// The `CalendarEvent = AllDayEvent | TimedEvent` discriminated union, its
// validation, the all-day/timed partition, per-day touch test (half-open
// multi-day spans), and the visible-window filter. No signals, no DOM — the
// reactive lift happens in create-calendar.ts; these are the pure predicates
// the derived chain composes.

import { parseIso, parseIsoDateTime } from './date-math'
import type { AllDayEvent, CalendarEvent, ISODate, TimedEvent } from './types'

export interface ValidationResult {
  ok: boolean
  reason?: string
}

const isDateOnly = (s: unknown): s is ISODate => typeof s === 'string' && parseIso(s) !== null
const isDateTime = (s: unknown): boolean => typeof s === 'string' && parseIsoDateTime(s) !== null

/** Discriminate the union by the `allDay` tag. */
export const isAllDay = (e: CalendarEvent): e is AllDayEvent => e.allDay === true

/**
 * Validate one event against the union shape:
 *  - all-day: `start` (and optional `end`) are date-only; end >= start.
 *  - timed: `start`+`end` are datetimes; end > start (zero-length rejected).
 * Mixed shapes (date-only start on a timed event, datetime start on all-day)
 * are rejected.
 */
export function validateEvent(e: CalendarEvent): ValidationResult {
  if (!e || typeof e.id !== 'string' || e.id.length === 0)
    return { ok: false, reason: 'event requires a non-empty string id' }

  if (e.allDay === true) {
    if (!isDateOnly(e.start)) return { ok: false, reason: 'all-day start must be YYYY-MM-DD' }
    if (e.end !== undefined) {
      if (!isDateOnly(e.end)) return { ok: false, reason: 'all-day end must be YYYY-MM-DD' }
      if (e.end < e.start) return { ok: false, reason: 'all-day end is before start' }
    }
    return { ok: true }
  }

  // timed
  if (!isDateTime(e.start)) return { ok: false, reason: 'timed start must be YYYY-MM-DDThh:mm' }
  if (!isDateTime((e as TimedEvent).end))
    return { ok: false, reason: 'timed end must be YYYY-MM-DDThh:mm' }
  if ((e as TimedEvent).end <= e.start)
    return { ok: false, reason: 'timed end must be strictly after start' }
  return { ok: true }
}

export interface Partitioned {
  allDayEvents: AllDayEvent[]
  timedEvents: TimedEvent[]
}

export function partitionEvents(events: CalendarEvent[]): Partitioned {
  const allDayEvents: AllDayEvent[] = []
  const timedEvents: TimedEvent[] = []
  for (const e of events) {
    if (isAllDay(e)) allDayEvents.push(e)
    else timedEvents.push(e)
  }
  return { allDayEvents, timedEvents }
}

/** The calendar day (ISODate) an event's start falls on. */
export const eventStartDay = (e: CalendarEvent): ISODate => e.start.slice(0, 10)
/** The calendar day an event's end falls on (timed) or its exclusive end (all-day). */
export const eventEndDay = (e: CalendarEvent): ISODate =>
  isAllDay(e) ? e.end ?? addOneDay(e.start) : e.end.slice(0, 10)

const addOneDay = (iso: ISODate): ISODate => {
  const p = parseIso(iso)
  if (!p) return iso
  const d = new Date(p.y, p.m, p.d + 1)
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`
}
const pad = (n: number) => (n < 10 ? `0${n}` : String(n))

/**
 * Does `event` touch calendar day `iso`?
 * - all-day multi-day spans are half-open [start, end) — the end day is NOT
 *   covered; a single-day all-day (no end) touches only its start day.
 * - timed events touch every calendar day between their start and end day
 *   (inclusive, since end is a concrete instant on that day).
 */
export function touchesDay(event: CalendarEvent, iso: ISODate): boolean {
  const startDay = eventStartDay(event)
  if (isAllDay(event)) {
    const endExclusive = event.end ?? addOneDay(startDay)
    return iso >= startDay && iso < endExclusive
  }
  const endDay = event.end.slice(0, 10)
  return iso >= startDay && iso <= endDay
}

/** Keep events that intersect the inclusive [from, to] day window. */
export function eventsInRange(
  events: CalendarEvent[],
  from: ISODate,
  to: ISODate,
): CalendarEvent[] {
  return events.filter((e) => {
    const startDay = eventStartDay(e)
    const endDay = eventEndDay(e)
    // half-open all-day end is the exclusive day; treat the last covered day as endDay-1.
    const lastDay = isAllDay(e) && e.end ? prevDay(endDay) : endDay
    return startDay <= to && lastDay >= from
  })
}

const prevDay = (iso: ISODate): ISODate => {
  const p = parseIso(iso)
  if (!p) return iso
  const d = new Date(p.y, p.m, p.d - 1)
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`
}
