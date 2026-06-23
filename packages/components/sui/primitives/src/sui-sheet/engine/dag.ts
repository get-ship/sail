// sui-sheet engine — the cell dependency DAG (pure graph algebra).
//
// The dependency graph over A1 cell keys: each formula cell records the cells it
// references (its PRECEDENTS); the reverse edge gives a cell's DEPENDENTS. The
// graph answers three questions the recalc path needs:
//   • precedents / dependents lookup (highlight + the eval read set),
//   • the dirty SUFFIX of a changed cell (the cell's transitive dependents in
//     TOPOLOGICAL order — a precedent before its dependent, each cell once),
//   • cycle detection (the back-edge participant set → the cells to mark #CYCLE!).
//
// PURE data structure — NO signals, NO eval, NO runtime imports. The reactive
// recalc wiring lives in `recalc.ts` + `create-sheet.ts`; this module is the
// graph in isolation (proven by `dag.test.ts`).
//
// Authority: build-spec §Features #2 (Cell DAG + recalc, dirty suffix), §"Auth…
// DAG API" (dependents/precedents/hasCycle built from parse().deps; topo recalc;
// a back-edge marks the participating cells #CYCLE!), §Slice 3.

export interface CellDag {
  /** Register a cell's formula precedent set (REPLACES any prior set). */
  setFormula(a1: string, precedents: readonly string[]): void
  /** Drop a cell's formula (removes its outgoing edges). */
  clearFormula(a1: string): void
  /** The cells this cell directly references. */
  precedents(a1: string): string[]
  /** The cells that directly reference this cell. */
  dependents(a1: string): string[]
  /** The cell's transitive dependents in topological order (excludes the cell
   *  itself; each dependent once; a precedent precedes its dependent). */
  dirtySuffix(a1: string): string[]
  /** Any cycle present in the current graph. */
  hasCycle(): boolean
  /** The set of cells participating in a cycle (to mark #CYCLE!). */
  cycleCells(): string[]
}

export function createDag(): CellDag {
  // precedents: cell → the cells it references. dependents: the reverse index.
  const precedents = new Map<string, Set<string>>()
  const dependents = new Map<string, Set<string>>()

  const ensure = (m: Map<string, Set<string>>, k: string): Set<string> => {
    let s = m.get(k)
    if (!s) {
      s = new Set()
      m.set(k, s)
    }
    return s
  }

  const clearFormula = (a1: string): void => {
    const prev = precedents.get(a1)
    if (prev) {
      for (const p of prev) dependents.get(p)?.delete(a1)
      precedents.delete(a1)
    }
  }

  const setFormula = (a1: string, deps: readonly string[]): void => {
    clearFormula(a1)
    if (deps.length === 0) return
    const set = ensure(precedents, a1)
    for (const p of deps) {
      set.add(p)
      ensure(dependents, p).add(a1)
    }
  }

  /** Walk transitive dependents of `a1` and return them in topological order.
   *  A post-order DFS reversed yields a valid topo order; we seed from each of
   *  `a1`'s direct dependents so `a1` itself is excluded. Cells inside a cycle
   *  are still emitted once (the recalc marks them #CYCLE!, it does not loop). */
  const dirtySuffix = (a1: string): string[] => {
    const order: string[] = []
    const visited = new Set<string>()
    const onStack = new Set<string>()

    const visit = (node: string): void => {
      visited.add(node)
      onStack.add(node)
      for (const dep of dependents.get(node) ?? []) {
        if (!visited.has(dep)) visit(dep)
        // a back-edge (dep on the current stack) is a cycle — do not recurse.
      }
      onStack.delete(node)
      order.push(node)
    }

    for (const dep of dependents.get(a1) ?? []) {
      if (!visited.has(dep)) visit(dep)
    }
    // `order` is post-order (dependents-deepest first); reverse → topo order
    // (a precedent before its dependent).
    return order.reverse()
  }

  /** Cycle detection: a DFS over the precedent edges; any back-edge (a node
   *  re-encountered while still on the recursion stack) marks every node on the
   *  stack from that node down as a cycle participant. */
  const computeCycleCells = (): Set<string> => {
    const cyclic = new Set<string>()
    const color = new Map<string, 0 | 1 | 2>() // 0=unseen 1=on-stack 2=done
    const stack: string[] = []

    const visit = (node: string): void => {
      color.set(node, 1)
      stack.push(node)
      for (const p of precedents.get(node) ?? []) {
        const c = color.get(p) ?? 0
        if (c === 0) visit(p)
        else if (c === 1) {
          // Back-edge to `p` — every node from `p` to the top of the stack
          // participates in the cycle.
          const from = stack.lastIndexOf(p)
          for (let i = from; i < stack.length; i++) cyclic.add(stack[i]!)
        }
      }
      stack.pop()
      color.set(node, 2)
    }

    for (const node of precedents.keys()) {
      if ((color.get(node) ?? 0) === 0) visit(node)
    }
    return cyclic
  }

  return {
    setFormula,
    clearFormula,
    precedents(a1: string): string[] {
      return [...(precedents.get(a1) ?? [])]
    },
    dependents(a1: string): string[] {
      return [...(dependents.get(a1) ?? [])]
    },
    dirtySuffix,
    hasCycle(): boolean {
      return computeCycleCells().size > 0
    },
    cycleCells(): string[] {
      return [...computeCycleCells()]
    },
  }
}
