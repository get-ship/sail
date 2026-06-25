// Backbone #3 substrate — engine behavioral specs. Pure-runtime: no DOM
// mount needed beyond a fake viewport element with a bounding rect.
import { describe, expect, it, vi } from 'vitest'
import { createCanvas } from '../src/engine/create-canvas'
import {
  clampScale,
  composeZoomAt,
  screenToWorld,
  worldToScreen,
  visibleWorldRect,
} from '../src/engine/transform'
import type { Transform } from '../src/engine/types'

/** A fake viewport element with a fixed 800×600 bounding rect at origin. */
function fakeViewport(w = 800, h = 600): HTMLElement {
  return {
    getBoundingClientRect: () => ({ left: 0, top: 0, width: w, height: h, right: w, bottom: h }),
  } as unknown as HTMLElement
}

const ID: Transform = { x: 0, y: 0, scale: 1 }

describe('transform math (pure)', () => {
  it('clampScale bounds to [min,max]', () => {
    expect(clampScale(100, 0.1, 8)).toBe(8)
    expect(clampScale(0.001, 0.1, 8)).toBe(0.1)
    expect(clampScale(2, 0.1, 8)).toBe(2)
  })

  it('screenToWorld inverts worldToScreen', () => {
    const t: Transform = { x: 120, y: -40, scale: 2 }
    const screen = { x: 300, y: 220 }
    const world = screenToWorld(t, screen)
    const back = worldToScreen(t, world)
    expect(back.x).toBeCloseTo(screen.x, 6)
    expect(back.y).toBeCloseTo(screen.y, 6)
  })

  it('worldToScreen applies pan then scale', () => {
    const t: Transform = { x: 10, y: 20, scale: 2 }
    expect(worldToScreen(t, { x: 5, y: 5 })).toEqual({ x: 20, y: 30 })
  })

  it('composeZoomAt keeps the anchor screen-point fixed', () => {
    const t = ID
    const anchor = { x: 400, y: 300 }
    const next = composeZoomAt(t, 2, anchor, 0.1, 8)
    expect(next.scale).toBe(2)
    // the world point under the anchor must map back to the same screen point
    const worldUnder = screenToWorld(t, anchor)
    const screenAfter = worldToScreen(next, worldUnder)
    expect(screenAfter.x).toBeCloseTo(anchor.x, 6)
    expect(screenAfter.y).toBeCloseTo(anchor.y, 6)
  })

  it('visibleWorldRect covers the viewport in world coords', () => {
    const t: Transform = { x: 0, y: 0, scale: 2 }
    const r = visibleWorldRect(t, { w: 800, h: 600 })
    expect(r).toEqual({ x: 0, y: 0, w: 400, h: 300 })
  })
})

describe('createCanvas — viewport transform', () => {
  it('seeds from initialTransform', () => {
    const c = createCanvas({
      getViewportElement: () => fakeViewport(),
      initialTransform: { x: 5, y: 6, scale: 3 },
    })
    expect(c.transform()).toEqual({ x: 5, y: 6, scale: 3 })
    expect(c.scale()).toBe(3)
    expect(c.pan()).toEqual({ x: 5, y: 6 })
  })

  it('panBy accumulates and panTo replaces', () => {
    const c = createCanvas({ getViewportElement: () => fakeViewport() })
    c.panBy(10, 20)
    c.panBy(5, -5)
    expect(c.pan()).toEqual({ x: 15, y: 15 })
    c.panTo({ x: 0, y: 0 })
    expect(c.pan()).toEqual({ x: 0, y: 0 })
  })

  it('zoomTo clamps to min/max', () => {
    const c = createCanvas({
      getViewportElement: () => fakeViewport(),
      minScale: 0.5,
      maxScale: 4,
    })
    c.zoomTo(100)
    expect(c.scale()).toBe(4)
    c.zoomTo(0.01)
    expect(c.scale()).toBe(0.5)
  })

  it('zoomBy about a screen center keeps that point fixed', () => {
    const c = createCanvas({ getViewportElement: () => fakeViewport() })
    const center = { x: 400, y: 300 }
    const worldUnder = c.screenToWorld(center)
    c.zoomBy(2, center)
    expect(c.scale()).toBe(2)
    const after = c.worldToScreen(worldUnder)
    expect(after.x).toBeCloseTo(center.x, 6)
    expect(after.y).toBeCloseTo(center.y, 6)
  })

  it('setTransform merges a partial', () => {
    const c = createCanvas({ getViewportElement: () => fakeViewport() })
    c.setTransform({ scale: 2 })
    expect(c.transform()).toEqual({ x: 0, y: 0, scale: 2 })
  })

  it('resetView returns to identity', () => {
    const c = createCanvas({
      getViewportElement: () => fakeViewport(),
      initialTransform: { x: 9, y: 9, scale: 4 },
    })
    c.resetView()
    expect(c.transform()).toEqual({ x: 0, y: 0, scale: 1 })
  })

  it('fires onTransformChange on mutation', () => {
    const onTransformChange = vi.fn()
    const c = createCanvas({ getViewportElement: () => fakeViewport(), onTransformChange })
    c.panBy(1, 1)
    expect(onTransformChange).toHaveBeenCalledWith({ x: 1, y: 1, scale: 1 })
  })

  it('fitToRect frames a world rect centered with clamp', () => {
    const c = createCanvas({ getViewportElement: () => fakeViewport(800, 600), maxScale: 8 })
    c.fitToRect({ x: 0, y: 0, w: 400, h: 300 }, { padding: 0 })
    // viewport 800×600, rect 400×300 → scale 2 (limited by both axes equally)
    expect(c.scale()).toBe(2)
    // rect center (200,150) → viewport center (400,300)
    const center = c.worldToScreen({ x: 200, y: 150 })
    expect(center.x).toBeCloseTo(400, 6)
    expect(center.y).toBeCloseTo(300, 6)
  })
})

describe('createCanvas — pointer model', () => {
  function down(x: number, y: number, button = 1): PointerEvent {
    return { clientX: x, clientY: y, button, pointerId: 1 } as unknown as PointerEvent
  }
  function move(x: number, y: number): PointerEvent {
    return { clientX: x, clientY: y, pointerId: 1 } as unknown as PointerEvent
  }
  function up(): PointerEvent {
    return { pointerId: 1 } as unknown as PointerEvent
  }

  it('middle-button drag pans the viewport', () => {
    const c = createCanvas({ getViewportElement: () => fakeViewport() })
    c.onPointerDown(down(100, 100))
    expect(c.isPanning()).toBe(true)
    c.onPointerMove(move(130, 120))
    expect(c.pan()).toEqual({ x: 30, y: 20 })
    c.onPointerUp(up())
    expect(c.isPanning()).toBe(false)
  })

  it('a non-pan button does not start panning', () => {
    const c = createCanvas({ getViewportElement: () => fakeViewport() })
    c.onPointerDown(down(100, 100, 0))
    expect(c.isPanning()).toBe(false)
  })

  it('tracks pointerWorld in world coordinates', () => {
    const c = createCanvas({
      getViewportElement: () => fakeViewport(),
      initialTransform: { scale: 2 },
    })
    c.onPointerMove(move(200, 100))
    expect(c.pointerWorld()).toEqual({ x: 100, y: 50 })
  })

  it('onPointerWorld callback fires on move', () => {
    const onPointerWorld = vi.fn()
    const c = createCanvas({ getViewportElement: () => fakeViewport(), onPointerWorld })
    c.onPointerMove(move(50, 60))
    expect(onPointerWorld).toHaveBeenCalledWith({ x: 50, y: 60 }, expect.anything())
  })

  it('wheel with ctrlKey zooms anchored at the cursor', () => {
    const c = createCanvas({ getViewportElement: () => fakeViewport() })
    const center = { x: 400, y: 300 }
    const worldUnder = c.screenToWorld(center)
    c.onWheel({
      clientX: 400,
      clientY: 300,
      deltaY: -100,
      ctrlKey: true,
      preventDefault: () => {},
    } as unknown as WheelEvent)
    expect(c.scale()).toBeGreaterThan(1)
    const after = c.worldToScreen(worldUnder)
    expect(after.x).toBeCloseTo(center.x, 6)
    expect(after.y).toBeCloseTo(center.y, 6)
  })

  it('plain wheel pans (wheelPan default)', () => {
    const c = createCanvas({ getViewportElement: () => fakeViewport() })
    c.onWheel({
      clientX: 0,
      clientY: 0,
      deltaX: 10,
      deltaY: 20,
      ctrlKey: false,
      preventDefault: () => {},
    } as unknown as WheelEvent)
    expect(c.pan()).toEqual({ x: -10, y: -20 })
  })

  it('dispose is idempotent and safe before mount', () => {
    const c = createCanvas({ getViewportElement: () => null })
    expect(() => {
      c.dispose()
      c.dispose()
    }).not.toThrow()
  })
})
