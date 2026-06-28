import { controller, signal, derived, effect, debounce, ref } from '@ship/sui-runtime'
import type { Scope } from './sui-canvas.sui'
import { createCanvas } from '../engine/create-canvas'
import type { Canvas, Transform } from '../engine/types'

// SSR fallback = the verified sui-combobox shape: plain JSON.parse in the
// controller, try/caught. The live path is prop:* identity onto the engine.
function parseSnapshot(raw: string): { transform?: Partial<Transform> } | null {
  if (!raw) return null
  try {
    return JSON.parse(raw) as { transform?: Partial<Transform> }
  } catch {
    return null
  }
}

// <sui-canvas> controller — the bare substrate primitive. Builds a
// createCanvas() in scope, forwards raw pointer/wheel events to the engine
// in on:*, and emits a debounced sui-canvas-viewport event via host.emit!.
export default controller({
  scope: ({ value }: Scope) => {
    const viewportEl = ref<HTMLDivElement>(null)
    // Fallback engine — used when no engine is injected via prop:canvas.
    const fallback = createCanvas({
      getViewportElement: () => viewportEl(),
      initialTransform: parseSnapshot(value())?.transform,
    })
    // An engine injected by a composing face (sui-board / sui-draw) via
    // prop:canvas identity. When present it supersedes the fallback so the
    // board/draw transform and sui-canvas pan/zoom share ONE engine.
    const injected = signal<Canvas | null>(null)
    const canvas = derived<Canvas>(() => injected() ?? fallback)
    // Drives the debounced outbound viewport event off the ACTIVE engine.
    const transformSig = derived<Transform>(() => canvas().transform())
    const debounced = debounce(transformSig, 200)
    // The world-layer transform. This MUST live in the scope return (not a
    // top-level `derived:` section): the current ScopedControllerDef merges
    // only scope/on/use/mount/unmount, so a `derived:` block is silently
    // dropped and the world would never receive its transform (the rest of the
    // codebase inlines derived() in scope for exactly this reason).
    const transformCss = derived<string>(() => {
      const t = canvas().transform()
      return `transform: translate(${t.x}px, ${t.y}px) scale(${t.scale})`
    })
    return { canvas, injected, fallback, viewportEl, debounced, transformCss }
  },

  on: {
    pointerDown: (s, e: PointerEvent) => {
      s.canvas.onPointerDown(e)
    },
    pointerMove: (s, e: PointerEvent) => {
      s.canvas.onPointerMove(e)
    },
    pointerUp: (s, e: PointerEvent) => {
      s.canvas.onPointerUp(e)
    },
    wheel: (s, e: WheelEvent) => {
      s.canvas.onWheel(e)
      e.preventDefault()
    },
  },

  mount(state, host) {
    // prop:canvas identity — a composing face (sui-board / sui-draw) injects
    // its engine (built with getViewportElement:()=>null) here; we rewire it
    // onto sui-canvas's REAL viewport so screenToWorld/pan/zoom share ONE
    // engine. Mirrors the sui-calendar prop:calendar identity pattern.
    Object.defineProperty(host, 'canvas', {
      configurable: true,
      get: () => state.$signals.injected(),
      set: (c: Canvas | null) => {
        if (c) c.setOptions({ getViewportElement: () => state.$signals.viewportEl() })
        state.injected = c
      },
    })

    // The typed scope OVER-CONSTRAINS union-typed reads — read the
    // reflected enum/number props imperatively off the host (the verified
    // sui-table.controller.ts:95 precedent), not through the typed scope.
    const minScale = host.getAttribute('min-scale')
    const maxScale = host.getAttribute('max-scale')
    state.canvas.setOptions({
      ...(minScale != null ? { minScale: Number(minScale) } : {}),
      ...(maxScale != null ? { maxScale: Number(maxScale) } : {}),
    })

    // Emit the debounced viewport event off the active engine.
    const stopEmit = effect(() => {
      const t = state.debounced
      if (!t) return
      host.emit!('sui-canvas-viewport', { transform: t })
    })

    return () => {
      stopEmit()
      delete (host as { canvas?: unknown }).canvas
      state.fallback.dispose()
    }
  },
})
