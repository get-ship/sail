// Backbone #3 — `createCanvas()`: the shared pan/zoom/pointer substrate.
//
// Pure reactive state on `@ship/sui-runtime` signals. No DOM emission, no
// SDK, no React, no fetch. The viewport element is a thunk (null until
// mount, mirroring `createVirtualizer`'s `getScrollElement`). Heavy
// coordinate/transform math lives in `transform.ts`; this file is the thin
// assembler — construct slices, wire the derived chain, own the pointer
// gesture pump, return the facade.

import { signal, derived, batch, untrack, type Derived } from '@ship/sui-runtime'
import { onCleanup } from '@ship/sui-runtime'
import {
  clampScale,
  composeZoomAt,
  fitTransform,
  screenToWorld as s2w,
  visibleWorldRect as vwr,
  worldToScreen as w2s,
} from './transform'
import {
  DEFAULT_MAX_SCALE,
  DEFAULT_MIN_SCALE,
  DEFAULT_ZOOM_SENSITIVITY,
  type Canvas,
  type CanvasOptions,
  type PointerState,
  type Transform,
  type Vec2,
} from './types'

export function createCanvas(options: CanvasOptions): Canvas {
  let opts = options
  const min = () => opts.minScale ?? DEFAULT_MIN_SCALE
  const max = () => opts.maxScale ?? DEFAULT_MAX_SCALE

  const seed = opts.initialTransform ?? {}
  const transform = signal<Transform>({
    x: seed.x ?? 0,
    y: seed.y ?? 0,
    scale: clampScale(seed.scale ?? 1, min(), max()),
  })
  const pointer = signal<PointerState>({ kind: 'idle' })
  const pointerScreen = signal<Vec2 | null>(null)
  const viewportSize = signal<{ w: number; h: number }>({ w: 0, h: 0 })

  // Derived presentational chain (one-direction; touching transform
  // recomputes only the suffix).
  const scale = derived(() => transform().scale)
  const pan = derived<Vec2>(() => {
    const t = transform()
    return { x: t.x, y: t.y }
  })
  const visibleWorldRect = derived(() => vwr(transform(), viewportSize()))
  const pointerWorld = derived<Vec2 | null>(() => {
    const p = pointerScreen()
    return p ? s2w(transform(), p) : null
  })
  const isPanning = derived(() => pointer().kind === 'panning')
  const isPinching = derived(() => pointer().kind === 'pinch')

  // --- Mutators (all writes inside batch; each fires onTransformChange). ---
  function commit(next: Transform): void {
    batch(() => {
      transform.set({ ...next, scale: clampScale(next.scale, min(), max()) })
    })
    opts.onTransformChange?.(untrack(transform))
  }
  function setTransform(t: Partial<Transform>): void {
    commit({ ...untrack(transform), ...t })
  }
  function panBy(dx: number, dy: number): void {
    const t = untrack(transform)
    commit({ ...t, x: t.x + dx, y: t.y + dy })
  }
  function panTo(p: Vec2): void {
    commit({ ...untrack(transform), x: p.x, y: p.y })
  }
  function zoomTo(next: number, center?: Vec2): void {
    const t = untrack(transform)
    const anchor = center ?? viewportCenter()
    commit(composeZoomAt(t, next, anchor, min(), max()))
  }
  function zoomBy(factor: number, center?: Vec2): void {
    zoomTo(untrack(transform).scale * factor, center)
  }
  function fitToRect(r: { x: number; y: number; w: number; h: number }, o?: { padding?: number; maxScale?: number }): void {
    commit(fitTransform(r, untrack(viewportSize), min(), o?.maxScale ?? max(), o?.padding ?? 24))
  }
  function resetView(): void {
    commit({ x: 0, y: 0, scale: 1 })
  }

  // --- Coordinate mapping (tracked when read in a derived/template). ---
  const screenToWorld = (p: Vec2): Vec2 => s2w(transform(), p)
  const worldToScreen = (p: Vec2): Vec2 => w2s(transform(), p)

  // --- Viewport geometry helpers. ---
  function viewportRect(): DOMRect | { left: number; top: number } {
    const el = opts.getViewportElement()
    return el ? el.getBoundingClientRect() : { left: 0, top: 0 }
  }
  function toLocal(clientX: number, clientY: number): Vec2 {
    const r = viewportRect()
    return { x: clientX - r.left, y: clientY - r.top }
  }
  function viewportCenter(): Vec2 {
    const s = untrack(viewportSize)
    return { x: s.w / 2, y: s.h / 2 }
  }

  // --- Pointer gesture pump. ---
  function onPointerDown(e: PointerEvent): void {
    const panButton = opts.gestures?.panButton ?? 1
    if (e.button === panButton) {
      pointer.set({ kind: 'panning', start: toLocal(e.clientX, e.clientY), origin: untrack(transform) })
    }
  }
  function onPointerMove(e: PointerEvent): void {
    const local = toLocal(e.clientX, e.clientY)
    pointerScreen.set(local)
    opts.onPointerWorld?.(s2w(untrack(transform), local), e)
    const st = untrack(pointer)
    if (st.kind === 'panning') {
      const inv = opts.panInverted ? -1 : 1
      panTo({
        x: st.origin.x + (local.x - st.start.x) * inv,
        y: st.origin.y + (local.y - st.start.y) * inv,
      })
    }
  }
  function onPointerUp(_e: PointerEvent): void {
    pointer.set({ kind: 'idle' })
  }
  function onWheel(e: WheelEvent): void {
    const wheelZoom = opts.gestures?.wheelZoom ?? true
    const wheelPan = opts.gestures?.wheelPan ?? true
    const sens = opts.zoomSensitivity ?? DEFAULT_ZOOM_SENSITIVITY
    if (e.ctrlKey && wheelZoom) {
      const factor = Math.exp(-e.deltaY * sens)
      zoomBy(factor, toLocal(e.clientX, e.clientY))
    } else if (wheelPan) {
      panBy(-e.deltaX, -e.deltaY)
    }
  }

  // --- ResizeObserver wiring (best-effort; null until mount). ---
  let ro: ResizeObserver | null = null
  let observed: HTMLElement | null = null
  function syncViewport(): void {
    const el = opts.getViewportElement()
    if (el === observed) return
    if (ro && observed) ro.unobserve(observed)
    observed = el
    if (!el) return
    const r = el.getBoundingClientRect()
    viewportSize.set({ w: r.width, h: r.height })
    if (typeof ResizeObserver !== 'undefined') {
      ro ??= new ResizeObserver((entries) => {
        for (const entry of entries) {
          const cr = entry.contentRect
          viewportSize.set({ w: cr.width, h: cr.height })
        }
      })
      ro.observe(el)
    }
  }
  syncViewport()

  function dispose(): void {
    if (ro) {
      ro.disconnect()
      ro = null
    }
    observed = null
  }
  onCleanup(dispose)

  return {
    transform: transform as Derived<Transform>,
    scale,
    pan,
    viewportSize: viewportSize as Derived<{ w: number; h: number }>,
    visibleWorldRect,
    isPanning,
    isPinching,
    pointerWorld,
    screenToWorld,
    worldToScreen,
    state: { transform, pointer, pointerScreen, viewportSize },
    setTransform,
    panBy,
    panTo,
    zoomBy,
    zoomTo,
    fitToRect,
    resetView,
    onPointerDown,
    onPointerMove,
    onPointerUp,
    onWheel,
    setOptions(patch) {
      opts = { ...opts, ...patch }
      syncViewport()
    },
    dispose,
  }
}
