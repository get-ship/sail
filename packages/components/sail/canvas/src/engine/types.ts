// Backbone #3 — the shared pan/zoom/pointer substrate type surface.
//
// Pure geometry/state types for `createCanvas()`. No DOM, no SDK, no
// runtime imports beyond `@ship/sail-runtime` signal types. A `Vec2` is a
// world OR screen point — the calling context names which.

import type { Signal, Derived } from '@ship/sail-runtime'

/** A 2-D point — world OR screen space (context names which). */
export interface Vec2 {
  x: number
  y: number
}

/** Viewport transform: pan offset in screen px + zoom factor. */
export interface Transform {
  x: number
  y: number
  scale: number
}

/** A world-space axis-aligned bounding box. */
export interface Rect {
  x: number
  y: number
  w: number
  h: number
}

/** The pointer gesture state machine — a discriminated union. A face
 *  (board node-drag, draw stroke) claims `face-drag`; otherwise the
 *  substrate owns the gesture (pan/box-select). */
export type PointerState =
  | { kind: 'idle' }
  | { kind: 'panning'; start: Vec2; origin: Transform }
  | { kind: 'box-select'; start: Vec2 }
  | { kind: 'pinch'; startDist: number; startScale: number; center: Vec2 }
  | { kind: 'face-drag'; itemId: string; start: Vec2 }

export interface GestureOptions {
  /** Which mouse button pans (default middle = 1). */
  panButton?: 0 | 1 | 2
  pinchZoom?: boolean
  wheelZoom?: boolean
  wheelPan?: boolean
  boxSelect?: boolean
}

export interface CanvasOptions {
  /** Viewport element thunk — null until mount (mirrors
   *  `createVirtualizer`'s `getScrollElement`). */
  getViewportElement: () => any | null
  /** Uncontrolled seed; the engine owns the transform thereafter. */
  initialTransform?: Partial<Transform>
  /** Zoom clamp (default 0.1 … 8). */
  minScale?: number
  maxScale?: number
  zoomSensitivity?: number
  panInverted?: boolean
  gestures?: GestureOptions
  /** Callbacks — the only path the substrate "talks out". */
  onTransformChange?: (t: Transform) => void
  onPointerWorld?: (p: Vec2, e: any) => void
}

/** Each state slice is a `Signal<…>`. */
export interface ReactiveCanvasState {
  transform: Signal<Transform>
  pointer: Signal<PointerState>
  pointerScreen: Signal<Vec2 | null>
  viewportSize: Signal<{ w: number; h: number }>
}

export interface Canvas {
  // VIEWPORT / TRANSFORM
  transform: Derived<Transform>
  scale: Derived<number>
  pan: Derived<Vec2>
  viewportSize: Derived<{ w: number; h: number }>
  visibleWorldRect: Derived<Rect>
  isPanning: Derived<boolean>
  isPinching: Derived<boolean>
  pointerWorld: Derived<Vec2 | null>
  // COORDINATE MAPPING
  screenToWorld(p: Vec2): Vec2
  worldToScreen(p: Vec2): Vec2
  // STATE
  state: ReactiveCanvasState
  // METHODS — mutators; call ONLY from on:* / mount.
  setTransform(t: Partial<Transform>): void
  panBy(dx: number, dy: number): void
  panTo(p: Vec2): void
  zoomBy(factor: number, centerScreen?: Vec2): void
  zoomTo(scale: number, centerScreen?: Vec2): void
  fitToRect(r: Rect, opts?: { padding?: number; maxScale?: number }): void
  resetView(): void
  // POINTER PUMP — the view forwards raw DOM events.
  onPointerDown(e: any): void
  onPointerMove(e: any): void
  onPointerUp(e: any): void
  onWheel(e: any): void
  setOptions(patch: Partial<CanvasOptions>): void
  dispose(): void
}

export const DEFAULT_MIN_SCALE = 0.1
export const DEFAULT_MAX_SCALE = 8
export const DEFAULT_ZOOM_SENSITIVITY = 0.0015
