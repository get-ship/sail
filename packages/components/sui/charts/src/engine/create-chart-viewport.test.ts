// Backbone #2 — `createChartViewport()` specs (TESTS-FIRST, RED).
//
// The responsive-geometry adapter: a shared `ResizeObserver` wrapper turning the
// host's measured pixel box into reactive `width()`/`height()` signals so charts
// stop hardcoding the viewBox. One observer implementation, seven consumers.
// Pure `@ship/sui-runtime` (signal/derived) + the DOM `ResizeObserver` platform
// API created in `observe()` and torn down in `dispose()`.
//
// Build-spec: Backbone #2 "API" + "Mechanics" + Slice 4.

import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { createChartViewport } from './create-chart-viewport'

// A controllable ResizeObserver: capture the callback so a test can drive a
// resize, and count live observers to assert teardown.
interface FakeRO {
  cb: ResizeObserverCallback
  el: Element | null
}
let observers: FakeRO[] = []
let disconnects = 0
const realRO = globalThis.ResizeObserver

beforeEach(() => {
  observers = []
  disconnects = 0
  class TestRO {
    cb: ResizeObserverCallback
    el: Element | null = null
    constructor(cb: ResizeObserverCallback) {
      this.cb = cb
      observers.push(this as unknown as FakeRO)
    }
    observe(el: Element) {
      this.el = el
    }
    unobserve() {}
    disconnect() {
      disconnects++
    }
  }
  // @ts-expect-error — test double
  globalThis.ResizeObserver = TestRO
})

afterEach(() => {
  globalThis.ResizeObserver = realRO
})

/** Push a measured content box to the last-created observer's callback. */
function resizeTo(w: number, h: number): void {
  const ro = observers[observers.length - 1]!
  const entry = {
    contentRect: { width: w, height: h } as DOMRectReadOnly,
    target: ro.el,
  } as unknown as ResizeObserverEntry
  ro.cb([entry], ro as unknown as ResizeObserver)
}

describe('createChartViewport — seed + reactive box', () => {
  it('seeds width/height from the estimate options', () => {
    const vp = createChartViewport({ initialWidth: 300, initialHeight: 200 })
    expect(vp.width()).toBe(300)
    expect(vp.height()).toBe(200)
  })

  it('exposes width/height as readable signals (functions)', () => {
    const vp = createChartViewport({ initialWidth: 300, initialHeight: 200 })
    expect(typeof vp.width).toBe('function')
    expect(typeof vp.height).toBe('function')
  })
})

describe('createChartViewport — observe drives the signals', () => {
  it('observe(el) creates a ResizeObserver', () => {
    const vp = createChartViewport({ initialWidth: 300, initialHeight: 200 })
    vp.observe(document.createElement('div'))
    expect(observers.length).toBe(1)
  })

  it('a measured box updates the reactive width/height', () => {
    const vp = createChartViewport({ initialWidth: 300, initialHeight: 200 })
    vp.observe(document.createElement('div'))
    resizeTo(640, 480)
    expect(vp.width()).toBe(640)
    expect(vp.height()).toBe(480)
  })
})

describe('createChartViewport — teardown', () => {
  it('dispose() disconnects the observer', () => {
    const vp = createChartViewport({ initialWidth: 300, initialHeight: 200 })
    vp.observe(document.createElement('div'))
    vp.dispose()
    expect(disconnects).toBe(1)
  })
})
