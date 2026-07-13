// @vitest-environment happy-dom
// TESTS-FIRST — Slice 2: the <sui-calendar> VIEW (month/week/day + controller).
//
// Status: RED until `../sui-calendar.sui` + `../sui-calendar.controller.ts`
// exist. The engine (`./engine/`) is COMMITTED + GREEN — this slice renders its
// view-model (monthCells / dayColumns / timeSlots / agenda) and wires
// navigation / view-switch / event-select, emitting `sui-*` via `host.emit!`.
//
// Harness: `renderComponent` compiles `sui-calendar.sui` under the tag and
// registers the custom element. The live engine reaches the view through the
// `prop:calendar` IDENTITY path — assigned to the host `calendar` property
// (a live JS reference, never a stringified attribute), mirroring the
// sui-viewer `prop:src` pattern. The `events` `type="string"` attr is the
// SSR/declarative fallback (JSON.parse in the controller).
//
// Authority: build-spec §"The view", §"Accessibility", behavioral specs
// #1 (month grid from live engine, 6×7, no windowed-count ARIA),
// #2 (view switching + title), #3 (navigation + aria-current),
// #8 (event selection emits sui-event-select), #11 (empty precedence).

import { readFileSync } from 'node:fs'
import { afterEach, describe, expect, it } from 'vitest'
import '../sui-calendar.compiled.js'
import { createContainerRegistry, mount, flush } from '../../test-utils'
import { createCalendar } from '../engine'
import type { Calendar, CalendarEvent } from '../engine'


const shadow = (el: HTMLElement): ShadowRoot | HTMLElement => el.shadowRoot ?? el

/** Dispatch a bubbling click at the first shadow-DOM element matching `sel`. */
function clickShadow(el: HTMLElement, sel: string): void {
  const target = shadow(el).querySelector(sel)
  if (!target) throw new Error(`clickShadow: no element matches "${sel}"`)
  target.dispatchEvent(new MouseEvent('click', { bubbles: true, composed: true }))
}

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

/** Mount the calendar with a live engine fed through the prop:calendar identity. */
function mountWith(events: CalendarEvent[], opts: Omit<Parameters<typeof createCalendar>[0], 'events' | 'initialDate'> = {}) {
  const calendar = createCalendar({ events, initialDate: '2026-06-15', ...opts })
  const m = mount(reg, 'sui-calendar', {})
  ;(m.el as unknown as { calendar: Calendar }).calendar = calendar
  const v = { el: m.el, flush, unmount: () => m.el.remove() }
  return { v, calendar }
}

const EV: CalendarEvent[] = [
  { id: 'a', title: 'Standup', allDay: false, start: '2026-06-15T09:00', end: '2026-06-15T09:30' },
  { id: 'b', title: 'Holiday', allDay: true, start: '2026-06-18' },
]

describe('<sui-calendar> — registration + engine identity', () => {

  it('renders a month grid from the live engine (prop:calendar identity)', async () => {
    const { v } = mountWith(EV)
    live.push(v)
    await v.flush()
    const grid = shadow(v.el).querySelector('[role=grid]')
    expect(grid).not.toBeNull()
    // 6×7 fixed month grid — 42 gridcells.
    expect(shadow(v.el).querySelectorAll('[role=gridcell]').length).toBe(42)
  })
})

describe('<sui-calendar> — month grid (#1)', () => {
  it('reports 6 rows / 7 columns regardless of DOM, NO windowed-count ARIA', async () => {
    const { v } = mountWith(EV)
    live.push(v)
    await v.flush()
    const grid = shadow(v.el).querySelector('[role=grid]')!
    expect(shadow(v.el).querySelectorAll('[role=row]').length).toBe(6)
    expect(shadow(v.el).querySelectorAll('[role=columnheader]').length).toBe(7)
    // The fixed month grid is NEVER windowed → no aria-rowcount.
    expect(grid.hasAttribute('aria-rowcount')).toBe(false)
  })

  it('places event chips into the day cell they touch (native button descendants)', async () => {
    const { v } = mountWith(EV)
    live.push(v)
    await v.flush()
    const cell = shadow(v.el).querySelector('[data-iso="2026-06-15"]')!
    const chip = cell.querySelector('button[data-event-id="a"]')
    expect(chip).not.toBeNull()
    // A chip is a native <button> descendant (implicit role=button), NOT itself a gridcell.
    expect((chip as HTMLElement).tagName).toBe('BUTTON')
  })

  it("today's cell carries aria-current=date (#3)", async () => {
    // The engine seeds itself on today; the grid it renders must mark that
    // date. (mountWith pins an initialDate, so build the engine unpinned.)
    const calendar = createCalendar({ events: [] })
    const today = calendar.focusDate()
    const m = mount(reg, 'sui-calendar', {})
    ;(m.el as unknown as { calendar: Calendar }).calendar = calendar
    const v = { el: m.el, flush, unmount: () => m.el.remove() }
    live.push(v)
    await flush()
    const cur = shadow(v.el).querySelector('[aria-current="date"]')
    expect(cur).not.toBeNull()
    expect(cur!.getAttribute('data-iso')).toBe(today)
  })
})

describe('<sui-calendar> — view switching + navigation (#2/#3)', () => {
  it('reflects the view prop and updates the title via the engine', async () => {
    const { v, calendar } = mountWith(EV)
    live.push(v)
    await v.flush()
    expect(shadow(v.el).querySelector('h2')!.textContent).toContain('June 2026')
    calendar.setView('day')
    await v.flush()
    expect(v.el.getAttribute('view')).toBe('day')
    expect(shadow(v.el).querySelector('h2')!.textContent).toContain('Jun')
  })

  it('emits sui-view-change {view} (bubbles + composed) when the toolbar switches view', async () => {
    const { v } = mountWith(EV)
    live.push(v)
    let evt: CustomEvent | null = null
    v.el.addEventListener('sui-view-change', (e) => { evt = e as CustomEvent })
    await v.flush()
    clickShadow(v.el, '[data-view="week"]')
    await v.flush()
    expect(evt).not.toBeNull()
    expect(evt!.detail).toEqual({ view: 'week' })
    expect(evt!.bubbles).toBe(true)
    expect(evt!.composed).toBe(true)
  })

  it('emits sui-date-change {date} (bubbles + composed) on next/prev navigation', async () => {
    const { v } = mountWith(EV)
    live.push(v)
    let evt: CustomEvent | null = null
    v.el.addEventListener('sui-date-change', (e) => { evt = e as CustomEvent })
    await v.flush()
    clickShadow(v.el, '[data-nav="next"]')
    await v.flush()
    expect(evt).not.toBeNull()
    expect(evt!.detail).toEqual({ date: '2026-07-15' })
    expect(evt!.bubbles).toBe(true)
    expect(evt!.composed).toBe(true)
  })
})

describe('<sui-calendar> — week/day grid', () => {
  it('renders the time-slot axis from timeSlots() in day view', async () => {
    const { v } = mountWith(EV, { initialView: 'day', slotMinutes: 60 })
    live.push(v)
    await v.flush()
    // 24 hourly slots over a 0..24 window.
    expect(shadow(v.el).querySelectorAll('[data-slot]').length).toBe(24)
  })

  it('renders timed events as positioned lanes in the day column', async () => {
    const { v } = mountWith(EV, { initialView: 'day' })
    live.push(v)
    await v.flush()
    const chip = shadow(v.el).querySelector('[data-event-id="a"]') as HTMLElement
    expect(chip).not.toBeNull()
    // lane geometry from PositionedEvent → inline top/height.
    expect(chip.style.top).not.toBe('')
    expect(chip.style.height).not.toBe('')
  })
})

describe('<sui-calendar> — event selection (#8)', () => {
  it('clicking a chip selects it and emits sui-event-select + sui-event-click', async () => {
    const { v } = mountWith(EV)
    live.push(v)
    let selectEvt: CustomEvent | null = null
    let clickEvt: CustomEvent | null = null
    v.el.addEventListener('sui-event-select', (e) => { selectEvt = e as CustomEvent })
    v.el.addEventListener('sui-event-click', (e) => { clickEvt = e as CustomEvent })
    await v.flush()
    clickShadow(v.el, '[data-event-id="a"]')
    await v.flush()
    expect(selectEvt).not.toBeNull()
    const detail = selectEvt!.detail as { id: string; selectedIds: string[] }
    expect(detail.id).toBe('a')
    expect(detail.selectedIds).toContain('a')
    expect(selectEvt!.bubbles).toBe(true)
    expect(selectEvt!.composed).toBe(true)
    // The raw chip-click event fires alongside selection with just the id.
    expect(clickEvt).not.toBeNull()
    expect(clickEvt!.detail).toEqual({ id: 'a' })
    expect(clickEvt!.bubbles).toBe(true)
    expect(clickEvt!.composed).toBe(true)
    // aria-selected reflects engine selection.
    expect(shadow(v.el).querySelector('[data-event-id="a"]')!.getAttribute('aria-selected')).toBe(
      'true',
    )
  })
})

describe('<sui-calendar> — empty state precedence (#11)', () => {
  it('renders the empty part when the visible window has no events', async () => {
    const { v } = mountWith([])
    live.push(v)
    await v.flush()
    expect(shadow(v.el).querySelector('[role="status"]')).not.toBeNull()
  })

  it('does NOT render the empty part when events are present', async () => {
    const { v } = mountWith(EV)
    live.push(v)
    await v.flush()
    expect(shadow(v.el).querySelector('[role="status"]')).toBeNull()
  })
})

describe('<sui-calendar> — month grid keyboard operability (a11y #259)', () => {
  const press = (el: HTMLElement, key: string): void => {
    const cell = shadow(el).querySelector('[role=gridcell][data-iso="2026-06-15"]') as HTMLElement
    cell.dispatchEvent(new KeyboardEvent('keydown', { key, bubbles: true, composed: true }))
  }

  it('gives the focus anchor a roving tabindex (focused cell 0, others -1)', async () => {
    const { v } = mountWith(EV) // initialDate 2026-06-15
    live.push(v)
    await v.flush()
    const focused = shadow(v.el).querySelector('[data-iso="2026-06-15"]')!
    expect(focused.getAttribute('tabindex')).toBe('0')
    const other = shadow(v.el).querySelector('[data-iso="2026-06-16"]')!
    expect(other.getAttribute('tabindex')).toBe('-1')
  })

  it('ArrowRight moves the focus anchor forward one day', async () => {
    const { v, calendar } = mountWith(EV)
    live.push(v)
    await v.flush()
    press(v.el, 'ArrowRight')
    await v.flush()
    expect(calendar.focusDate()).toBe('2026-06-16')
    expect(shadow(v.el).querySelector('[data-iso="2026-06-16"]')!.getAttribute('tabindex')).toBe('0')
  })

  it('ArrowDown moves the focus anchor forward one week', async () => {
    const { v, calendar } = mountWith(EV)
    live.push(v)
    await v.flush()
    press(v.el, 'ArrowDown')
    await v.flush()
    expect(calendar.focusDate()).toBe('2026-06-22')
  })

  it('emits sui-date-change as arrow navigation moves the anchor', async () => {
    const { v } = mountWith(EV)
    live.push(v)
    let date: string | null = null
    v.el.addEventListener('sui-date-change', (e) => {
      date = (e as CustomEvent).detail.date
    })
    await v.flush()
    press(v.el, 'ArrowLeft')
    await v.flush()
    expect(date).toBe('2026-06-14')
  })
})

describe('<sui-calendar> — SSR fallback + density/token discipline', () => {
  it('parses the events type="string" SSR fallback when no engine identity is set', async () => {
    const json = JSON.stringify(EV)
    const m = mount(reg, 'sui-calendar', { events: json, 'focus-date': '2026-06-15' })
    const v = { el: m.el, flush, unmount: () => m.el.remove() }
    live.push(v)
    await flush()
    const cell = shadow(v.el).querySelector('[data-iso="2026-06-15"]')!
    expect(cell.querySelector('[data-event-id="a"]')).not.toBeNull()
  })

  it('exposes NO density/dense prop on the host', async () => {
    const { v } = mountWith(EV)
    live.push(v)
    await v.flush()
    const proto = Object.getPrototypeOf(v.el) as object
    expect(Object.getOwnPropertyNames(proto)).not.toContain('density')
    expect(Object.getOwnPropertyNames(proto)).not.toContain('dense')
  })

  it('authors no CSS at all — the floor (REVIEW gate, asserted)', () => {
    const src = readFileSync(new URL('../sui-calendar.sui', import.meta.url), 'utf8')
    expect(src).not.toMatch(/<style[\s>]/)
    expect(src).not.toMatch(/\sstyle="/)
  })
})
