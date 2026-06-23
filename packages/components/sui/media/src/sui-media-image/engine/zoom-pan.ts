// sui-media engine — the image-stage zoom/pan interaction. PURE controller
// glue extracted from the renderer's `mount` so the controller stays glue-only
// (build-spec Slice 6 pre-committed split). No `@ship/sui-runtime`, no DOM
// construction — it wires listeners onto an already-mounted `part="stage"`
// element and reports scale/pan changes through a callback.
//
// Source-in, never fetch: pure interaction math + event wiring, no I/O.
//
// Authority: build-spec Slice 6 (zoomable wheel/+/-/arrow-pan), §Accessibility
// (keyboard-operable zoom/pan), §Props (sui-viewer-zoom {scale}).

/** Clamp helper — keep the zoom scale inside sane bounds. */
const clamp = (n: number, lo: number, hi: number): number => Math.min(hi, Math.max(lo, n))

export const MIN_SCALE = 0.25
export const MAX_SCALE = 8
const STEP = 0.25
const PAN_STEP = 24

export interface ZoomPanState {
  scale: number
  x: number
  y: number
}

export interface ZoomPanOptions {
  /** The zoom/pan surface (`part="stage"`). */
  stage: HTMLElement
  /** Read whether zoom/pan is currently enabled (the `zoomable` prop). */
  enabled: () => boolean
  /** Called after any scale/pan change with the new transform state. */
  onChange: (state: ZoomPanState) => void
  /** Called specifically when the scale changes (drives `sui-viewer-zoom`). */
  onZoom: (scale: number) => void
}

/**
 * Attach wheel + keyboard (+/-, arrows) zoom/pan to a stage element. Returns a
 * teardown that removes every listener. The caller owns rendering the transform
 * (the controller applies `state` to the `<img>` via a reactive effect).
 */
export function attachZoomPan(opts: ZoomPanOptions): () => void {
  const { stage, enabled, onChange, onZoom } = opts
  const state: ZoomPanState = { scale: 1, x: 0, y: 0 }

  const applyScale = (next: number): void => {
    const clamped = clamp(next, MIN_SCALE, MAX_SCALE)
    if (clamped === state.scale) return
    state.scale = clamped
    onChange({ ...state })
    onZoom(clamped)
  }

  const pan = (dx: number, dy: number): void => {
    state.x += dx
    state.y += dy
    onChange({ ...state })
  }

  const onWheel = (e: WheelEvent): void => {
    if (!enabled()) return
    e.preventDefault()
    // Wheel up (deltaY < 0) zooms in.
    const dir = e.deltaY < 0 ? 1 : -1
    applyScale(state.scale + dir * STEP)
  }

  const onKeyDown = (e: KeyboardEvent): void => {
    if (!enabled()) return
    switch (e.key) {
      case '+':
      case '=':
        e.preventDefault()
        applyScale(state.scale + STEP)
        break
      case '-':
      case '_':
        e.preventDefault()
        applyScale(state.scale - STEP)
        break
      case 'ArrowLeft':
        e.preventDefault()
        pan(PAN_STEP, 0)
        break
      case 'ArrowRight':
        e.preventDefault()
        pan(-PAN_STEP, 0)
        break
      case 'ArrowUp':
        e.preventDefault()
        pan(0, PAN_STEP)
        break
      case 'ArrowDown':
        e.preventDefault()
        pan(0, -PAN_STEP)
        break
      default:
        break
    }
  }

  stage.addEventListener('wheel', onWheel, { passive: false })
  stage.addEventListener('keydown', onKeyDown)

  return () => {
    stage.removeEventListener('wheel', onWheel)
    stage.removeEventListener('keydown', onKeyDown)
  }
}
