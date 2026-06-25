// sui-board — board-model engine behavioral specs (Slice 2).
//
// Pure-runtime: nodes/edges/ports/selection + edge routing on @ship/sui-runtime
// signals. No DOM, no canvas-engine — the board is SVG-native and layers its
// domain model on the createCanvas() substrate's coordinate transforms.
import { describe, expect, it } from 'vitest'
import { createBoardModel } from '../src/sui-board/engine/board-model'
import type { Node, Edge } from '../src/sui-board/engine/board-model'

function node(id: string, x: number, y: number, w = 100, h = 60): Node {
  return {
    id,
    rect: { x, y, w, h },
    kind: 'default',
    ports: [
      { id: `${id}:out`, side: 'right', offset: 0.5 },
      { id: `${id}:in`, side: 'left', offset: 0.5 },
    ],
    data: {},
  }
}

describe('board-model — node registry', () => {
  it('seeds nodes and edges and exposes them as derived slices', () => {
    const b = createBoardModel({
      nodes: [node('a', 0, 0), node('b', 300, 0)],
      edges: [{ id: 'e1', source: { node: 'a', port: 'a:out' }, target: { node: 'b', port: 'b:in' }, kind: 'bezier' }],
    })
    expect(b.nodes().map((n) => n.id)).toEqual(['a', 'b'])
    expect(b.edges().map((e) => e.id)).toEqual(['e1'])
  })

  it('adds, updates, and removes nodes', () => {
    const b = createBoardModel({ nodes: [node('a', 0, 0)] })
    b.addNode(node('b', 200, 0))
    expect(b.nodes()).toHaveLength(2)
    b.updateNode('b', { rect: { x: 250, y: 10, w: 100, h: 60 } })
    expect(b.nodes().find((n) => n.id === 'b')!.rect.x).toBe(250)
    b.removeNode('a')
    expect(b.nodes().map((n) => n.id)).toEqual(['b'])
  })

  it('removing a node prunes its incident edges', () => {
    const b = createBoardModel({
      nodes: [node('a', 0, 0), node('b', 300, 0)],
      edges: [{ id: 'e1', source: { node: 'a', port: 'a:out' }, target: { node: 'b', port: 'b:in' }, kind: 'bezier' }],
    })
    b.removeNode('a')
    expect(b.edges()).toHaveLength(0)
  })
})

describe('board-model — port anchoring (world coords)', () => {
  it('anchors a port to a point on the node rect by side + offset', () => {
    const b = createBoardModel({ nodes: [node('a', 0, 0, 100, 60)] })
    // right side, offset 0.5 → (x + w, y + h*0.5)
    const out = b.portAnchor('a', 'a:out')!
    expect(out).toEqual({ x: 100, y: 30 })
    // left side, offset 0.5 → (x, y + h*0.5)
    const inn = b.portAnchor('a', 'a:in')!
    expect(inn).toEqual({ x: 0, y: 30 })
  })

  it('port anchor follows the node when it moves', () => {
    const b = createBoardModel({ nodes: [node('a', 0, 0, 100, 60)] })
    b.updateNode('a', { rect: { x: 50, y: 20, w: 100, h: 60 } })
    expect(b.portAnchor('a', 'a:out')).toEqual({ x: 150, y: 50 })
  })

  it('returns null for an unknown node or port', () => {
    const b = createBoardModel({ nodes: [node('a', 0, 0)] })
    expect(b.portAnchor('zzz', 'a:out')).toBeNull()
    expect(b.portAnchor('a', 'nope')).toBeNull()
  })
})

describe('board-model — edge routing', () => {
  it('routes a straight edge as a line between port anchors', () => {
    const b = createBoardModel({
      nodes: [node('a', 0, 0, 100, 60), node('b', 300, 0, 100, 60)],
      edges: [{ id: 'e1', source: { node: 'a', port: 'a:out' }, target: { node: 'b', port: 'b:in' }, kind: 'straight' }],
    })
    const routes = b.routes()
    expect(routes).toHaveLength(1)
    expect(routes[0].path).toBe('M 100 30 L 300 30')
  })

  it('routes a bezier edge with horizontal control points', () => {
    const b = createBoardModel({
      nodes: [node('a', 0, 0, 100, 60), node('b', 300, 0, 100, 60)],
      edges: [{ id: 'e1', source: { node: 'a', port: 'a:out' }, target: { node: 'b', port: 'b:in' }, kind: 'bezier' }],
    })
    const r = b.routes()[0]
    // cubic bezier: M start C c1 c2 end — control points pulled along x toward midpoint
    expect(r.path).toMatch(/^M 100 30 C /)
    expect(r.path.endsWith('300 30')).toBe(true)
  })

  it('reroutes reactively when a source node moves', () => {
    const b = createBoardModel({
      nodes: [node('a', 0, 0, 100, 60), node('b', 300, 0, 100, 60)],
      edges: [{ id: 'e1', source: { node: 'a', port: 'a:out' }, target: { node: 'b', port: 'b:in' }, kind: 'straight' }],
    })
    expect(b.routes()[0].path).toBe('M 100 30 L 300 30')
    b.updateNode('a', { rect: { x: 0, y: 100, w: 100, h: 60 } })
    expect(b.routes()[0].path).toBe('M 100 130 L 300 30')
  })

  it('drops a route whose endpoint port no longer resolves', () => {
    const b = createBoardModel({
      nodes: [node('a', 0, 0), node('b', 300, 0)],
      edges: [{ id: 'e1', source: { node: 'a', port: 'a:out' }, target: { node: 'b', port: 'b:in' }, kind: 'straight' }],
    })
    b.removeNode('b')
    expect(b.routes()).toHaveLength(0)
  })
})

describe('board-model — selection semantics', () => {
  it('selects nodes and reports incident edges', () => {
    const b = createBoardModel({
      nodes: [node('a', 0, 0), node('b', 300, 0), node('c', 600, 0)],
      edges: [
        { id: 'e1', source: { node: 'a', port: 'a:out' }, target: { node: 'b', port: 'b:in' }, kind: 'bezier' },
        { id: 'e2', source: { node: 'b', port: 'b:out' }, target: { node: 'c', port: 'c:in' }, kind: 'bezier' },
      ],
    })
    b.setSelection(['b'])
    expect(b.selectedNodes()).toEqual(['b'])
    // b is incident to both e1 (target) and e2 (source)
    expect(new Set(b.incidentEdges())).toEqual(new Set(['e1', 'e2']))
  })

  it('toggleSelection adds and removes additively', () => {
    const b = createBoardModel({ nodes: [node('a', 0, 0), node('b', 300, 0)] })
    b.toggleSelection('a')
    b.toggleSelection('b')
    expect(new Set(b.selectedNodes())).toEqual(new Set(['a', 'b']))
    b.toggleSelection('a')
    expect(b.selectedNodes()).toEqual(['b'])
  })

  it('clearSelection empties the selection', () => {
    const b = createBoardModel({ nodes: [node('a', 0, 0)] })
    b.setSelection(['a'])
    b.clearSelection()
    expect(b.selectedNodes()).toEqual([])
  })

  it('selectInRect picks nodes whose rect intersects a world rect', () => {
    const b = createBoardModel({
      nodes: [node('a', 0, 0, 100, 60), node('b', 300, 0, 100, 60), node('c', 600, 0, 100, 60)],
    })
    // a box covering a and b but not c
    b.selectInRect({ x: -10, y: -10, w: 360, h: 100 })
    expect(new Set(b.selectedNodes())).toEqual(new Set(['a', 'b']))
  })

  it('selectInRect additive keeps prior selection', () => {
    const b = createBoardModel({
      nodes: [node('a', 0, 0, 100, 60), node('b', 300, 0, 100, 60)],
    })
    b.setSelection(['a'])
    b.selectInRect({ x: 290, y: -10, w: 120, h: 100 }, true)
    expect(new Set(b.selectedNodes())).toEqual(new Set(['a', 'b']))
  })
})

describe('board-model — node dragging + snap', () => {
  it('moveNodeBy translates the node rect', () => {
    const b = createBoardModel({ nodes: [node('a', 10, 20, 100, 60)] })
    b.moveNodeBy('a', 5, -7)
    expect(b.nodes()[0].rect).toMatchObject({ x: 15, y: 13 })
  })

  it('moveNodeBy snaps to grid when snapToGrid > 0', () => {
    const b = createBoardModel({ nodes: [node('a', 0, 0, 100, 60)], snapToGrid: 20 })
    b.moveNodeBy('a', 13, 27)
    // 13 → 20, 27 → 20 (nearest multiple of 20)
    expect(b.nodes()[0].rect).toMatchObject({ x: 20, y: 20 })
  })

  it('moves all selected nodes together when dragging a selected one', () => {
    const b = createBoardModel({ nodes: [node('a', 0, 0), node('b', 300, 0)] })
    b.setSelection(['a', 'b'])
    b.moveSelectionBy(10, 10)
    expect(b.nodes().find((n) => n.id === 'a')!.rect.x).toBe(10)
    expect(b.nodes().find((n) => n.id === 'b')!.rect.x).toBe(310)
  })
})

describe('board-model — auto-layout (applyLayout)', () => {
  it('layered repositions overlapping nodes into ranked, non-overlapping rects', () => {
    // Both nodes seeded at the origin (fully overlapping) with an a→b edge.
    const b = createBoardModel({
      nodes: [node('a', 0, 0), node('b', 0, 0)],
      edges: [{ id: 'e1', source: { node: 'a', port: 'a:out' }, target: { node: 'b', port: 'b:in' }, kind: 'bezier' }],
    })
    b.applyLayout({ layout: 'layered' })
    const by = Object.fromEntries(b.nodes().map((n) => [n.id, n.rect]))
    // a is the source rank, b the next rank → b sits to the right (LR default).
    expect(by.b.x).toBeGreaterThan(by.a.x)
  })

  it("layout 'none' is a no-op (manual placement preserved)", () => {
    const b = createBoardModel({ nodes: [node('a', 17, 23), node('b', 99, 5)] })
    b.applyLayout({ layout: 'none' })
    const by = Object.fromEntries(b.nodes().map((n) => [n.id, n.rect]))
    expect(by.a).toMatchObject({ x: 17, y: 23 })
    expect(by.b).toMatchObject({ x: 99, y: 5 })
  })

  it('routes follow the relaid-out node positions', () => {
    const b = createBoardModel({
      nodes: [node('a', 0, 0), node('b', 0, 0)],
      edges: [{ id: 'e1', source: { node: 'a', port: 'a:out' }, target: { node: 'b', port: 'b:in' }, kind: 'straight' }],
    })
    const before = b.routes()[0].path
    b.applyLayout({ layout: 'layered' })
    const after = b.routes()[0].path
    // Geometry moved, so the rendered edge path must change.
    expect(after).not.toBe(before)
  })
})

describe('board-model — pending edge (drag-from-port)', () => {
  it('begins, updates, and commits a pending edge', () => {
    const b = createBoardModel({ nodes: [node('a', 0, 0, 100, 60), node('b', 300, 0, 100, 60)] })
    b.beginEdge({ node: 'a', port: 'a:out' })
    expect(b.pendingEdge()).not.toBeNull()
    b.updatePendingEnd({ x: 250, y: 30 })
    expect(b.pendingEdge()!.end).toEqual({ x: 250, y: 30 })
    const edge = b.commitEdge({ node: 'b', port: 'b:in' })
    expect(edge).not.toBeNull()
    expect(b.pendingEdge()).toBeNull()
    expect(b.edges().some((e) => e.target.port === 'b:in')).toBe(true)
  })

  it('cancels a pending edge without creating one', () => {
    const b = createBoardModel({ nodes: [node('a', 0, 0)] })
    b.beginEdge({ node: 'a', port: 'a:out' })
    b.cancelEdge()
    expect(b.pendingEdge()).toBeNull()
    expect(b.edges()).toHaveLength(0)
  })

  it('refuses to commit a self-edge (same node)', () => {
    const b = createBoardModel({ nodes: [node('a', 0, 0)] })
    b.beginEdge({ node: 'a', port: 'a:out' })
    const e = b.commitEdge({ node: 'a', port: 'a:in' })
    expect(e).toBeNull()
    expect(b.edges()).toHaveLength(0)
  })
})

describe('board-model — content bounds + serialization', () => {
  it('contentBounds is the union AABB of all node rects', () => {
    const b = createBoardModel({
      nodes: [node('a', 0, 0, 100, 60), node('b', 300, 40, 100, 60)],
    })
    expect(b.contentBounds()).toEqual({ x: 0, y: 0, w: 400, h: 100 })
  })

  it('contentBounds is null with no nodes', () => {
    const b = createBoardModel({ nodes: [] })
    expect(b.contentBounds()).toBeNull()
  })

  it('serialize/snapshot round-trips through JSON', () => {
    const b = createBoardModel({
      nodes: [node('a', 0, 0)],
      edges: [{ id: 'e1', source: { node: 'a', port: 'a:out' }, target: { node: 'a', port: 'a:in' }, kind: 'bezier' }],
    })
    const json = b.serialize()
    const parsed = JSON.parse(json) as { nodes: Node[]; edges: Edge[] }
    expect(parsed.nodes).toHaveLength(1)
    expect(parsed.edges).toHaveLength(1)
  })
})
