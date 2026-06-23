// TESTS-FIRST — Slice 0: date-math characterization.
//
// Ports the proven sui-date-picker helpers (isoFor/parseIso/daysInMonth/
// addDays/clampIso) and pins the new calendar primitives (weekStart/monthGrid/
// addMinutes/rangeOverlaps + the ISODateTime helpers). Timezone is LOCAL
// WALL-CLOCK (see module header); a DST-gap case is `it.skip`-ed with reason.

import { describe, expect, it } from 'vitest'
import {
  addDays,
  addMinutes,
  clampIso,
  dateTimeMinutes,
  daysInMonth,
  diffMinutes,
  isoFor,
  monthGrid,
  parseIso,
  parseIsoDateTime,
  rangeOverlaps,
  startOfWeek,
  weekDays,
} from './date-math'

describe('date-math — ported date-picker primitives', () => {
  it('isoFor pads month+day and is 1-based on month input', () => {
    expect(isoFor(2026, 5, 2)).toBe('2026-06-02') // month is 0-based
    expect(isoFor(2026, 0, 9)).toBe('2026-01-09')
  })

  it('parseIso accepts well-formed ISO dates and rejects junk', () => {
    expect(parseIso('2026-06-02')).toEqual({ y: 2026, m: 5, d: 2 })
    expect(parseIso('2026-13-02')).toBeNull()
    expect(parseIso('garbage')).toBeNull()
    expect(parseIso('2026-06-2')).toBeNull()
  })

  it('daysInMonth handles leap February', () => {
    expect(daysInMonth(2024, 1)).toBe(29) // Feb 2024 leap
    expect(daysInMonth(2025, 1)).toBe(28)
    expect(daysInMonth(2026, 3)).toBe(30) // April
  })

  it('addDays crosses month + year boundaries', () => {
    expect(addDays('2026-06-30', 1)).toBe('2026-07-01')
    expect(addDays('2026-12-31', 1)).toBe('2027-01-01')
    expect(addDays('2026-01-01', -1)).toBe('2025-12-31')
  })

  it('clampIso bounds to [min,max]', () => {
    expect(clampIso('2026-06-02', '2026-06-05', '2026-06-30')).toBe('2026-06-05')
    expect(clampIso('2026-07-02', '2026-06-05', '2026-06-30')).toBe('2026-06-30')
    expect(clampIso('2026-06-10', '2026-06-05', '2026-06-30')).toBe('2026-06-10')
  })
})

describe('date-math — calendar grid primitives', () => {
  it('startOfWeek respects weekStartsOn (0=Sun, 1=Mon)', () => {
    // 2026-06-02 is a Tuesday.
    expect(startOfWeek('2026-06-02', 0)).toBe('2026-05-31') // Sunday
    expect(startOfWeek('2026-06-02', 1)).toBe('2026-06-01') // Monday
  })

  it('weekDays returns 7 consecutive ISO days from the week start', () => {
    const days = weekDays('2026-06-02', 1)
    expect(days).toHaveLength(7)
    expect(days[0]).toBe('2026-06-01')
    expect(days[6]).toBe('2026-06-07')
  })

  it('monthGrid returns a fixed 6x7 = 42 cells covering the month', () => {
    const cells = monthGrid(2026, 5, 0) // June 2026, Sunday start
    expect(cells).toHaveLength(42)
    // June 1 2026 is a Monday → Sunday-start grid begins on May 31.
    expect(cells[0]!.iso).toBe('2026-05-31')
    expect(cells[0]!.inMonth).toBe(false)
    const june1 = cells.find((c) => c.iso === '2026-06-01')!
    expect(june1.inMonth).toBe(true)
    expect(june1.day).toBe(1)
  })

  it('monthGrid marks weekends', () => {
    const cells = monthGrid(2026, 5, 0)
    const sun = cells.find((c) => c.iso === '2026-06-07')! // Sunday
    const wed = cells.find((c) => c.iso === '2026-06-03')! // Wednesday
    expect(sun.isWeekend).toBe(true)
    expect(wed.isWeekend).toBe(false)
  })
})

describe('date-math — datetime primitives (local wall-clock)', () => {
  it('parseIsoDateTime parses YYYY-MM-DDThh:mm', () => {
    expect(parseIsoDateTime('2026-06-02T09:30')).toEqual({
      y: 2026,
      m: 5,
      d: 2,
      hh: 9,
      mm: 30,
    })
    expect(parseIsoDateTime('2026-06-02')).toBeNull()
    expect(parseIsoDateTime('2026-06-02T25:00')).toBeNull()
  })

  it('dateTimeMinutes returns minutes-from-midnight for the wall-clock time', () => {
    expect(dateTimeMinutes('2026-06-02T00:00')).toBe(0)
    expect(dateTimeMinutes('2026-06-02T09:30')).toBe(570)
    expect(dateTimeMinutes('2026-06-02T23:59')).toBe(1439)
  })

  it('addMinutes advances wall-clock time and rolls the date', () => {
    expect(addMinutes('2026-06-02T09:30', 45)).toBe('2026-06-02T10:15')
    expect(addMinutes('2026-06-02T23:45', 30)).toBe('2026-06-03T00:15')
    expect(addMinutes('2026-06-02T00:15', -30)).toBe('2026-06-01T23:45')
  })

  it('diffMinutes computes signed wall-clock difference', () => {
    expect(diffMinutes('2026-06-02T09:00', '2026-06-02T10:30')).toBe(90)
    expect(diffMinutes('2026-06-02T10:30', '2026-06-02T09:00')).toBe(-90)
    expect(diffMinutes('2026-06-02T23:00', '2026-06-03T01:00')).toBe(120)
  })

  it('rangeOverlaps detects half-open interval overlap [start,end)', () => {
    expect(rangeOverlaps(0, 60, 30, 90)).toBe(true) // overlap
    expect(rangeOverlaps(0, 60, 60, 120)).toBe(false) // touching, not overlapping
    expect(rangeOverlaps(0, 60, 90, 120)).toBe(false) // disjoint
    expect(rangeOverlaps(30, 90, 0, 60)).toBe(true) // symmetric
  })

  // DECIDED v1 GAP — see date-math.ts header. DST fold/gap is out of scope for
  // v1 (local wall-clock, no offset/zone). Pinned, not silently assumed.
  it.skip('DST spring-forward gap (2am→3am) — v1 gap: wall-clock has no zone', () => {
    // A zone-aware build would skip the non-existent 02:30 local time.
    // v1 treats wall-clock as nominal minutes; DST handling is deferred.
    expect(true).toBe(true)
  })
})
