// TESTS-FIRST — Slice 1b/2: pure view-model builders.

import { describe, expect, it } from 'vitest'
import {
  buildAllDayBand,
  buildDayColumns,
  buildMonthCells,
  buildTimeSlots,
  titleFor,
} from './view-model'
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

describe('buildMonthCells', () => {
  it('produces a fixed 6x7 = 42 grid honoring weekStartsOn', () => {
    const cells = buildMonthCells({
      year: 2026,
      month: 5,
      weekStartsOn: 0,
      today: '2026-06-15',
      events: [],
      selectedIds: [],
      focusedIso: null,
      cap: 3,
    })
    expect(cells).toHaveLength(42)
    expect(cells.filter((c) => c.inMonth)).toHaveLength(30) // June has 30 days
  })

  it('puts a multi-day all-day span in every covered cell (end exclusive)', () => {
    const cells = buildMonthCells({
      year: 2026,
      month: 5,
      weekStartsOn: 0,
      today: '2026-06-15',
      events: [allDay('a', '2026-06-02', '2026-06-05')],
      selectedIds: [],
      focusedIso: null,
      cap: 3,
    })
    const has = (iso: string) =>
      cells.find((c) => c.iso === iso)!.events.some((e) => e.id === 'a')
    expect(has('2026-06-02')).toBe(true)
    expect(has('2026-06-04')).toBe(true)
    expect(has('2026-06-05')).toBe(false) // end exclusive
  })

  it('caps events per cell and reports overflowCount', () => {
    const events: CalendarEvent[] = [
      timed('a', '2026-06-02T09:00', '2026-06-02T10:00'),
      timed('b', '2026-06-02T10:00', '2026-06-02T11:00'),
      timed('c', '2026-06-02T11:00', '2026-06-02T12:00'),
      timed('d', '2026-06-02T12:00', '2026-06-02T13:00'),
      timed('e', '2026-06-02T13:00', '2026-06-02T14:00'),
    ]
    const cells = buildMonthCells({
      year: 2026,
      month: 5,
      weekStartsOn: 0,
      today: '2026-06-15',
      events,
      selectedIds: [],
      focusedIso: null,
      cap: 3,
    })
    const cell = cells.find((c) => c.iso === '2026-06-02')!
    expect(cell.events).toHaveLength(3)
    expect(cell.overflowCount).toBe(2)
  })

  it('marks today + selected + focused', () => {
    const cells = buildMonthCells({
      year: 2026,
      month: 5,
      weekStartsOn: 0,
      today: '2026-06-15',
      events: [timed('a', '2026-06-15T09:00', '2026-06-15T10:00')],
      selectedIds: ['a'],
      focusedIso: '2026-06-15',
      cap: 3,
    })
    const cell = cells.find((c) => c.iso === '2026-06-15')!
    expect(cell.today).toBe(true)
    expect(cell.selected).toBe(true)
    expect(cell.focused).toBe(true)
  })
})

describe('buildTimeSlots', () => {
  it('emits dayStartHour..dayEndHour at slotMinutes granularity', () => {
    const slots = buildTimeSlots(9, 12, 30)
    expect(slots).toHaveLength(6) // 9,9:30,10,10:30,11,11:30
    expect(slots[0]).toMatchObject({ index: 0, minutes: 540, label: '09:00' })
    expect(slots[2]).toMatchObject({ minutes: 600, label: '10:00' })
    expect(slots[1]!.label).toBe('09:30')
  })
})

describe('buildDayColumns', () => {
  it('lays out per-day timed events with overlap geometry', () => {
    const cols = buildDayColumns({
      days: ['2026-06-02'],
      today: '2026-06-02',
      events: [
        timed('a', '2026-06-02T09:00', '2026-06-02T11:00'),
        timed('b', '2026-06-02T09:30', '2026-06-02T10:30'),
        allDay('c', '2026-06-02'),
      ],
      windowStart: 0,
      windowEnd: 1440,
    })
    expect(cols).toHaveLength(1)
    expect(cols[0]!.today).toBe(true)
    expect(cols[0]!.laidOut).toHaveLength(2)
    expect(cols[0]!.laidOut.every((p) => p.laneCount === 2)).toBe(true)
    expect(cols[0]!.allDayEvents.map((e) => e.id)).toEqual(['c'])
  })
})

describe('buildAllDayBand', () => {
  it('packs overlapping multi-day spans into separate lanes', () => {
    const days = ['2026-06-01', '2026-06-02', '2026-06-03', '2026-06-04']
    const band = buildAllDayBand({
      days,
      events: [allDay('a', '2026-06-01', '2026-06-03'), allDay('b', '2026-06-02', '2026-06-04')],
    })
    expect(band).toHaveLength(2)
    expect(band[0]!.spans[0]!.event.id).toBe('a')
    // a spans col 0..1 (days 01,02 — end 03 exclusive)
    expect(band[0]!.spans[0]).toMatchObject({ startCol: 0, endCol: 1 })
    expect(band[1]!.spans[0]!.event.id).toBe('b')
  })

  it('reuses a lane for non-overlapping spans', () => {
    const days = ['2026-06-01', '2026-06-02', '2026-06-03', '2026-06-04']
    const band = buildAllDayBand({
      days,
      events: [allDay('a', '2026-06-01', '2026-06-02'), allDay('b', '2026-06-03', '2026-06-04')],
    })
    expect(band).toHaveLength(1)
    expect(band[0]!.spans).toHaveLength(2)
  })
})

describe('titleFor', () => {
  it('month → "June 2026"', () => {
    expect(titleFor('month', '2026-06-15', 0)).toBe('June 2026')
  })
  it('day → weekday + date', () => {
    expect(titleFor('day', '2026-06-02', 0)).toBe('Tue, Jun 2, 2026')
  })
  it('week → range', () => {
    // Sunday-start week of 2026-06-02 is 2026-05-31 .. 2026-06-06
    expect(titleFor('week', '2026-06-02', 0)).toBe('May 31 – Jun 6, 2026')
  })
})
