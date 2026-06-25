// @ship/sui-canvas — the spatial substrate pack.
//
// <sui-canvas> is the bare pan/zoom/infinite-surface primitive (backbone #3),
// built on @ship/sui-runtime signals — no React, no Excalidraw, no SDK. A
// single `import '@ship/sui-canvas'` registers the tag globally; the compiled
// tag calls customElements.define() on import.
//
// The shared engine is exported for the sibling faces (sui-board, sui-draw)
// to build their domain models on the same coordinate substrate.
import './sui-canvas/sui-canvas.compiled.js'
import './sui-board/sui-board.compiled.js'
import './sui-draw/sui-draw.compiled.js'
import './sui-graph-canvas/sui-graph-canvas.compiled.js'

// Annotation subsystem — the framework-agnostic capture overlay
// (<sui-annotation-layer>) plus its marker + list faces. Pure runtime
// signals; POSTs to a configurable endpoint, no SDK. Lives beside the
// spatial faces because annotations are a canvas-surface concern.
import './sui-annotation/sui-annotation-layer.compiled.js'
import './sui-annotation/sui-annotation-marker.compiled.js'
import './sui-annotation/sui-annotation-list.compiled.js'

export { createCanvas } from './engine/create-canvas'
export { createBoardModel } from './sui-board/engine/board-model'
export { layoutBoard } from './sui-board/engine/layout-adapter'
export type { LayoutKind, LayoutOptions } from './sui-board/engine/layout-adapter'
export { createDrawScene } from './sui-draw/engine/scene'
export type {
  DrawScene,
  DrawElement,
  DrawTool,
  StrokeElement,
  ShapeElement,
  TextElement,
  RenderedElement,
} from './sui-draw/engine/scene'
export type { ShapeKind } from './sui-draw/engine/shape'
export type { StrokeSample } from './sui-draw/engine/stroke'
export type {
  BoardModel,
  Node,
  Edge,
  Port,
  PortRef,
  EdgeRoute,
  PendingEdge,
} from './sui-board/engine/board-model'
export type {
  Canvas,
  CanvasOptions,
  Transform,
  Vec2,
  Rect,
  PointerState,
} from './engine/types'
