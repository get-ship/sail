// sui-board — the SOLE dagre importer ("one contract per boundary"). A pure
// graph-layout adapter: (nodes, edges, options) → new nodes with repositioned
// rects in world coords. Deterministic — no signals, no DOM, no randomness.
//
// `layered` runs the dagre Sugiyama pipeline (rank → order → place) and
// projects dagre's CENTER coords back onto top-left node rects. `force` runs a
// minimal seeded force-directed sim (fixed iteration budget, layout seeded from
// node index, so the result is reproducible). `none` is identity.
//
// The board model binds to THIS adapter's `layoutBoard`, never to the dagre
// core directly. The adapter owns geometry mapping; dagre owns ranking.

import dagre from '@dagrejs/dagre'
import type { Node, Edge } from './board-model'

/** Layered rank direction: left→right, right→left, top→bottom, bottom→top.
 *  Matches @dagrejs/dagre's `GraphLabel['rankdir']` union. */
export type RankDir = 'LR' | 'RL' | 'TB' | 'BT'

export type LayoutKind = 'none' | 'layered' | 'force'

export interface LayoutOptions {
  layout: LayoutKind
  /** Layered rank direction (default 'LR' — left→right). */
  direction?: RankDir
  /** Gap between adjacent nodes within a rank (world units, default 50). */
  nodesep?: number
  /** Gap between adjacent ranks (world units, default 120). */
  ranksep?: number
}

/** Replace a node's rect x/y while preserving everything else (incl. w/h). */
function moveTo(node: Node, x: number, y: number): Node {
  return { ...node, rect: { ...node.rect, x, y } }
}

/** Layered (Sugiyama) layout through the @dagrejs/dagre core. Builds a graphlib
 *  Graph, runs dagre.layout() (which mutates node labels with center x/y in
 *  place), then projects those CENTER coords back onto top-left node rects. */
function layered(nodes: Node[], edges: Edge[], opts: LayoutOptions): Node[] {
  const g = new dagre.graphlib.Graph()
  g.setGraph({
    rankdir: opts.direction ?? 'LR',
    nodesep: opts.nodesep ?? 50,
    ranksep: opts.ranksep ?? 120,
  })
  // Edges carry no label here; dagre still requires a default so setEdge works.
  g.setDefaultEdgeLabel(() => ({}))
  for (const n of nodes) g.setNode(n.id, { width: n.rect.w, height: n.rect.h })
  for (const e of edges) {
    if (g.hasNode(e.source.node) && g.hasNode(e.target.node)) {
      g.setEdge(e.source.node, e.target.node)
    }
  }
  dagre.layout(g)
  return nodes.map((n) => {
    const p = g.node(n.id)
    if (!p || p.x === undefined || p.y === undefined) return n
    // dagre returns CENTERS; the board rect is top-left.
    return moveTo(n, p.x - n.rect.w / 2, p.y - n.rect.h / 2)
  })
}

/** A minimal deterministic force-directed sim: Fruchterman–Reingold-style
 *  repulsion between every pair + spring attraction along edges. The initial
 *  layout is seeded from node index on a circle (no randomness), and the sim
 *  runs a fixed number of cooling iterations, so the output is reproducible. */
function force(nodes: Node[], edges: Edge[]): Node[] {
  const n = nodes.length
  if (n === 0) return nodes
  const idx = new Map(nodes.map((node, i) => [node.id, i]))
  const k = 180 // ideal edge length (world units)
  const area = k * k
  // Seed on a circle by index — deterministic, no overlap at start.
  const px: number[] = new Array(n)
  const py: number[] = new Array(n)
  const radius = n > 1 ? (k * n) / 2 : 0
  for (let i = 0; i < n; i++) {
    const a = (2 * Math.PI * i) / n
    px[i] = Math.cos(a) * radius
    py[i] = Math.sin(a) * radius
  }
  const dxv: number[] = new Array(n)
  const dyv: number[] = new Array(n)
  const at = (arr: number[], i: number): number => arr[i] ?? 0
  const ITER = 120
  for (let it = 0; it < ITER; it++) {
    for (let i = 0; i < n; i++) {
      dxv[i] = 0
      dyv[i] = 0
    }
    // Repulsion: every unordered pair.
    for (let i = 0; i < n; i++) {
      for (let j = i + 1; j < n; j++) {
        let dx = at(px, i) - at(px, j)
        let dy = at(py, i) - at(py, j)
        let d2 = dx * dx + dy * dy
        if (d2 === 0) {
          // Separate coincident nodes deterministically by index.
          dx = (i - j) * 0.01 + 0.01
          dy = (i + j) * 0.01 + 0.01
          d2 = dx * dx + dy * dy
        }
        const d = Math.sqrt(d2)
        const ux = (dx / d) * (area / d)
        const uy = (dy / d) * (area / d)
        dxv[i] = at(dxv, i) + ux
        dyv[i] = at(dyv, i) + uy
        dxv[j] = at(dxv, j) - ux
        dyv[j] = at(dyv, j) - uy
      }
    }
    // Attraction: along each edge.
    for (const e of edges) {
      const a = idx.get(e.source.node)
      const b = idx.get(e.target.node)
      if (a === undefined || b === undefined || a === b) continue
      const dx = at(px, a) - at(px, b)
      const dy = at(py, a) - at(py, b)
      const d = Math.hypot(dx, dy) || 1e-6
      const attr = (d * d) / k
      const ux = (dx / d) * attr
      const uy = (dy / d) * attr
      dxv[a] = at(dxv, a) - ux
      dyv[a] = at(dyv, a) - uy
      dxv[b] = at(dxv, b) + ux
      dyv[b] = at(dyv, b) + uy
    }
    // Cool: cap step length, shrinking with iteration.
    const temp = k * (1 - it / ITER)
    for (let i = 0; i < n; i++) {
      const d = Math.hypot(at(dxv, i), at(dyv, i)) || 1e-6
      const step = Math.min(d, temp)
      px[i] = at(px, i) + (at(dxv, i) / d) * step
      py[i] = at(py, i) + (at(dyv, i) / d) * step
    }
  }
  // px/py are CENTERS; round for stable coords, then map to top-left rects.
  return nodes.map((node, i) =>
    moveTo(node, Math.round(at(px, i) - node.rect.w / 2), Math.round(at(py, i) - node.rect.h / 2)),
  )
}

/** Lay out the board graph. Pure + deterministic. */
export function layoutBoard(nodes: Node[], edges: Edge[], opts: LayoutOptions): Node[] {
  switch (opts.layout) {
    case 'layered':
      return layered(nodes, edges, opts)
    case 'force':
      return force(nodes, edges)
    case 'none':
      return nodes
  }
}
