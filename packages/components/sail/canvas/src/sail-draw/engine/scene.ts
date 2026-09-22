// sail-draw — the whiteboard scene model on the createCanvas() substrate.
//
// Pure reactive state on @ship/sail-runtime signals: a flat list of draw
// elements (freehand strokes, rectangles, ellipses, lines, arrows, text) +
// the active tool + selection + a command-stack undo/redo. This REPLACES the
// deleted Excalidraw bridge — the scene-graph is OURS (SVG-native, like the
// board), backed by perfect-freehand for pen outlines and roughjs for the
// optional hand-drawn aesthetic. Geometry lives in WORLD coordinates; the view
// maps to screen through the substrate transform. Path-string rendering is in
// stroke.ts/shape.ts/rough-adapter.ts.

import { signal, derived, batch, untrack, type Derived } from '@ship/sail-runtime'
import type { Rect, Vec2 } from '../../engine/types'
import { strokePath, strokeBounds, type StrokeSample } from './stroke'
import { shapePath, rectFromPoints, type ShapeKind } from './shape'
import { roughPath } from './rough-adapter'

export type DrawTool = 'select' | 'pan' | 'pen' | 'rect' | 'ellipse' | 'arrow' | 'line' | 'text'

export interface StrokeElement {
  id: string
  kind: 'pen'
  samples: StrokeSample[]
  color: string
}

export interface ShapeElement {
  id: string
  kind: ShapeKind
  a: Vec2
  b: Vec2
  color: string
  seed: number
}

export interface TextElement {
  id: string
  kind: 'text'
  at: Vec2
  text: string
  color: string
}

export type DrawElement = StrokeElement | ShapeElement | TextElement

/** A render-ready element: its SVG path (or text) in WORLD coords + AABB. */
export interface RenderedElement {
  id: string
  kind: DrawElement['kind']
  path: string
  text?: string
  at?: Vec2
  color: string
  bounds: Rect
}

export interface SceneOptions {
  elements?: DrawElement[]
  /** Roughen shapes with the hand-drawn aesthetic (roughjs). Default false. */
  sketchy?: boolean
}

export interface DrawScene {
  elements: Derived<DrawElement[]>
  rendered: Derived<RenderedElement[]>
  tool: Derived<DrawTool>
  selection: Derived<string[]>
  selectionBounds: Derived<Rect | null>
  contentBounds: Derived<Rect | null>
  canUndo: Derived<boolean>
  canRedo: Derived<boolean>
  // tool
  setTool(tool: DrawTool): void
  // element lifecycle (each pushes onto the undo stack)
  addElement(el: DrawElement): void
  updateElement(id: string, patch: Partial<DrawElement>): void
  removeElements(ids: string[]): void
  // live drafting (NOT history — committed on pointerup via addElement)
  beginStroke(at: Vec2, color: string): string
  extendStroke(sample: StrokeSample): void
  beginShape(kind: ShapeKind, at: Vec2, color: string): string
  extendShape(to: Vec2): void
  commitDraft(): DrawElement | null
  cancelDraft(): void
  draft: Derived<RenderedElement | null>
  // selection
  setSelection(ids: string[]): void
  toggleSelection(id: string): void
  selectInRect(box: Rect, additive?: boolean): void
  clearSelection(): void
  moveSelectionBy(dx: number, dy: number): void
  // history
  undo(): void
  redo(): void
  // serialization
  serialize(): string
}

let seedCounter = 1

function elementBounds(el: DrawElement): Rect {
  if (el.kind === 'pen') return strokeBounds(el.samples)
  if (el.kind === 'text') return { x: el.at.x, y: el.at.y, w: 0, h: 0 }
  return rectFromPoints(el.a, el.b)
}

function rectsIntersect(a: Rect, b: Rect): boolean {
  return a.x < b.x + b.w && a.x + a.w > b.x && a.y < b.y + b.h && a.y + a.h > b.y
}

function unionBounds(rects: Rect[]): Rect | null {
  if (rects.length === 0) return null
  let minX = Infinity
  let minY = Infinity
  let maxX = -Infinity
  let maxY = -Infinity
  for (const r of rects) {
    minX = Math.min(minX, r.x)
    minY = Math.min(minY, r.y)
    maxX = Math.max(maxX, r.x + r.w)
    maxY = Math.max(maxY, r.y + r.h)
  }
  return { x: minX, y: minY, w: maxX - minX, h: maxY - minY }
}

function translateElement(el: DrawElement, dx: number, dy: number): DrawElement {
  if (el.kind === 'pen') {
    return { ...el, samples: el.samples.map((s) => ({ ...s, x: s.x + dx, y: s.y + dy })) }
  }
  if (el.kind === 'text') {
    return { ...el, at: { x: el.at.x + dx, y: el.at.y + dy } }
  }
  return { ...el, a: { x: el.a.x + dx, y: el.a.y + dy }, b: { x: el.b.x + dx, y: el.b.y + dy } }
}

/** Render one element to an SVG path (sketchy shapes via roughjs when set). */
function renderElement(el: DrawElement, sketchy: boolean): RenderedElement {
  const bounds = elementBounds(el)
  if (el.kind === 'pen') {
    return { id: el.id, kind: 'pen', path: strokePath(el.samples), color: el.color, bounds }
  }
  if (el.kind === 'text') {
    return { id: el.id, kind: 'text', path: '', text: el.text, at: el.at, color: el.color, bounds }
  }
  const path = sketchy ? roughPath(el.kind, el.a, el.b, el.seed) : shapePath(el.kind, el.a, el.b)
  return { id: el.id, kind: el.kind, path, color: el.color, bounds }
}

type Draft =
  | { kind: 'pen'; id: string; samples: StrokeSample[]; color: string }
  | { kind: ShapeKind; id: string; a: Vec2; b: Vec2; color: string; seed: number }
  | null

export function createDrawScene(options: SceneOptions = {}): DrawScene {
  const sketchy = options.sketchy ?? false
  const elements = signal<DrawElement[]>(options.elements ?? [])
  const tool = signal<DrawTool>('select')
  const selection = signal<string[]>([])
  const draft = signal<Draft>(null)
  // command stack: each entry is a full snapshot of the elements list.
  const undoStack = signal<DrawElement[][]>([])
  const redoStack = signal<DrawElement[][]>([])

  function pushHistory(prev: DrawElement[]): void {
    batch(() => {
      undoStack.set([...untrack(undoStack), prev])
      redoStack.set([])
    })
  }

  const rendered = derived<RenderedElement[]>(() => elements().map((el) => renderElement(el, sketchy)))

  const draftRendered = derived<RenderedElement | null>(() => {
    const d = draft()
    if (!d) return null
    const el: DrawElement =
      d.kind === 'pen'
        ? { id: d.id, kind: 'pen', samples: d.samples, color: d.color }
        : { id: d.id, kind: d.kind, a: d.a, b: d.b, color: d.color, seed: d.seed }
    return renderElement(el, sketchy)
  })

  const selectionBounds = derived<Rect | null>(() => {
    const sel = new Set(selection())
    if (sel.size === 0) return null
    return unionBounds(elements().filter((el) => sel.has(el.id)).map(elementBounds))
  })

  const contentBounds = derived<Rect | null>(() => unionBounds(elements().map(elementBounds)))

  // --- tool ---
  function setTool(t: DrawTool): void {
    tool.set(t)
  }

  // --- element lifecycle ---
  function addElement(el: DrawElement): void {
    pushHistory(untrack(elements))
    elements.set([...untrack(elements), el])
  }
  function updateElement(id: string, patch: Partial<DrawElement>): void {
    pushHistory(untrack(elements))
    elements.set(untrack(elements).map((el) => (el.id === id ? ({ ...el, ...patch } as DrawElement) : el)))
  }
  function removeElements(ids: string[]): void {
    const drop = new Set(ids)
    pushHistory(untrack(elements))
    batch(() => {
      elements.set(untrack(elements).filter((el) => !drop.has(el.id)))
      selection.set(untrack(selection).filter((s) => !drop.has(s)))
    })
  }

  // --- live drafting ---
  function beginStroke(at: Vec2, color: string): string {
    const id = `pen:${Date.now()}:${seedCounter++}`
    draft.set({ kind: 'pen', id, samples: [{ x: at.x, y: at.y, pressure: 0.5 }], color })
    return id
  }
  function extendStroke(sample: StrokeSample): void {
    const d = untrack(draft)
    if (d?.kind !== 'pen') return
    draft.set({ ...d, samples: [...d.samples, sample] })
  }
  function beginShape(kind: ShapeKind, at: Vec2, color: string): string {
    const id = `${kind}:${Date.now()}:${seedCounter}`
    draft.set({ kind, id, a: at, b: at, color, seed: seedCounter++ })
    return id
  }
  function extendShape(to: Vec2): void {
    const d = untrack(draft)
    if (!d || d.kind === 'pen') return
    draft.set({ ...d, b: to })
  }
  function commitDraft(): DrawElement | null {
    const d = untrack(draft)
    draft.set(null)
    if (!d) return null
    const el: DrawElement =
      d.kind === 'pen'
        ? { id: d.id, kind: 'pen', samples: d.samples, color: d.color }
        : { id: d.id, kind: d.kind, a: d.a, b: d.b, color: d.color, seed: d.seed }
    // discard degenerate drafts (a tap with no extent)
    if (el.kind === 'pen') {
      if (el.samples.length < 2) return null
    } else {
      // a draft is always a pen stroke or a shape — never text
      const r = rectFromPoints(el.a, el.b)
      if (r.w < 2 && r.h < 2) return null
    }
    addElement(el)
    return el
  }
  function cancelDraft(): void {
    draft.set(null)
  }

  // --- selection ---
  function setSelection(ids: string[]): void {
    selection.set([...ids])
  }
  function toggleSelection(id: string): void {
    const cur = untrack(selection)
    selection.set(cur.includes(id) ? cur.filter((s) => s !== id) : [...cur, id])
  }
  function selectInRect(box: Rect, additive = false): void {
    const hit = untrack(elements)
      .filter((el) => rectsIntersect(elementBounds(el), box))
      .map((el) => el.id)
    selection.set(additive ? [...new Set([...untrack(selection), ...hit])] : hit)
  }
  function clearSelection(): void {
    selection.set([])
  }
  function moveSelectionBy(dx: number, dy: number): void {
    const sel = new Set(untrack(selection))
    if (sel.size === 0) return
    pushHistory(untrack(elements))
    elements.set(untrack(elements).map((el) => (sel.has(el.id) ? translateElement(el, dx, dy) : el)))
  }

  // --- history ---
  function undo(): void {
    const stack = untrack(undoStack)
    if (stack.length === 0) return
    const prev = stack[stack.length - 1]!
    batch(() => {
      redoStack.set([...untrack(redoStack), untrack(elements)])
      undoStack.set(stack.slice(0, -1))
      elements.set(prev)
    })
  }
  function redo(): void {
    const stack = untrack(redoStack)
    if (stack.length === 0) return
    const next = stack[stack.length - 1]!
    batch(() => {
      undoStack.set([...untrack(undoStack), untrack(elements)])
      redoStack.set(stack.slice(0, -1))
      elements.set(next)
    })
  }

  function serialize(): string {
    return JSON.stringify({ elements: untrack(elements) })
  }

  return {
    elements: elements as Derived<DrawElement[]>,
    rendered,
    tool: tool as Derived<DrawTool>,
    selection: selection as Derived<string[]>,
    selectionBounds,
    contentBounds,
    canUndo: derived(() => undoStack().length > 0),
    canRedo: derived(() => redoStack().length > 0),
    setTool,
    addElement,
    updateElement,
    removeElements,
    beginStroke,
    extendStroke,
    beginShape,
    extendShape,
    commitDraft,
    cancelDraft,
    draft: draftRendered,
    setSelection,
    toggleSelection,
    selectInRect,
    clearSelection,
    moveSelectionBy,
    undo,
    redo,
    serialize,
  }
}
