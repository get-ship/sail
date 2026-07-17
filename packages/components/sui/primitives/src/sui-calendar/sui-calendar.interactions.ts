// <sui-calendar> view — interaction wiring (the surface ↔ engine bridge).
//
// The controller's mount() installs ONE delegated pointer + keyboard listener
// set on the shadow root that translate DOM events into the engine's range /
// drag mutators and re-emit the advertised `sui-*` events. Kept out of the
// controller so each stays under the file cap (sui-sheet's interactions split).
//
// All mutations run inside the on:* handlers (Cardinal 3); the engine stays
// pure — it owns the `range` signal + emits create/move INTENTS but never grows
// its own events array. Pointer geometry is element-addressed: the month grid's
// gridcells carry data-iso, and each day column renders a background slot grid
// whose cells carry data-iso + data-minutes, so the pointer→date/time map needs
// no layout/rect math (works headless + under test).

import { bindNativeEvent } from '@ship/sail-runtime'
import { addDays, addMinutes, diffMinutes } from './engine/date-math'
import type { Calendar, CalendarEvent, ISODateTime } from './engine'

/** The context the interaction wiring needs from the controller scope. */
export interface CalendarInteractionContext {
  /** The live engine, read fresh per event (a prop:calendar swap is honored).
   *  The engine's own `rangeMode` gates range drag/keyboard; `none` disables. */
  calOf(): Calendar
  /** The slot granularity in minutes (drag-to-create slot span). */
  slotMinutes(): number
}

/** The nearest ancestor of an event's target carrying `data-iso` (a day cell or
 *  slot), or null. Uses `.closest()` off `e.target` — the surface lives in one
 *  shadow root so the target is never retargeted across a boundary. */
function isoFromEvent(e: Event): string | null {
  const el = (e.target as HTMLElement | null)?.closest?.('[data-iso]') as HTMLElement | null
  return el?.dataset.iso ?? null
}

/** The `{ iso, minutes }` of the background time-slot under a pointer event. */
function slotFromEvent(e: Event): { iso: string; minutes: number } | null {
  const el = (e.target as HTMLElement | null)?.closest?.('[data-iso][data-minutes]') as HTMLElement | null
  if (!el || el.dataset.minutes == null || !el.dataset.iso) return null
  return { iso: el.dataset.iso, minutes: Number(el.dataset.minutes) }
}

/** Format minutes-from-midnight on `iso` as a wall-clock ISODateTime. */
function timeAt(iso: string, minutes: number): ISODateTime {
  const hh = Math.floor(minutes / 60)
  const mm = minutes % 60
  const p2 = (n: number) => (n < 10 ? `0${n}` : String(n))
  return `${iso}T${p2(hh)}:${p2(mm)}`
}

/** Map a keydown to a whole-day delta for Shift+Arrow range extension. */
function shiftArrowDelta(e: KeyboardEvent): number | null {
  if (!e.shiftKey) return null
  switch (e.key) {
    case 'ArrowRight':
      return 1
    case 'ArrowLeft':
      return -1
    case 'ArrowDown':
      return 7
    case 'ArrowUp':
      return -7
    default:
      return null
  }
}

/** Install all surface handlers on the host's shadow root. One delegated
 *  pointerdown routes month-grid range drag vs. time-grid create/move; keydown
 *  handles Shift+Arrow range extension. Cleanups auto-enroll via
 *  `bindNativeEvent` against the active effect-scope. */
export function wireCalendarInteractions(host: HTMLElement, ctx: CalendarInteractionContext): void {
  const sr = host.shadowRoot ?? host
  if (!sr) return
  const { calOf, slotMinutes } = ctx

  // Transient document-less drag: listen for moves/up on the shadow root and
  // self-remove on pointerup (mirrors sui-sheet's dragWithin).
  const dragWithin = (onMove: (e: Event) => void, onUp: (e: Event) => void): void => {
    const move = (ev: Event) => onMove(ev)
    const up = (ev: Event) => {
      sr.removeEventListener('pointermove', move)
      sr.removeEventListener('pointerup', up)
      onUp(ev)
    }
    sr.addEventListener('pointermove', move)
    sr.addEventListener('pointerup', up)
  }

  // ── Month-grid range drag (gridcell → gridcell) ──────────────────────────────
  const beginMonthRange = (startIso: string): void => {
    const cal = calOf()
    cal.beginRange(startIso)
    let last = startIso
    dragWithin(
      (ev) => {
        const iso = isoFromEvent(ev)
        if (iso) {
          last = iso
          cal.extendRange(iso)
        }
      },
      () => {
        cal.extendRange(last)
        const r = cal.range()
        if (r) host.emit!('sui-range-change', r)
      },
    )
  }

  // ── Time-grid drag-to-create (empty slot) / drag-to-move (timed chip) ─────────
  const beginCreate = (slot: { iso: string; minutes: number }): void => {
    let endSlot = slot
    dragWithin(
      (ev) => {
        const s = slotFromEvent(ev)
        if (s) endSlot = s
      },
      () => {
        const span = slotMinutes()
        // create-intent runs [drag-start slot start, drag-end slot END) so a
        // single-slot click still yields a non-zero-length span.
        const a = timeAt(slot.iso, slot.minutes)
        const b = timeAt(endSlot.iso, endSlot.minutes + span)
        const bounds = normalizeBounds(a, b)
        calOf().createIntent(bounds.start as ISODateTime, bounds.end as ISODateTime)
        host.emit!('sui-event-create-intent', bounds)
      },
    )
  }

  const beginMove = (id: string): void => {
    const ev0 = findEvent(calOf().state.events(), id)
    if (!ev0 || ev0.allDay) return
    const duration = Math.max(slotMinutes(), diffMinutes(ev0.start, ev0.end))
    let dropSlot: { iso: string; minutes: number } | null = null
    dragWithin(
      (ev) => {
        const s = slotFromEvent(ev)
        if (s) dropSlot = s
      },
      () => {
        if (!dropSlot) return // released without crossing a slot — a plain click
        const start = timeAt(dropSlot.iso, dropSlot.minutes)
        const end = addMinutes(start, duration)
        calOf().moveEvent(id, start, end)
        host.emit!('sui-event-move', { id, start, end })
      },
    )
  }

  const onPointerDown = (e: Event): void => {
    const target = e.target as HTMLElement | null
    if (!target) return

    // A timed chip → move drag.
    const chip = target.closest?.('[data-event-id]') as HTMLElement | null
    if (chip) {
      beginMove(chip.dataset.eventId!)
      return
    }
    // An empty time slot → create drag.
    const slot = slotFromEvent(e)
    if (slot && target.closest?.('[data-iso][data-minutes]')) {
      beginCreate(slot)
      return
    }
    // A month-grid day cell → range drag (gated by the engine's range-mode).
    if (calOf().rangeMode === 'none') return
    const cell = target.closest?.('[role=gridcell][data-iso]') as HTMLElement | null
    if (cell) beginMonthRange(cell.dataset.iso!)
  }

  const onKeyDown = (e: KeyboardEvent): void => {
    const cal = calOf()
    if (cal.rangeMode === 'none') return
    const cell = (e.target as HTMLElement | null)?.closest?.(
      '[role=gridcell][data-iso]',
    ) as HTMLElement | null
    if (!cell) return
    const delta = shiftArrowDelta(e)
    if (delta == null) return
    // Shift+Arrow extends the range as the focus anchor moves. Anchor the range
    // on the current focus (first extend), move focus by the delta, then extend
    // to the new focus — so the range tracks focus regardless of which DOM cell
    // the keydown fired on. The controller's plain-arrow nav skips shiftKey, so
    // focus is moved here exactly once.
    if (cal.range() == null) cal.beginRange(cal.focusDate())
    const next = addDays(cal.focusDate(), delta)
    cal.goToDate(next)
    cal.extendRange(next)
    const r = cal.range()
    if (r) host.emit!('sui-range-change', r)
    e.preventDefault()
  }

  bindNativeEvent(sr, 'pointerdown', onPointerDown)
  bindNativeEvent(sr, 'keydown', onKeyDown as (e: Event) => void)
}

/** Chronological [start,end] normalization (lexical order = chronological for
 *  both ISODate + ISODateTime). */
function normalizeBounds(a: string, b: string): { start: string; end: string } {
  return a <= b ? { start: a, end: b } : { start: b, end: a }
}

function findEvent(events: CalendarEvent[], id: string): CalendarEvent | undefined {
  return events.find((e) => e.id === id)
}
