// @vitest-environment happy-dom
// TESTS-FIRST — the stub-lie closeout (range selection + drag-to-create/move).
//
// The engine grew the `range` mutators + the create/move intent emitters; this
// suite wires them through the VIEW. Range selection sets the `range` signal and
// emits `sail-range-change`; drag-on-empty-time emits `sail-event-create-intent`;
// dragging a timed chip emits `sail-event-move`. Pointer geometry stays
// element-addressed (data-iso / data-minutes carried on the DOM) so the math is
// deterministic under happy-dom (no layout/rect dependence).
//
// Authority: build-spec §"GAP (drag-to-create/move, range selection)" —
// the slices the engine now exposes. Cardinal 3: mutations only in on:* / mount.

import { afterEach, describe, expect, it } from 'vitest'
import '../sail-calendar.compiled.js'
import { createContainerRegistry, mount, flush } from '../../test-utils'
import { createCalendar } from '../engine'
import type { Calendar, CalendarEvent } from '../engine'

const shadow = (el: HTMLElement): ShadowRoot | HTMLElement => el.shadowRoot ?? el

const reg = createContainerRegistry()
const live: Array<{ unmount(): void }> = []
afterEach(() => {
  for (const v of live.splice(0)) {
    try {
      v.unmount()
    } catch {
      /* best-effort */
    }
  }
})

function mountWith(events: CalendarEvent[], opts: Omit<Parameters<typeof createCalendar>[0], 'events' | 'initialDate'> = {}) {
  const calendar = createCalendar({ events, initialDate: '2026-06-15', ...opts })
  const m = mount(reg, 'sail-calendar', {})
  ;(m.el as unknown as { calendar: Calendar }).calendar = calendar
  const v = { el: m.el, flush, unmount: () => m.el.remove() }
  return { v, calendar }
}

/** Dispatch a pointer event of `type` at `el` (bubbles, composed). */
function pointer(el: Element, type: string): void {
  el.dispatchEvent(new (globalThis.PointerEvent ?? Event)(type, { bubbles: true, composed: true }))
}

const TIMED: CalendarEvent[] = [
  { id: 'a', title: 'Standup', allDay: false, start: '2026-06-15T09:00', end: '2026-06-15T10:00' },
]

describe('<sail-calendar> — month-grid range selection (range-mode=date)', () => {
  it('click+drag across day cells sets the engine range and emits sail-range-change (bubbles + composed)', async () => {
    const { v, calendar } = mountWith([], { rangeMode: 'date' })
    live.push(v)
    let evt: CustomEvent | null = null
    v.el.addEventListener('sail-range-change', (e) => { evt = e as CustomEvent })
    await v.flush()
    const from = shadow(v.el).querySelector('[role=gridcell][data-iso="2026-06-15"]')!
    const to = shadow(v.el).querySelector('[role=gridcell][data-iso="2026-06-18"]')!
    pointer(from, 'pointerdown')
    pointer(to, 'pointermove')
    pointer(to, 'pointerup')
    await v.flush()
    expect(calendar.range()).toEqual({ start: '2026-06-15', end: '2026-06-18' })
    expect(evt).not.toBeNull()
    expect(evt!.detail).toEqual({ start: '2026-06-15', end: '2026-06-18' })
    expect(evt!.bubbles).toBe(true)
    expect(evt!.composed).toBe(true)
  })

  it('does NOT begin a range when range-mode is none', async () => {
    const { v, calendar } = mountWith([], { rangeMode: 'none' })
    live.push(v)
    let fired = false
    v.el.addEventListener('sail-range-change', () => {
      fired = true
    })
    await v.flush()
    const from = shadow(v.el).querySelector('[role=gridcell][data-iso="2026-06-15"]')!
    const to = shadow(v.el).querySelector('[role=gridcell][data-iso="2026-06-18"]')!
    pointer(from, 'pointerdown')
    pointer(to, 'pointermove')
    pointer(to, 'pointerup')
    await v.flush()
    expect(calendar.range()).toBeNull()
    expect(fired).toBe(false)
  })

  it('Shift+ArrowRight extends the range from the focus anchor', async () => {
    const { v, calendar } = mountWith([], { rangeMode: 'date' })
    live.push(v)
    let detail: { start: string; end: string } | null = null
    v.el.addEventListener('sail-range-change', (e) => {
      detail = (e as CustomEvent).detail
    })
    await v.flush()
    const cell = shadow(v.el).querySelector('[role=gridcell][data-iso="2026-06-15"]')!
    cell.dispatchEvent(
      new KeyboardEvent('keydown', { key: 'ArrowRight', shiftKey: true, bubbles: true, composed: true }),
    )
    await v.flush()
    expect(calendar.range()).toEqual({ start: '2026-06-15', end: '2026-06-16' })
    cell.dispatchEvent(
      new KeyboardEvent('keydown', { key: 'ArrowRight', shiftKey: true, bubbles: true, composed: true }),
    )
    await v.flush()
    expect(calendar.range()).toEqual({ start: '2026-06-15', end: '2026-06-17' })
    expect(detail!).toEqual({ start: '2026-06-15', end: '2026-06-17' })
  })
})

describe('<sail-calendar> — drag-to-create (timed, day view)', () => {
  it('dragging across empty time slots emits sail-event-create-intent { start, end } (bubbles + composed)', async () => {
    const { v } = mountWith([], { initialView: 'day', slotMinutes: 60, dayStartHour: 8, dayEndHour: 18 })
    live.push(v)
    let evt: CustomEvent | null = null
    v.el.addEventListener('sail-event-create-intent', (e) => { evt = e as CustomEvent })
    await v.flush()
    // The day column carries a background slot grid: data-iso + data-minutes per slot.
    const slots = shadow(v.el).querySelectorAll('[data-iso][data-minutes]')
    expect(slots.length).toBeGreaterThan(0)
    const from = shadow(v.el).querySelector('[data-iso][data-minutes="540"]')! // 09:00
    const to = shadow(v.el).querySelector('[data-iso][data-minutes="660"]')! // 11:00
    pointer(from, 'pointerdown')
    pointer(to, 'pointermove')
    pointer(to, 'pointerup')
    await v.flush()
    expect(evt).not.toBeNull()
    // create-intent end is the END of the slot the drag released on (11:00 slot → 12:00).
    expect(evt!.detail).toEqual({ start: '2026-06-15T09:00', end: '2026-06-15T12:00' })
    expect(evt!.bubbles).toBe(true)
    expect(evt!.composed).toBe(true)
  })

  it('a plain click on one empty slot creates a single-slot intent', async () => {
    const { v } = mountWith([], { initialView: 'day', slotMinutes: 60, dayStartHour: 8, dayEndHour: 18 })
    live.push(v)
    let intent: { start: string; end: string } | null = null
    v.el.addEventListener('sail-event-create-intent', (e) => {
      intent = (e as CustomEvent).detail
    })
    await v.flush()
    const slot = shadow(v.el).querySelector('[data-iso][data-minutes="600"]')! // 10:00
    pointer(slot, 'pointerdown')
    pointer(slot, 'pointerup')
    await v.flush()
    expect(intent!).toEqual({ start: '2026-06-15T10:00', end: '2026-06-15T11:00' })
  })
})

describe('<sail-calendar> — drag-to-move (timed, day view)', () => {
  it('dragging an event chip onto a new slot emits sail-event-move with shifted bounds', async () => {
    const { v } = mountWith(TIMED, {
      initialView: 'day',
      slotMinutes: 60,
      dayStartHour: 8,
      dayEndHour: 18,
    })
    live.push(v)
    let evt: CustomEvent | null = null
    v.el.addEventListener('sail-event-move', (e) => { evt = e as CustomEvent })
    await v.flush()
    const chip = shadow(v.el).querySelector('[data-event-id="a"]')!
    const target = shadow(v.el).querySelector('[data-iso][data-minutes="660"]')! // 11:00
    pointer(chip, 'pointerdown')
    pointer(target, 'pointermove')
    pointer(target, 'pointerup')
    await v.flush()
    expect(evt).not.toBeNull()
    // event was 09:00–10:00 (60 min); dropping its start on 11:00 preserves duration.
    expect(evt!.detail).toEqual({ id: 'a', start: '2026-06-15T11:00', end: '2026-06-15T12:00' })
    expect(evt!.bubbles).toBe(true)
    expect(evt!.composed).toBe(true)
  })

  it('a click on a chip (no move) selects it — does NOT emit sail-event-move', async () => {
    const { v } = mountWith(TIMED, {
      initialView: 'day',
      slotMinutes: 60,
      dayStartHour: 8,
      dayEndHour: 18,
    })
    live.push(v)
    let movedCount = 0
    v.el.addEventListener('sail-event-move', () => {
      movedCount++
    })
    await v.flush()
    const chip = shadow(v.el).querySelector('[data-event-id="a"]')!
    pointer(chip, 'pointerdown')
    pointer(chip, 'pointerup') // released on the same chip — no slot move
    chip.dispatchEvent(new MouseEvent('click', { bubbles: true, composed: true }))
    await v.flush()
    expect(movedCount).toBe(0)
  })
})
