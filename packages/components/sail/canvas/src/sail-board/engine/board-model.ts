// sail-board — the board domain model on the createCanvas() substrate.
//
// Pure reactive state on @ship/sail-runtime signals: nodes/edges/ports +
// selection + edge routing + drag/snap + pending-edge (drag-from-port). The
// board is SVG-NATIVE — there is NO canvas engine. Geometry lives in world
// coordinates; the view maps to screen through the substrate's
// worldToScreen. Heavy edge-path math lives in `edge-route.ts`.

import { signal, derived, batch, untrack, type Derived } from '@ship/sail-runtime'
import type { Rect, Vec2 } from '../../engine/types'
import { routeEdge, type EdgeKind } from './edge-route'
import { layoutBoard, type LayoutOptions } from './layout-adapter'

export type PortSide = 'left' | 'right' | 'top' | 'bottom'

/** An anchor point on a node, located by side + fractional offset [0..1]. */
export interface Port {
  id: string
  side: PortSide
  offset: number
}

export interface Node {
  id: string
  rect: Rect
  ports: Port[]
  kind: string
  data: unknown
}

/** A reference to a specific port on a node. */
export interface PortRef {
  node: string
  port: string
}

export interface Edge {
  id: string
  source: PortRef
  target: PortRef
  kind: EdgeKind
}

/** A resolved, ready-to-render edge route. */
export interface EdgeRoute {
  id: string
  kind: EdgeKind
  start: Vec2
  end: Vec2
  path: string
}

/** A pending edge being dragged from a port toward a free world point. */
export interface PendingEdge {
  source: PortRef
  start: Vec2
  end: Vec2
}

export interface BoardModelOptions {
  nodes?: Node[]
  edges?: Edge[]
  /** Grid step in world units; 0 = no snap. */
  snapToGrid?: number
}

export interface BoardModel {
  nodes: Derived<Node[]>
  edges: Derived<Edge[]>
  routes: Derived<EdgeRoute[]>
  selectedNodes: Derived<string[]>
  incidentEdges: Derived<string[]>
  pendingEdge: Derived<PendingEdge | null>
  contentBounds: Derived<Rect | null>
  // node registry
  addNode(n: Node): void
  updateNode(id: string, patch: Partial<Node>): void
  removeNode(id: string): void
  // geometry
  portAnchor(nodeId: string, portId: string): Vec2 | null
  // dragging
  moveNodeBy(id: string, dx: number, dy: number): void
  moveSelectionBy(dx: number, dy: number): void
  // auto-layout
  applyLayout(opts: LayoutOptions): void
  // selection
  setSelection(ids: string[]): void
  toggleSelection(id: string): void
  selectInRect(box: Rect, additive?: boolean): void
  clearSelection(): void
  // pending edge (drag-from-port)
  beginEdge(source: PortRef): void
  updatePendingEnd(p: Vec2): void
  commitEdge(target: PortRef): Edge | null
  cancelEdge(): void
  // serialization
  serialize(): string
}

function snap(v: number, step: number): number {
  return step > 0 ? Math.round(v / step) * step : v
}

function rectsIntersect(a: Rect, b: Rect): boolean {
  return a.x < b.x + b.w && a.x + a.w > b.x && a.y < b.y + b.h && a.y + a.h > b.y
}

/** The world point of a port on a node rect, from side + offset. */
function anchorOf(node: Node, port: Port): Vec2 {
  const { x, y, w, h } = node.rect
  switch (port.side) {
    case 'left':
      return { x, y: y + h * port.offset }
    case 'right':
      return { x: x + w, y: y + h * port.offset }
    case 'top':
      return { x: x + w * port.offset, y }
    case 'bottom':
      return { x: x + w * port.offset, y: y + h }
  }
}

export function createBoardModel(options: BoardModelOptions = {}): BoardModel {
  const step = options.snapToGrid ?? 0
  const nodes = signal<Node[]>(options.nodes ?? [])
  const edges = signal<Edge[]>(options.edges ?? [])
  const selection = signal<string[]>([])
  const pending = signal<PendingEdge | null>(null)

  const nodeById = derived(() => {
    const map = new Map<string, Node>()
    for (const n of nodes()) map.set(n.id, n)
    return map
  })

  function resolveAnchor(map: Map<string, Node>, ref: PortRef): Vec2 | null {
    const node = map.get(ref.node)
    if (!node) return null
    const port = node.ports.find((p) => p.id === ref.port)
    return port ? anchorOf(node, port) : null
  }

  const routes = derived<EdgeRoute[]>(() => {
    const map = nodeById()
    const out: EdgeRoute[] = []
    for (const e of edges()) {
      const start = resolveAnchor(map, e.source)
      const end = resolveAnchor(map, e.target)
      if (!start || !end) continue
      out.push({ id: e.id, kind: e.kind, start, end, path: routeEdge(e.kind, start, end) })
    }
    return out
  })

  const incidentEdges = derived<string[]>(() => {
    const sel = new Set(selection())
    if (sel.size === 0) return []
    return edges()
      .filter((e) => sel.has(e.source.node) || sel.has(e.target.node))
      .map((e) => e.id)
  })

  const contentBounds = derived<Rect | null>(() => {
    const ns = nodes()
    if (ns.length === 0) return null
    let minX = Infinity
    let minY = Infinity
    let maxX = -Infinity
    let maxY = -Infinity
    for (const n of ns) {
      minX = Math.min(minX, n.rect.x)
      minY = Math.min(minY, n.rect.y)
      maxX = Math.max(maxX, n.rect.x + n.rect.w)
      maxY = Math.max(maxY, n.rect.y + n.rect.h)
    }
    return { x: minX, y: minY, w: maxX - minX, h: maxY - minY }
  })

  function addNode(n: Node): void {
    nodes.set([...untrack(nodes), n])
  }
  function updateNode(id: string, patch: Partial<Node>): void {
    nodes.set(untrack(nodes).map((n) => (n.id === id ? { ...n, ...patch } : n)))
  }
  function removeNode(id: string): void {
    batch(() => {
      nodes.set(untrack(nodes).filter((n) => n.id !== id))
      edges.set(untrack(edges).filter((e) => e.source.node !== id && e.target.node !== id))
      selection.set(untrack(selection).filter((s) => s !== id))
    })
  }

  function portAnchor(nodeId: string, portId: string): Vec2 | null {
    return resolveAnchor(untrack(nodeById), { node: nodeId, port: portId })
  }

  function moveNodeBy(id: string, dx: number, dy: number): void {
    nodes.set(
      untrack(nodes).map((n) =>
        n.id === id
          ? { ...n, rect: { ...n.rect, x: snap(n.rect.x + dx, step), y: snap(n.rect.y + dy, step) } }
          : n,
      ),
    )
  }
  function moveSelectionBy(dx: number, dy: number): void {
    const sel = new Set(untrack(selection))
    nodes.set(
      untrack(nodes).map((n) =>
        sel.has(n.id)
          ? { ...n, rect: { ...n.rect, x: snap(n.rect.x + dx, step), y: snap(n.rect.y + dy, step) } }
          : n,
      ),
    )
  }

  function applyLayout(opts: LayoutOptions): void {
    if (opts.layout === 'none') return
    const next = layoutBoard(untrack(nodes), untrack(edges), opts)
    nodes.set(next)
  }

  function setSelection(ids: string[]): void {
    selection.set([...ids])
  }
  function toggleSelection(id: string): void {
    const cur = untrack(selection)
    selection.set(cur.includes(id) ? cur.filter((s) => s !== id) : [...cur, id])
  }
  function selectInRect(box: Rect, additive = false): void {
    const hit = untrack(nodes).filter((n) => rectsIntersect(n.rect, box)).map((n) => n.id)
    selection.set(additive ? [...new Set([...untrack(selection), ...hit])] : hit)
  }
  function clearSelection(): void {
    selection.set([])
  }

  function beginEdge(source: PortRef): void {
    const start = portAnchor(source.node, source.port)
    if (!start) return
    pending.set({ source, start, end: start })
  }
  function updatePendingEnd(p: Vec2): void {
    const cur = untrack(pending)
    if (cur) pending.set({ ...cur, end: p })
  }
  function commitEdge(target: PortRef): Edge | null {
    const cur = untrack(pending)
    pending.set(null)
    if (!cur || cur.source.node === target.node) return null
    const edge: Edge = {
      id: `e:${cur.source.port}->${target.port}:${untrack(edges).length}`,
      source: cur.source,
      target,
      kind: 'bezier',
    }
    edges.set([...untrack(edges), edge])
    return edge
  }
  function cancelEdge(): void {
    pending.set(null)
  }

  function serialize(): string {
    return JSON.stringify({ nodes: untrack(nodes), edges: untrack(edges) })
  }

  return {
    nodes: nodes as Derived<Node[]>,
    edges: edges as Derived<Edge[]>,
    routes,
    selectedNodes: selection as Derived<string[]>,
    incidentEdges,
    pendingEdge: pending as Derived<PendingEdge | null>,
    contentBounds,
    addNode,
    updateNode,
    removeNode,
    portAnchor,
    moveNodeBy,
    moveSelectionBy,
    applyLayout,
    setSelection,
    toggleSelection,
    selectInRect,
    clearSelection,
    beginEdge,
    updatePendingEnd,
    commitEdge,
    cancelEdge,
    serialize,
  }
}
