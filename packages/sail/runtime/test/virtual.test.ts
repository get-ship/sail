// TESTS-FIRST — Backbone #2, Slice 7b: createVirtualizer() reactive adapter.
//
// Status: RED. `../src/virtual` does not exist yet (confirmed greenfield in the
// build-spec: "packages/sail/runtime/src/virtual.ts … does not exist on disk yet").
// The import is import-not-found until Slice 7a (vendor virtual-core) + 7b (the
// reactive adapter) land. DO NOT add the impl here. DO NOT stub virtual-core.
//
// Authority: build-spec §"Backbone #2 — createVirtualizer()", §"API", §"Signal-
// graph mechanics" (tick-based push + derived pull; effect bridges reactive
// options; element-thunk binding; onCleanup teardown), §"Identity short-circuit —
// corrected" (compare key + start + size; pure-scroll preserves identity,
// MEASURE frames BREAK it), §"Read/write discipline" (virtualItems/totalSize/
// range read tracked; measureElement/scrollToIndex imperative-only),
// §"horizontal / lanes scope for v1" (present in the surface, out-of-scope for the
// sail-table v1 consumer — no dead-surface assertion here).
//
// This file co-locates with the existing runtime suite under test/ so the runtime
// package's `include: ['test/**/*.test.ts']` glob already picks it up — no glob
// edit needed for the runtime package.

import { describe, expect, it, vi } from 'vitest'
import { signal, derived } from '../src/signal'
// RED: greenfield module — resolves only once Slice 7b ships.
import { createVirtualizer } from '../src/virtual'

/** A scroll element big enough to window a small slice out of a large count. */
function makeViewport(height = 200): HTMLDivElement {
  const el = document.createElement('div')
  Object.defineProperty(el, 'clientHeight', { value: height, configurable: true })
  Object.defineProperty(el, 'offsetHeight', { value: height, configurable: true })
  document.body.appendChild(el)
  return el
}

describe('createVirtualizer — windowing', () => {
  it('renders only a window (overscan + viewport), not the full count', () => {
    const viewport = makeViewport(200)
    const v = createVirtualizer({
      count: () => 100_000,
      estimateSize: () => 20,
      getScrollElement: () => viewport,
      overscan: 5,
    })
    const items = v.virtualItems()
    // ~ (200 / 20) + overscan ≈ a couple dozen, FAR fewer than 100k.
    expect(items.length).toBeGreaterThan(0)
    expect(items.length).toBeLessThan(100)
  })

  it('totalSize reflects count * estimate so the scrollbar is honest', () => {
    const viewport = makeViewport(200)
    const v = createVirtualizer({
      count: () => 1000,
      estimateSize: () => 20,
      getScrollElement: () => viewport,
    })
    expect(v.totalSize()).toBe(1000 * 20)
  })

  it('range exposes startIndex/endIndex for "showing X–Y of N"', () => {
    const viewport = makeViewport(200)
    const v = createVirtualizer({
      count: () => 1000,
      estimateSize: () => 20,
      getScrollElement: () => viewport,
    })
    const r = v.range()
    // At scroll 0 the window starts at the top, spans a real slice (viewport
    // 200 / est 20 = ~10 rows + overscan), and stays FAR short of the 1000-row
    // count — that windowing is the whole point of "showing X–Y of N".
    expect(r.startIndex).toBe(0)
    expect(r.endIndex).toBeGreaterThan(r.startIndex)
    expect(r.endIndex).toBeLessThan(100)
  })
})

describe('createVirtualizer — reactive count (binds to centerRows().length)', () => {
  it('recomputes totalSize when the reactive count() changes', () => {
    const viewport = makeViewport(200)
    const count = signal(100)
    const v = createVirtualizer({
      count: () => count(),
      estimateSize: () => 20,
      getScrollElement: () => viewport,
    })
    expect(v.totalSize()).toBe(100 * 20)
    count.set(500)
    expect(v.totalSize()).toBe(500 * 20)
  })
})

describe('createVirtualizer — identity short-circuit (the corrected gotcha)', () => {
  it('preserves virtualItems() identity across a pure no-op frame', () => {
    const viewport = makeViewport(200)
    const v = createVirtualizer({
      count: () => 1000,
      estimateSize: () => 20,
      getScrollElement: () => viewport,
    })
    const a = v.virtualItems()
    const b = v.virtualItems()
    // No scroll, no measure → same array identity so {#for} short-circuits.
    expect(b).toBe(a)
  })

  it('BREAKS virtualItems() identity after a measureElement size change', () => {
    const viewport = makeViewport(200)
    const v = createVirtualizer({
      count: () => 1000,
      estimateSize: () => 20, // estimate
      getScrollElement: () => viewport,
    })
    const before = v.virtualItems()
    const firstEl = document.createElement('div')
    // Real measured height differs from the estimate → offsets/sizes change.
    Object.defineProperty(firstEl, 'offsetHeight', { value: 80, configurable: true })
    Object.defineProperty(firstEl, 'clientHeight', { value: 80, configurable: true })
    firstEl.setAttribute('data-index', '0')
    v.measureElement(firstEl)
    const after = v.virtualItems()
    // A MEASUREMENT change must NOT be short-circuited (key+start+size compared).
    expect(after).not.toBe(before)
  })
})

describe('createVirtualizer — read/write discipline (Cardinals 2 & 3)', () => {
  it('virtualItems/totalSize/range are tracked reads inside a derived', () => {
    const viewport = makeViewport(200)
    const count = signal(50)
    const v = createVirtualizer({
      count: () => count(),
      estimateSize: () => 20,
      getScrollElement: () => viewport,
    })
    const total = derived(() => v.totalSize())
    expect(total()).toBe(50 * 20)
    count.set(60)
    expect(total()).toBe(60 * 20)
  })

  it('scrollToIndex / measureElement / measure are imperative methods', () => {
    const viewport = makeViewport(200)
    const v = createVirtualizer({
      count: () => 1000,
      estimateSize: () => 20,
      getScrollElement: () => viewport,
    })
    expect(typeof v.scrollToIndex).toBe('function')
    expect(typeof v.measureElement).toBe('function')
    expect(typeof v.measure).toBe('function')
    // Imperative calls must not throw with a mounted viewport.
    expect(() => v.scrollToIndex(500)).not.toThrow()
  })
})

describe('createVirtualizer — element-thunk binding + teardown', () => {
  it('tolerates a null scroll element until mount (thunk returns null)', () => {
    let el: HTMLDivElement | null = null
    const v = createVirtualizer({
      count: () => 1000,
      estimateSize: () => 20,
      getScrollElement: () => el, // null until mount
    })
    // No element yet → no crash; once the element appears the effect wires it.
    expect(() => v.virtualItems()).not.toThrow()
    el = makeViewport(200)
    expect(() => v.measure()).not.toThrow()
  })

  it('dispose tears down observers/effects (idempotent)', () => {
    const viewport = makeViewport(200)
    const v = createVirtualizer({
      count: () => 1000,
      estimateSize: () => 20,
      getScrollElement: () => viewport,
    })
    expect(() => v.dispose()).not.toThrow()
    expect(() => v.dispose()).not.toThrow() // idempotent
  })

  it('surfaces range changes via onChange (the onReachEnd wiring seam)', () => {
    const viewport = makeViewport(200)
    const onChange = vi.fn()
    const count = signal(1000)
    createVirtualizer({
      count: () => count(),
      estimateSize: () => 20,
      getScrollElement: () => viewport,
      onChange,
    })
    // A count change re-windows and fires onChange so onReachEnd can hook range().
    count.set(2000)
    expect(onChange).toHaveBeenCalled()
  })
})
