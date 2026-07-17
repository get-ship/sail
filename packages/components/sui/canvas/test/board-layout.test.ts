// sui-board — auto-layout adapter behavioral specs (Slice 7).
//
// Pure: nodes + edges in → positioned nodes out, deterministic. The adapter
// maps the board graph through the vendored dagre layered core (Sugiyama) or a
// minimal seeded force sim, returning new node rects in world coords. No
// signals, no DOM — a function of (nodes, edges, options).
import { describe, expect, it } from 'vitest'
import { signal, effect } from '@ship/sail-runtime'
import { layoutBoard, type LayoutKind } from '../src/sui-board/engine/layout-adapter'
import { createBoardModel } from '../src/sui-board/engine/board-model'
import type { Node, Edge } from '../src/sui-board/engine/board-model'

function node(id: string, x = 0, y = 0, w = 100, h = 60): Node {
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

function edge(id: string, from: string, to: string): Edge {
  return { id, source: { node: from, port: `${from}:out` }, target: { node: to, port: `${to}:in` }, kind: 'bezier' }
}

describe('layout-adapter — layered (dagre)', () => {
  it('returns one positioned node per input node, ids preserved', () => {
    const nodes = [node('a'), node('b'), node('c')]
    const edges = [edge('e1', 'a', 'b'), edge('e2', 'b', 'c')]
    const out = layoutBoard(nodes, edges, { layout: 'layered' })
    expect(out.map((n) => n.id).sort()).toEqual(['a', 'b', 'c'])
  })

  it('preserves each node size (w/h) — only x/y change', () => {
    const nodes = [node('a', 0, 0, 120, 80), node('b', 0, 0, 120, 80)]
    const out = layoutBoard(nodes, [edge('e1', 'a', 'b')], { layout: 'layered' })
    for (const n of out) {
      expect(n.rect.w).toBe(120)
      expect(n.rect.h).toBe(80)
    }
  })

  it('ranks a chain left→right: each successor sits in a later rank (greater x)', () => {
    const nodes = [node('a'), node('b'), node('c')]
    const edges = [edge('e1', 'a', 'b'), edge('e2', 'b', 'c')]
    const out = layoutBoard(nodes, edges, { layout: 'layered' })
    const by = Object.fromEntries(out.map((n) => [n.id, n.rect]))
    expect(by.b.x).toBeGreaterThan(by.a.x)
    expect(by.c.x).toBeGreaterThan(by.b.x)
  })

  it('places sibling nodes (same rank) at the same x, different y', () => {
    // a → b, a → c : b and c share rank 1
    const nodes = [node('a'), node('b'), node('c')]
    const edges = [edge('e1', 'a', 'b'), edge('e2', 'a', 'c')]
    const out = layoutBoard(nodes, edges, { layout: 'layered' })
    const by = Object.fromEntries(out.map((n) => [n.id, n.rect]))
    expect(by.b.x).toBe(by.c.x)
    expect(by.b.y).not.toBe(by.c.y)
  })

  it('is deterministic — identical input yields identical output', () => {
    const mk = () => [node('a'), node('b'), node('c'), node('d')]
    const edges = [edge('e1', 'a', 'b'), edge('e2', 'a', 'c'), edge('e3', 'b', 'd'), edge('e4', 'c', 'd')]
    const first = layoutBoard(mk(), edges, { layout: 'layered' })
    const second = layoutBoard(mk(), edges, { layout: 'layered' })
    expect(second).toEqual(first)
  })

  it('lays out disconnected nodes without throwing (no incident edges)', () => {
    const out = layoutBoard([node('a'), node('b')], [], { layout: 'layered' })
    expect(out).toHaveLength(2)
    expect(out.every((n) => Number.isFinite(n.rect.x) && Number.isFinite(n.rect.y))).toBe(true)
  })

  it('respects vertical direction (TB) — successors sit below (greater y), same x', () => {
    const nodes = [node('a'), node('b')]
    const out = layoutBoard(nodes, [edge('e1', 'a', 'b')], { layout: 'layered', direction: 'TB' })
    const by = Object.fromEntries(out.map((n) => [n.id, n.rect]))
    expect(by.b.y).toBeGreaterThan(by.a.y)
    expect(by.b.x).toBe(by.a.x)
  })
})

describe('layout-adapter — force (seeded sim)', () => {
  it('returns one positioned node per input, finite coords, deterministic', () => {
    const nodes = [node('a'), node('b'), node('c')]
    const edges = [edge('e1', 'a', 'b'), edge('e2', 'b', 'c')]
    const first = layoutBoard(nodes, edges, { layout: 'force' })
    const second = layoutBoard(nodes, edges, { layout: 'force' })
    expect(first).toHaveLength(3)
    expect(first.every((n) => Number.isFinite(n.rect.x) && Number.isFinite(n.rect.y))).toBe(true)
    expect(second).toEqual(first)
  })

  it('pushes unconnected nodes apart (no two nodes share a position)', () => {
    const nodes = [node('a'), node('b'), node('c')]
    const out = layoutBoard(nodes, [], { layout: 'force' })
    const keys = new Set(out.map((n) => `${Math.round(n.rect.x)},${Math.round(n.rect.y)}`))
    expect(keys.size).toBe(3)
  })

  it('keeps connected nodes closer than unconnected ones', () => {
    // a—b connected, c isolated. After the sim a/b should be nearer each other
    // than either is to c (attraction along the edge vs pure repulsion).
    const nodes = [node('a'), node('b'), node('c')]
    const out = layoutBoard(nodes, [edge('e1', 'a', 'b')], { layout: 'force' })
    const p = Object.fromEntries(out.map((n) => [n.id, { x: n.rect.x + n.rect.w / 2, y: n.rect.y + n.rect.h / 2 }]))
    const dist = (u: { x: number; y: number }, v: { x: number; y: number }) => Math.hypot(u.x - v.x, u.y - v.y)
    expect(dist(p.a, p.b)).toBeLessThan(dist(p.a, p.c))
  })
})

describe('layout-adapter — none', () => {
  it("returns the input rects unchanged for layout 'none'", () => {
    const nodes = [node('a', 17, 23), node('b', 99, 5)]
    const out = layoutBoard(nodes, [], { layout: 'none' })
    expect(out.map((n) => n.rect)).toEqual([{ x: 17, y: 23, w: 100, h: 60 }, { x: 99, y: 5, w: 100, h: 60 }])
  })
})

// Mirrors the controller's layout effect (sui-board.controller mount): a
// `layout` prop signal → board.applyLayout when it changes to a real algorithm.
// Documents that setting the prop repositions the board (the lie that was: the
// prop declared none|layered|force with ZERO implementation).
describe('layout prop drives reposition (controller effect)', () => {
  it('changing layout from none→layered repositions overlapping nodes', () => {
    const board = createBoardModel({
      nodes: [node('a', 0, 0), node('b', 0, 0)],
      edges: [edge('e1', 'a', 'b')],
    })
    const layout = signal<LayoutKind>('none')
    let last: LayoutKind | null = null
    const stop = effect(() => {
      const kind = layout()
      if (kind === last) return
      last = kind
      if (kind === 'none') return
      board.applyLayout({ layout: kind })
    })
    // none: untouched (both at origin).
    expect(board.nodes().find((n) => n.id === 'b')!.rect.x).toBe(0)
    // flip the prop → relayout.
    layout.set('layered')
    const by = Object.fromEntries(board.nodes().map((n) => [n.id, n.rect]))
    expect(by.b.x).toBeGreaterThan(by.a.x)
    stop()
  })

  it('switching layered→force re-lays-out again (each distinct value fires once)', () => {
    const board = createBoardModel({
      nodes: [node('a', 0, 0), node('b', 0, 0), node('c', 0, 0)],
      edges: [edge('e1', 'a', 'b')],
    })
    const layout = signal<LayoutKind>('layered')
    let last: LayoutKind | null = null
    let runs = 0
    const stop = effect(() => {
      const kind = layout()
      if (kind === last) return
      last = kind
      if (kind === 'none') return
      runs++
      board.applyLayout({ layout: kind })
    })
    expect(runs).toBe(1) // layered ran on mount
    layout.set('layered') // same value → no extra run
    expect(runs).toBe(1)
    layout.set('force') // distinct → re-layout
    expect(runs).toBe(2)
    const keys = new Set(board.nodes().map((n) => `${Math.round(n.rect.x)},${Math.round(n.rect.y)}`))
    expect(keys.size).toBe(3) // force separated the three coincident nodes
    stop()
  })
})
