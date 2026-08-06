<!--
  <sail-canvas> — the bare pan/zoom/infinite-surface substrate primitive.

  Backbone #3: a viewport + a transformed world layer + a `default` slot for
  arbitrary world-space content. Pan/zoom/coordinate state lives in
  `createCanvas()` (engine/) on @ship/sail-runtime signals — no React, no
  Excalidraw, no SDK. This pack imports @ship/sail-runtime + @ship/sail-dom ONLY.

  Props
  ─────────────────────────────────────────────────────────────────────
  value       (string, '')          SSR/declarative snapshot FALLBACK only
                                     (parsed via JSON.parse in the controller).
                                     * identity supersedes.
  readonly    (bool,  false reflect) Pan/zoom allowed but no edits.
  theme       ('light'|'dark')       Read imperatively off the host.
  grid        ('none'|'dots'|'lines')Background grid render hint.
  min-scale   (number)               Zoom clamp.
  max-scale   (number)               Zoom clamp.
  label       (string, '')           → aria-label on the application region.
  ─────────────────────────────────────────────────────────────────────
  Event: sail-canvas-viewport { transform } — debounced, on pan/zoom change.

  Geometry contract: the world layer carries the engine's transform (pan +
  zoom). `screenToWorld` is the inverse of that paint, and it assumes the
  layer scales about its TOP-LEFT corner and that the viewport swallows touch
  gestures rather than letting the page scroll them.
-->

<scope>
  props: {
    value = ''
    readonly = false
    theme: 'light' | 'dark' = 'light'
    grid: 'none' | 'dots' | 'lines' = 'none'
    'min-scale' = 0
    'max-scale' = 0
    label = ''
  }
</scope>
<controller>
import type { Action } from '@ship/sail-runtime'
import { debounce } from '@ship/sail-runtime'
import type { Scope } from './sail-canvas.sail'
import { createCanvas } from '../engine/create-canvas'
import type { Canvas, Transform } from '../engine/types'

// SSR fallback = the verified sail-combobox shape: plain JSON.parse in the
// controller, try/caught. The live path is * identity onto the engine.
function parseSnapshot(raw: string): { transform?: Partial<Transform> } | null {
  if (!raw) return null
  try {
    return JSON.parse(raw) as { transform?: Partial<Transform> }
  } catch {
    return null
  }
}

// <sail-canvas> controller — the bare substrate primitive. Builds a
// createCanvas() in setup, forwards raw pointer/wheel events to the engine
// as returned handlers, and emits a debounced sail-canvas-viewport event via
// host.emit! from mount.
export default controller(({ value }: Scope) => {
  const viewportEl = signal<HTMLDivElement | null>(null)
  // Fallback engine — used when no engine is injected via canvas.
  // exactOptionalPropertyTypes: only set initialTransform when present —
  // passing `undefined` to the optional field is itself a type error.
  const seedTransform = parseSnapshot(value())?.transform
  const fallback = createCanvas({
    getViewportElement: () => viewportEl(),
    ...(seedTransform ? { initialTransform: seedTransform } : {}),
  })
  // An engine injected by a composing face (sail-board / sail-draw) via
  // canvas identity. When present it supersedes the fallback so the
  // board/draw transform and sail-canvas pan/zoom share ONE engine.
  const injected = signal<Canvas | null>(null)
  const canvas = derived<Canvas>(() => injected() ?? fallback)
  // Drives the debounced outbound viewport event off the ACTIVE engine.
  const transformSig = derived<Transform>(() => canvas().transform())
  const debounced = debounce(transformSig, 200)
  // The world-layer transform — pan (x, y) + zoom (scale) off the ACTIVE
  // engine, in world units.
  const worldTransformCss = derived<string>(() => {
    const t = canvas().transform()
    return `translate(${t.x}px, ${t.y}px) scale(${t.scale})`
  })

  const pointerDown = (e: PointerEvent): void => {
    canvas().onPointerDown(e)
  }
  const pointerMove = (e: PointerEvent): void => {
    canvas().onPointerMove(e)
  }
  const pointerUp = (e: PointerEvent): void => {
    canvas().onPointerUp(e)
  }
  const wheel = (e: WheelEvent): void => {
    canvas().onWheel(e)
    e.preventDefault()
  }

  mount((host) => {
    // canvas identity — a composing face (sail-board / sail-draw) injects
    // its engine (built with getViewportElement:()=>null) here; we rewire it
    // onto sail-canvas's REAL viewport so screenToWorld/pan/zoom share ONE
    // engine. Mirrors the sail-calendar calendar identity pattern.
    Object.defineProperty(host, 'canvas', {
      configurable: true,
      get: () => injected(),
      set: (c: Canvas | null) => {
        if (c) c.setOptions({ getViewportElement: () => viewportEl() })
        injected.set(c)
      },
    })

    // The typed scope OVER-CONSTRAINS union-typed reads — read the
    // reflected enum/number props imperatively off the host (the verified
    // sail-table.controller.ts:95 precedent), not through the typed scope.
    const minScale = host.getAttribute('min-scale')
    const maxScale = host.getAttribute('max-scale')
    canvas().setOptions({
      ...(minScale != null ? { minScale: Number(minScale) } : {}),
      ...(maxScale != null ? { maxScale: Number(maxScale) } : {}),
    })

    // Emit the debounced viewport event off the active engine.
    const stopEmit = effect(() => {
      const t = debounced()
      if (!t) return
      host.emit!('sail-canvas-viewport', { transform: t })
    })

    return () => {
      stopEmit()
      delete (host as { canvas?: unknown }).canvas
      fallback.dispose()
    }
  })

  const captureViewport: Action<void> = (node) => {
    viewportEl.set(node as HTMLDivElement)
    return { destroy: () => viewportEl.set(null) }
  }
  return { canvas, injected, fallback, captureViewport, debounced, worldTransformCss, pointerDown, pointerMove, pointerUp, wheel }
})

</controller>
<div
  role="application"
  aria-label={label()}
  use:captureViewport
  on:pointerdown={pointerDown}
  on:pointermove={pointerMove}
  on:pointerup={pointerUp}
  on:wheel={wheel}

>
  style.position(relative).w(full).h(full).minH(panel-min-w-lg).bg(background).fg(foreground).borderWidth(1).border(border).radius(md).overflow(hidden).cursor(grab)

  <div use:worldTransform={worldTransformCss()}>
    style.position(absolute).top(0).left(0).transform(bind(worldTx))

    <slot></slot>
  </div>
</div>
