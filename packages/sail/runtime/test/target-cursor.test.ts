// `use:targetCursor` — the action's contract.
//
// What this test pins down (and why each line earns its place):
//
// 1. The action writes ONLY custom properties (`--tc-x`, `--tc-y`).
//    A bare `.style.translate =` would slip past the compiler's runtime
//    guard (the lint only fires on pack `.ts`), but the mission says
//    "mirror how other stdlib actions (e.g. `spotlight`) do it". The
//    assertion below is the only check that enforces it.
//
// 2. The host gets `data-tc-on` toggled so the companion stylesheet
//    can suppress the native cursor. The action never assigns a CSS
//    property — `[data-tc-on='true'] { cursor: none }` lives in the
//    injected sheet. This is the floor-legal shape.
//
// 3. The initial state has no stray dot at the viewport centre. The
//    previous implementation initialised `mx = innerWidth / 2` and
//    `my = innerHeight / 2`, which the rAF tick would happily draw
//    a glyph at — even with `data-on='false'` and opacity:0, the dot
//    was a visible artefact the moment a pointer hovered the host.
//    The fix is `mx = my = -9999` until the first `pointermove`.
//
// 4. On `pointerleave`, the action parks the cursor and clears
//    `data-tc-on` so the host's native cursor is restored.
//
// 5. On `destroy`, the overlay is removed from the body and
//    `data-tc-on` is cleared from the host.
//
// rAF is driven by vi's fake timers + `vi.advanceTimersToNextFrame()`
// — no real wall-clock waits, no race against the machine. Every
// assertion fires after a deterministic `await pumpFrame()`, which
// flushes the next queued rAF callback via the fake-timer clock.

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { targetCursor } from '../src/actions/target-cursor'

// jsdom is the default environment (vitest.config.ts `environment: 'jsdom'`).
// The action consults matchMedia at attach time: (hover: hover) AND
// (pointer: fine) gate the action. JSDOM's matchMedia returns
// `matches: false` by default — without the override the action
// short-circuits to a no-op. We force the desktop-pointer environment.
function setDesktopPointer(matches = true): void {
  Object.defineProperty(window, 'matchMedia', {
    writable: true,
    configurable: true,
    value: (query: string) => ({
      matches,
      media: query,
      onchange: null,
      addListener: () => {},
      removeListener: () => {},
      addEventListener: () => {},
      removeEventListener: () => {},
      dispatchEvent: () => false,
    }),
  })
}

function firePointer(target: Element, type: string, init: { clientX: number; clientY: number }): void {
  // JSDOM has no PointerEvent (only MouseEvent). The action only reads
  // clientX/clientY off the event, so MouseEvent is sufficient for the
  // listener coverage.
  target.dispatchEvent(
    new MouseEvent(type, {
      bubbles: true,
      cancelable: true,
      clientX: init.clientX,
      clientY: init.clientY,
    }),
  )
}

interface RectLike {
  left: number
  right: number
  top: number
  bottom: number
  width: number
  height: number
}
const FULL_RECT: RectLike = { left: 0, right: 1000, top: 0, bottom: 800, width: 1000, height: 800 }

function stubHostRect(host: HTMLElement, rect: RectLike = FULL_RECT): void {
  vi.spyOn(host, 'getBoundingClientRect').mockReturnValue(rect as DOMRect)
}

// Pump the rAF queue exactly once. vi.useFakeTimers() makes rAF a
// fake-timer callback; `vi.advanceTimersToNextFrame()` flushes one
// frame. The action's tick and the test's resolve are both queued; the
// tick runs first (it was registered at mount), then the resolve fires
// and the awaiter unblocks. Each frame is exactly one tick's worth of
// work — no race, no time-sliced recompute.
function pumpFrame(): Promise<void> {
  const { promise, resolve } = Promise.withResolvers<void>()
  requestAnimationFrame(() => resolve())
  vi.advanceTimersToNextFrame()
  return promise
}

afterEach(() => {
  vi.useRealTimers()
  vi.restoreAllMocks()
  document.body.innerHTML = ''
  // The injected <style> persists across tests (it's a module-level
  // singleton). Strip it so each test starts from a clean sheet.
  document.querySelectorAll('style[id^="sail-target-cursor-"]').forEach((s) => s.remove())
})

describe('targetCursor — the floor-legal write shape', () => {
  beforeEach(() => {
    setDesktopPointer(true)
    vi.useFakeTimers()
  })

  it('writes ONLY --tc-x and --tc-y custom properties on the overlay nodes', async () => {
    const host = document.createElement('div')
    document.body.appendChild(host)
    stubHostRect(host)
    const result = targetCursor(host)

    firePointer(host, 'pointermove', { clientX: 100, clientY: 120 })
    await pumpFrame()

    const layer = document.body.querySelector('.sail-tc-layer') as HTMLElement
    expect(layer).toBeTruthy()
    const brackets = layer.querySelectorAll('.sail-tc-bracket')
    const dot = layer.querySelector('.sail-tc-dot') as HTMLElement
    expect(brackets.length).toBe(4)
    expect(dot).toBeTruthy()

    // The custom-property write is the only sanctioned form. The lint
    // refuses `bracket.style.translate = …` (a CSS property assignment);
    // `--tc-x` and `--tc-y` are read by the framework's
    // `translate: var(--tc-x) var(--tc-y)` rule. No real CSS property
    // is set on any overlay node.
    for (const c of Array.from(brackets) as HTMLElement[]) {
      expect(c.style.translate).toBe('')
      expect(c.style.left).toBe('')
      expect(c.style.top).toBe('')
      expect(c.style.getPropertyValue('--tc-x')).toMatch(/-?\d+(\.\d+)?px$/)
      expect(c.style.getPropertyValue('--tc-y')).toMatch(/-?\d+(\.\d+)?px$/)
    }
    expect(dot.style.translate).toBe('')
    expect(dot.style.getPropertyValue('--tc-x')).toMatch(/-?\d+(\.\d+)?px$/)
    expect(dot.style.getPropertyValue('--tc-y')).toMatch(/-?\d+(\.\d+)?px$/)

    result.destroy!()
  })

  it('uses a data-attribute to suppress the native cursor (no host.style.cursor write)', async () => {
    const host = document.createElement('div')
    document.body.appendChild(host)
    stubHostRect(host)
    const result = targetCursor(host)

    // The companion stylesheet carries `[data-tc-on='true'] { cursor: none }`.
    // The action toggles the attribute; it never assigns host.style.cursor.
    // The attribute is initialised to 'false' synchronously so a
    // pre-first-pointermove query is deterministic.
    expect(host.dataset.tcOn).toBe('false')

    firePointer(host, 'pointermove', { clientX: 50, clientY: 60 })
    await pumpFrame()
    expect(host.dataset.tcOn).toBe('true')
    // The floor: no CSS property assigned on the host element.
    expect(host.style.cursor).toBe('')
    expect(host.style.cssText).toBe('')

    result.destroy!()
  })
})

describe('targetCursor — initial state has no stray dot', () => {
  beforeEach(() => {
    setDesktopPointer(true)
    vi.useFakeTimers()
  })

  it('starts with the overlay hidden and mx/my parked off-screen', async () => {
    const host = document.createElement('div')
    document.body.appendChild(host)
    stubHostRect(host)
    const result = targetCursor(host)

    // The layer is mounted to <body> but its visibility is gated on
    // both `data-on='true'` AND a seen pointer. Before the first
    // pointermove, the dot must NOT be drawn at the viewport centre.
    const layer = document.body.querySelector('.sail-tc-layer') as HTMLElement
    expect(layer).toBeTruthy()
    expect(layer.dataset.on).toBe('false')
    expect(host.dataset.tcOn).toBe('false')

    // Let the rAF tick fire once with the parked values.
    await pumpFrame()
    const dot = layer.querySelector('.sail-tc-dot') as HTMLElement
    // The custom property is set (the tick always writes it), but it
    // sits at -10001px / -10001px (the parked mx/my of -9999 minus the
    // 2px dot-centre offset) — not at the viewport centre.
    expect(dot.style.getPropertyValue('--tc-x')).toBe('-10001px')
    expect(dot.style.getPropertyValue('--tc-y')).toBe('-10001px')

    result.destroy!()
  })
})

describe('targetCursor — pointer tracking', () => {
  beforeEach(() => {
    setDesktopPointer(true)
    vi.useFakeTimers()
  })

  it('follows the pointer while inside the host', async () => {
    const host = document.createElement('div')
    document.body.appendChild(host)
    stubHostRect(host)
    const result = targetCursor(host)

    firePointer(host, 'pointermove', { clientX: 200, clientY: 300 })
    await pumpFrame()
    const layer = document.body.querySelector('.sail-tc-layer') as HTMLElement
    const dot = layer.querySelector('.sail-tc-dot') as HTMLElement
    expect(layer.dataset.on).toBe('true')
    expect(host.dataset.tcOn).toBe('true')
    // Dot centre sits at (mx-2, my-2).
    expect(dot.style.getPropertyValue('--tc-x')).toBe('198px')
    expect(dot.style.getPropertyValue('--tc-y')).toBe('298px')
    // The TL corner has eased partway from the parked (-9999) start
    // toward the (200-14, 300-14) idle crosshair goal. We assert it
    // MOVED — not the exact eased value, which depends on the ease
    // constant.
    const tlCorner = layer.querySelector('.sail-tc-bracket[data-side="tl"]') as HTMLElement
    expect(tlCorner.style.getPropertyValue('--tc-x')).not.toBe('-9999px')

    result.destroy!()
  })

  it('parks the cursor and clears data-tc-on on pointerleave', async () => {
    const host = document.createElement('div')
    document.body.appendChild(host)
    stubHostRect(host)
    const result = targetCursor(host)

    firePointer(host, 'pointermove', { clientX: 100, clientY: 100 })
    await pumpFrame()
    expect(host.dataset.tcOn).toBe('true')

    firePointer(host, 'pointerleave', { clientX: 0, clientY: 0 })
    await pumpFrame()
    const layer = document.body.querySelector('.sail-tc-layer') as HTMLElement
    const dot = layer.querySelector('.sail-tc-dot') as HTMLElement
    expect(host.dataset.tcOn).toBe('false')
    expect(layer.dataset.on).toBe('false')
    // The dot translates by (mx-2, my-2); with mx/my parked at -9999
    // the dot sits at -10001px / -10001px — well off-screen.
    expect(dot.style.getPropertyValue('--tc-x')).toBe('-10001px')
    expect(dot.style.getPropertyValue('--tc-y')).toBe('-10001px')

    result.destroy!()
  })
})

describe('targetCursor — cleanup', () => {
  beforeEach(() => {
    setDesktopPointer(true)
    vi.useFakeTimers()
  })

  it('removes the overlay and clears the host data-attribute on destroy', async () => {
    const host = document.createElement('div')
    document.body.appendChild(host)
    stubHostRect(host)
    const result = targetCursor(host)
    expect(document.body.querySelector('.sail-tc-layer')).toBeTruthy()

    firePointer(host, 'pointermove', { clientX: 10, clientY: 20 })
    await pumpFrame()
    expect(host.dataset.tcOn).toBe('true')

    result.destroy!()
    expect(document.body.querySelector('.sail-tc-layer')).toBeNull()
    expect(host.dataset.tcOn).toBeUndefined()
  })

  it('a second mount after destroy does not leave a stale overlay behind', () => {
    const host = document.createElement('div')
    document.body.appendChild(host)
    stubHostRect(host)
    const d1 = targetCursor(host)
    d1.destroy!()
    expect(document.body.querySelectorAll('.sail-tc-layer').length).toBe(0)
    const d2 = targetCursor(host)
    expect(document.body.querySelectorAll('.sail-tc-layer').length).toBe(1)
    d2.destroy!()
  })
})

describe('targetCursor — touch / coarse pointer is a no-op', () => {
  it('returns a no-op destroyer and adds no overlay on a touch device', () => {
    setDesktopPointer(false)
    const host = document.createElement('div')
    document.body.appendChild(host)
    stubHostRect(host)
    const result = targetCursor(host)
    expect(document.body.querySelector('.sail-tc-layer')).toBeNull()
    expect(typeof result.destroy).toBe('function')
    expect(() => result.destroy!()).not.toThrow()
  })
})
