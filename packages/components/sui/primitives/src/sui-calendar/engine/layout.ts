// sui-calendar engine — overlap → lane layout (Slice 1a). PURE geometry.
//
// Interval-graph greedy coloring: sort timed events by start (tie-break on end,
// then id for determinism), assign each event the lowest-indexed lane whose
// previous occupant has ended (half-open: end == next.start does NOT overlap, so
// the lane is reusable). Events are grouped into overlap CLUSTERS — a maximal
// run where the running max-end keeps extending past the next start — and every
// event in a cluster shares that cluster's lane count so their boxes evenly
// split the column width. Emits {laneIndex, laneCount, topPct, heightPct}
// where top/height are percentages of the visible [windowStart,windowEnd)
// minute window, clamped to the window. No DOM, no signals.

import { dateTimeMinutes } from './date-math'
import type { PositionedEvent, TimedEvent } from './types'

interface Interval {
  event: TimedEvent
  start: number
  end: number
}

/**
 * Lay out a single day's timed events.
 * @param events  timed events whose start/end are wall-clock ISODateTime
 * @param windowStart minutes-from-midnight of the visible window start
 * @param windowEnd   minutes-from-midnight of the visible window end (exclusive)
 */
export function layoutDay(
  events: TimedEvent[],
  windowStart: number,
  windowEnd: number,
): PositionedEvent[] {
  const span = Math.max(1, windowEnd - windowStart)

  const intervals: Interval[] = events
    .map((event) => ({
      event,
      start: dateTimeMinutes(event.start),
      end: dateTimeMinutes(event.end),
    }))
    .sort((a, b) => a.start - b.start || a.end - b.end || cmp(a.event.id, b.event.id))

  const placed = new Map<string, { laneIndex: number }>()
  const laneCountFor = new Map<string, number>()

  // Walk clusters: a cluster ends when the next event starts at/after the
  // running max-end of all events seen so far in the cluster.
  let i = 0
  while (i < intervals.length) {
    let clusterEnd = intervals[i]!.end
    let j = i + 1
    while (j < intervals.length && intervals[j]!.start < clusterEnd) {
      clusterEnd = Math.max(clusterEnd, intervals[j]!.end)
      j++
    }
    const cluster = intervals.slice(i, j)

    // Greedy lane assignment within the cluster.
    const laneEnds: number[] = [] // laneEnds[k] = end-minute of last event in lane k
    for (const iv of cluster) {
      let lane = laneEnds.findIndex((e) => e <= iv.start)
      if (lane === -1) {
        lane = laneEnds.length
        laneEnds.push(iv.end)
      } else {
        laneEnds[lane] = iv.end
      }
      placed.set(iv.event.id, { laneIndex: lane })
    }
    const count = laneEnds.length
    for (const iv of cluster) laneCountFor.set(iv.event.id, count)

    i = j
  }

  return intervals.map((iv) => {
    const top = Math.max(windowStart, Math.min(iv.start, windowEnd))
    const bottom = Math.max(windowStart, Math.min(iv.end, windowEnd))
    return {
      event: iv.event,
      laneIndex: placed.get(iv.event.id)!.laneIndex,
      laneCount: laneCountFor.get(iv.event.id)!,
      topPct: ((top - windowStart) / span) * 100,
      heightPct: (Math.max(0, bottom - top) / span) * 100,
    }
  })
}

const cmp = (a: string, b: string): number => (a < b ? -1 : a > b ? 1 : 0)
