// TESTS-FIRST — Slice 1b: events model (union validation + partition + window).

import { describe, expect, it } from 'vitest'
import {
  isAllDay,
  partitionEvents,
  touchesDay,
  validateEvent,
  eventsInRange,
} from './events'
import type { AllDayEvent, CalendarEvent, TimedEvent } from './types'

const allDay = (id: string, start: string, end?: string): AllDayEvent => ({
  id,
  title: id,
  allDay: true,
  start,
  ...(end ? { end } : {}),
})
const timed = (id: string, start: string, end: string): TimedEvent => ({
  id,
  title: id,
  start,
  end,
})

describe('event union validation', () => {
  it('accepts a well-formed all-day event', () => {
    expect(validateEvent(allDay('a', '2026-06-02')).ok).toBe(true)
  })

  it('accepts a well-formed timed event (end > start)', () => {
    expect(validateEvent(timed('a', '2026-06-02T09:00', '2026-06-02T10:00')).ok).toBe(true)
  })

  it('rejects a timed event with end <= start (zero-length / inverted)', () => {
    expect(validateEvent(timed('a', '2026-06-02T10:00', '2026-06-02T10:00')).ok).toBe(false)
    expect(validateEvent(timed('a', '2026-06-02T11:00', '2026-06-02T10:00')).ok).toBe(false)
  })

  it('rejects a timed event whose start is a date-only (mixed shape)', () => {
    const bad = { id: 'x', title: 'x', start: '2026-06-02', end: '2026-06-02T10:00' } as CalendarEvent
    expect(validateEvent(bad).ok).toBe(false)
  })

  it('rejects an all-day event with a datetime start (mixed shape)', () => {
    const bad = { id: 'x', title: 'x', allDay: true, start: '2026-06-02T09:00' } as CalendarEvent
    expect(validateEvent(bad).ok).toBe(false)
  })

  it('rejects an all-day event whose end is before start', () => {
    expect(validateEvent(allDay('a', '2026-06-05', '2026-06-02')).ok).toBe(false)
  })
})

describe('partition + touches', () => {
  it('partitions all-day vs timed', () => {
    const { allDayEvents, timedEvents } = partitionEvents([
      allDay('a', '2026-06-02'),
      timed('b', '2026-06-02T09:00', '2026-06-02T10:00'),
    ])
    expect(allDayEvents.map((e) => e.id)).toEqual(['a'])
    expect(timedEvents.map((e) => e.id)).toEqual(['b'])
  })

  it('isAllDay discriminates the union', () => {
    expect(isAllDay(allDay('a', '2026-06-02'))).toBe(true)
    expect(isAllDay(timed('b', '2026-06-02T09:00', '2026-06-02T10:00'))).toBe(false)
  })

  it('touchesDay: multi-day all-day span [start,end) covers every interior day', () => {
    const span = allDay('a', '2026-06-02', '2026-06-05') // covers 02,03,04 (end exclusive)
    expect(touchesDay(span, '2026-06-02')).toBe(true)
    expect(touchesDay(span, '2026-06-04')).toBe(true)
    expect(touchesDay(span, '2026-06-05')).toBe(false) // end exclusive
    expect(touchesDay(span, '2026-06-01')).toBe(false)
  })

  it('touchesDay: a timed event touches its own calendar day', () => {
    const t = timed('b', '2026-06-02T09:00', '2026-06-02T10:00')
    expect(touchesDay(t, '2026-06-02')).toBe(true)
    expect(touchesDay(t, '2026-06-03')).toBe(false)
  })
})

describe('eventsInRange windowing', () => {
  it('keeps events that intersect the [from,to] day window', () => {
    const events = [
      allDay('a', '2026-05-30'),
      allDay('b', '2026-06-02'),
      timed('c', '2026-06-10T09:00', '2026-06-10T10:00'),
    ]
    const out = eventsInRange(events, '2026-06-01', '2026-06-07')
    expect(out.map((e) => e.id)).toEqual(['b'])
  })
})
