// TESTS-FIRST — Slice 1a: overlap → lane layout (interval graph coloring).
//
// The fiddly piece. Pure geometry, no DOM, no signals. Greedy lane packing:
// sort by start, assign each event the first lane whose last event has ended;
// laneCount within an overlap cluster is shared so boxes split the column.

import { describe, expect, it } from 'vitest'
import { layoutDay } from './layout'
import type { TimedEvent } from './types'

const ev = (id: string, start: string, end: string): TimedEvent => ({
  id,
  title: id,
  start: `2026-06-02T${start}`,
  end: `2026-06-02T${end}`,
})

describe('layoutDay — overlap lane packing', () => {
  it('three mutually-overlapping events land in three lanes (laneCount=3)', () => {
    const out = layoutDay(
      [ev('a', '09:00', '11:00'), ev('b', '09:30', '10:30'), ev('c', '10:00', '12:00')],
      0,
      1440,
    )
    expect(out).toHaveLength(3)
    const lanes = out.map((p) => p.laneIndex).sort()
    expect(lanes).toEqual([0, 1, 2])
    for (const p of out) expect(p.laneCount).toBe(3)
  })

  it('a non-overlapping later event reuses lane 0 (laneCount=1 in its cluster)', () => {
    const out = layoutDay([ev('a', '09:00', '10:00'), ev('b', '11:00', '12:00')], 0, 1440)
    const a = out.find((p) => p.event.id === 'a')!
    const b = out.find((p) => p.event.id === 'b')!
    expect(a!.laneIndex).toBe(0)
    expect(b.laneIndex).toBe(0)
    expect(a!.laneCount).toBe(1)
    expect(b.laneCount).toBe(1)
  })

  it('touching-but-not-overlapping (end==next.start) reuses a lane', () => {
    const out = layoutDay([ev('a', '09:00', '10:00'), ev('b', '10:00', '11:00')], 0, 1440)
    expect(out.find((p) => p.event.id === 'a')!.laneIndex).toBe(0)
    expect(out.find((p) => p.event.id === 'b')!.laneIndex).toBe(0)
  })

  it('two overlapping then a free one: cluster of 2 lanes, then a 1-lane cluster', () => {
    const out = layoutDay(
      [ev('a', '09:00', '11:00'), ev('b', '10:00', '12:00'), ev('c', '13:00', '14:00')],
      0,
      1440,
    )
    const a = out.find((p) => p.event.id === 'a')!
    const b = out.find((p) => p.event.id === 'b')!
    const c = out.find((p) => p.event.id === 'c')!
    expect(a!.laneCount).toBe(2)
    expect(b.laneCount).toBe(2)
    expect(a!.laneIndex).not.toBe(b.laneIndex)
    expect(c.laneCount).toBe(1)
    expect(c.laneIndex).toBe(0)
  })

  it('geometry: topPct/heightPct reflect the visible window bounds', () => {
    // window 09:00..17:00 = [540,1020); event 10:00..11:00.
    const out = layoutDay([ev('a', '10:00', '11:00')], 540, 1020)
    const a = out[0]
    // top = (600-540)/(1020-540) = 60/480 = 12.5%
    expect(a!.topPct).toBeCloseTo(12.5, 5)
    // height = 60/480 = 12.5%
    expect(a!.heightPct).toBeCloseTo(12.5, 5)
  })

  it('clamps events that extend beyond the window', () => {
    // window 09:00..17:00; event 16:00..20:00 clamps end to 17:00.
    const out = layoutDay([ev('a', '16:00', '20:00')], 540, 1020)
    const a = out[0]
    expect(a!.topPct).toBeCloseTo(((960 - 540) / 480) * 100, 5)
    expect(a!.heightPct).toBeCloseTo(((1020 - 960) / 480) * 100, 5)
  })

  it('is deterministic regardless of input order', () => {
    const a = ev('a', '09:00', '11:00')
    const b = ev('b', '09:30', '10:30')
    const out1 = layoutDay([a, b], 0, 1440)
    const out2 = layoutDay([b, a], 0, 1440)
    const byId = (o: typeof out1) =>
      Object.fromEntries(o.map((p) => [p.event.id, p.laneIndex]))
    expect(byId(out1)).toEqual(byId(out2))
  })
})
