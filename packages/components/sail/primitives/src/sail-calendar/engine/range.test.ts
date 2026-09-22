// TESTS-FIRST — range selection + drag intents (the stub-lie closeout).
//
// The `range` signal existed but no mutator ever set it, and drag-to-create /
// drag-to-move were unbuilt. These assert the engine grows the range mutators
// (begin/extend/select/clear) gated by `rangeMode` (none|date|datetime) and the
// pure create/move intent emitters. The engine stays pure — it owns the `range`
// state + the callbacks; it NEVER mutates the events array (the consumer does).

import { describe, expect, it, vi } from 'vitest'
import { createCalendar } from './create-calendar'
import type { DateRange } from './types'

describe('createCalendar — range selection (date mode)', () => {
  it('selectRange sets the range signal (start<=end normalized) and emits', () => {
    const onRangeChange = vi.fn()
    const cal = createCalendar({
      events: [],
      initialDate: '2026-06-15',
      rangeMode: 'date',
      onRangeChange,
    })
    expect(cal.range()).toBeNull()
    cal.selectRange('2026-06-20', '2026-06-17')
    // normalized: start is the earlier date.
    expect(cal.range()).toEqual({ start: '2026-06-17', end: '2026-06-20' })
    expect(onRangeChange).toHaveBeenCalledWith({ start: '2026-06-17', end: '2026-06-20' })
  })

  it('begin/extend builds a range from the anchor (drag + Shift+Arrow path)', () => {
    const onRangeChange = vi.fn()
    const cal = createCalendar({
      events: [],
      initialDate: '2026-06-15',
      rangeMode: 'date',
      onRangeChange,
    })
    cal.beginRange('2026-06-10')
    expect(cal.range()).toEqual({ start: '2026-06-10', end: '2026-06-10' })
    cal.extendRange('2026-06-14')
    expect(cal.range()).toEqual({ start: '2026-06-10', end: '2026-06-14' })
    // extend backward past the anchor flips the bounds, anchor stays fixed.
    cal.extendRange('2026-06-07')
    expect(cal.range()).toEqual({ start: '2026-06-07', end: '2026-06-10' })
    const last = onRangeChange.mock.calls.at(-1)![0] as DateRange
    expect(last).toEqual({ start: '2026-06-07', end: '2026-06-10' })
  })

  it('clearRange resets the range to null and emits null', () => {
    const onRangeChange = vi.fn()
    const cal = createCalendar({
      events: [],
      initialDate: '2026-06-15',
      rangeMode: 'date',
      onRangeChange,
    })
    cal.selectRange('2026-06-17', '2026-06-20')
    onRangeChange.mockClear()
    cal.clearRange()
    expect(cal.range()).toBeNull()
    expect(onRangeChange).toHaveBeenCalledWith(null)
  })
})

describe('createCalendar — range selection (mode gating)', () => {
  it('rangeMode "none" makes range mutators no-ops (the signal never sets)', () => {
    const onRangeChange = vi.fn()
    const cal = createCalendar({
      events: [],
      initialDate: '2026-06-15',
      rangeMode: 'none',
      onRangeChange,
    })
    cal.selectRange('2026-06-17', '2026-06-20')
    cal.beginRange('2026-06-10')
    cal.extendRange('2026-06-14')
    expect(cal.range()).toBeNull()
    expect(onRangeChange).not.toHaveBeenCalled()
  })

  it('default rangeMode is "none"', () => {
    const cal = createCalendar({ events: [], initialDate: '2026-06-15' })
    cal.selectRange('2026-06-17', '2026-06-20')
    expect(cal.range()).toBeNull()
  })

  it('datetime mode normalizes ISODateTime bounds chronologically', () => {
    const cal = createCalendar({
      events: [],
      initialDate: '2026-06-15',
      rangeMode: 'datetime',
    })
    cal.selectRange('2026-06-15T14:00', '2026-06-15T09:00')
    expect(cal.range()).toEqual({ start: '2026-06-15T09:00', end: '2026-06-15T14:00' })
  })
})

describe('createCalendar — drag-to-create intent', () => {
  it('createIntent emits sail-event-create-intent bounds without mutating events', () => {
    const onEventCreateIntent = vi.fn()
    const cal = createCalendar({
      events: [],
      initialDate: '2026-06-15',
      initialView: 'day',
      onEventCreateIntent,
    })
    cal.createIntent('2026-06-15T09:00', '2026-06-15T10:30')
    expect(onEventCreateIntent).toHaveBeenCalledWith({
      start: '2026-06-15T09:00',
      end: '2026-06-15T10:30',
    })
    // PURE: the engine never grows its own events array from an intent.
    expect(cal.isEmpty()).toBe(true)
  })

  it('createIntent normalizes a backward drag (end before start)', () => {
    const onEventCreateIntent = vi.fn()
    const cal = createCalendar({
      events: [],
      initialDate: '2026-06-15',
      initialView: 'day',
      onEventCreateIntent,
    })
    cal.createIntent('2026-06-15T11:00', '2026-06-15T09:00')
    expect(onEventCreateIntent).toHaveBeenCalledWith({
      start: '2026-06-15T09:00',
      end: '2026-06-15T11:00',
    })
  })
})

describe('createCalendar — drag-to-move intent', () => {
  it('moveEvent emits sail-event-move new bounds without mutating events', () => {
    const onEventMoveIntent = vi.fn()
    const cal = createCalendar({
      events: [
        { id: 'a', title: 'Standup', start: '2026-06-15T09:00', end: '2026-06-15T10:00' },
      ],
      initialDate: '2026-06-15',
      initialView: 'day',
      onEventMoveIntent,
    })
    cal.moveEvent('a', '2026-06-15T11:00', '2026-06-15T12:00')
    expect(onEventMoveIntent).toHaveBeenCalledWith({
      id: 'a',
      start: '2026-06-15T11:00',
      end: '2026-06-15T12:00',
    })
    // the original event bounds are untouched in the engine's view-model.
    const col = cal.dayColumns()[0]!
    const ev = col.laidOut.find((p) => p.event.id === 'a')!.event
    expect(ev.start).toBe('2026-06-15T09:00')
  })
})
