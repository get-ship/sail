// @ship/sail-canvas — the spatial substrate pack.
//
// <sail-canvas> is the bare pan/zoom/infinite-surface primitive (backbone #3),
// built on @ship/sail-runtime signals — no React, no Excalidraw, no SDK. A
// single `import '@ship/sail-canvas'` registers the tag globally; the compiled
// tag calls customElements.define() on import.
//
// The shared engine is exported for the sibling faces (sail-board, sail-draw)
// to build their domain models on the same coordinate substrate.
import './sail-canvas/sail-canvas.compiled.js'
import './sail-board/sail-board.compiled.js'
import './sail-draw/sail-draw.compiled.js'
import './sail-graph-canvas/sail-graph-canvas.compiled.js'

// Annotation subsystem — the framework-agnostic capture overlay
// (<sail-annotation-layer>) plus its marker + list faces. Pure runtime
// signals; POSTs to a configurable endpoint, no SDK. Lives beside the
// spatial faces because annotations are a canvas-surface concern.
import './sail-annotation/sail-annotation-layer.compiled.js'
import './sail-annotation/sail-annotation-marker.compiled.js'
import './sail-annotation/sail-annotation-list.compiled.js'

export { createCanvas } from './engine/create-canvas'
export { createBoardModel } from './sail-board/engine/board-model'
export { layoutBoard } from './sail-board/engine/layout-adapter'
export type { LayoutKind, LayoutOptions } from './sail-board/engine/layout-adapter'
export { createDrawScene } from './sail-draw/engine/scene'
export type {
  DrawScene,
  DrawElement,
  DrawTool,
  StrokeElement,
  ShapeElement,
  TextElement,
  RenderedElement,
} from './sail-draw/engine/scene'
export type { ShapeKind } from './sail-draw/engine/shape'
export type { StrokeSample } from './sail-draw/engine/stroke'
export type {
  BoardModel,
  Node,
  Edge,
  Port,
  PortRef,
  EdgeRoute,
  PendingEdge,
} from './sail-board/engine/board-model'
export type {
  Canvas,
  CanvasOptions,
  Transform,
  Vec2,
  Rect,
  PointerState,
} from './engine/types'
