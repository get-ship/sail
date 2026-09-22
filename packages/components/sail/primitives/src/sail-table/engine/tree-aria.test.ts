// TESTS-FIRST — Backbone #1, #240 engine follow-ups (Wave-2 sail-tree design).
//
// Two of the three #240 follow-ups the Wave-2 sail-tree design surfaced:
//
//   (1) BRANCH TRI-STATE on a regular (getSubRows) tree row — a branch row's
//       getIsSomeSelected() reflects PARTIAL leaf selection (some-but-not-all
//       leaf descendants selected); toggling a branch cascades select/deselect
//       to its leaf descendants. (Previously getIsSomeSelected was a `() => false`
//       stub on regular tree rows — real only on synthetic GROUP rows.)
//
//   (2) getPosInSet()/getSetSize() on Row<T> — the 1-based index of the row among
//       its siblings (its parent's subRows, or the root row list at depth 0) plus
//       the sibling count. Every tree/treegrid consumer needs these for
//       aria-posinset / aria-setsize.
//
// PURE @ship/sail-runtime engine — vitest-tested directly (no WASM).
//
// Authority: build-spec §Features #7 (selection cascade + tri-state), #13 (tree
// expansion), §"Authoritative Row API"; #240 follow-ups.

import { describe, expect, it } from 'vitest'
import { createTable } from './create-table'

interface Node {
  id: string
  label: string
  children?: Node[]
}

// Root
//  ├─ c1
//  │   ├─ g1
//  │   └─ g2
//  └─ c2
const TREE: Node[] = [
  {
    id: 'root',
    label: 'Root',
    children: [
      { id: 'c1', label: 'Child 1', children: [{ id: 'g1', label: 'GC1' }, { id: 'g2', label: 'GC2' }] },
      { id: 'c2', label: 'Child 2' },
    ],
  },
]
const COLS = [{ id: 'label', accessorKey: 'label' as const }]

function mkTree(opts: Record<string, unknown> = {}) {
  return createTable<Node>({
    data: TREE,
    columns: COLS,
    getRowId: (r) => r.id,
    getSubRows: (r) => r.children,
    enableExpanding: true,
    enableRowSelection: true,
    expanded: true,
    ...opts,
  })
}

// ── (1) BRANCH TRI-STATE on a regular tree row ────────────────────────────────

describe('tree branch tri-state — getIsSomeSelected reflects PARTIAL leaf selection', () => {
  it('a branch with only SOME leaf descendants selected is tri-state (some, not all)', () => {
    const t = mkTree()
    const c1 = t.getRow('c1')!
    // c1 leaf descendants: g1, g2. Select only g1.
    t.getRow('g1')!.toggleSelected(true)
    expect(c1.getIsSelected()).toBe(false) // not ALL leaves
    expect(c1.getIsSomeSelected()).toBe(true) // SOME leaves
  })

  it('a branch with ALL leaf descendants selected is fully selected, not tri-state', () => {
    const t = mkTree()
    const c1 = t.getRow('c1')!
    t.getRow('g1')!.toggleSelected(true)
    t.getRow('g2')!.toggleSelected(true)
    expect(c1.getIsSelected()).toBe(true)
    expect(c1.getIsSomeSelected()).toBe(false) // all → not partial
  })

  it('a branch with NO leaf descendants selected is neither selected nor partial', () => {
    const t = mkTree()
    const c1 = t.getRow('c1')!
    expect(c1.getIsSelected()).toBe(false)
    expect(c1.getIsSomeSelected()).toBe(false)
  })

  it('a deeper branch (root) is tri-state when a far descendant is selected', () => {
    const t = mkTree()
    t.getRow('g1')!.toggleSelected(true)
    const root = t.getRow('root')!
    expect(root.getIsSelected()).toBe(false) // c2, g2 unselected
    expect(root.getIsSomeSelected()).toBe(true)
  })

  it('a LEAF row (no subRows) is never tri-state', () => {
    const t = mkTree()
    const g1 = t.getRow('g1')!
    g1.toggleSelected(true)
    expect(g1.getIsSelected()).toBe(true)
    expect(g1.getIsSomeSelected()).toBe(false)
  })
})

describe('tree branch cascade — toggling a branch select/deselects its leaf descendants', () => {
  it('toggling a branch ON selects every leaf descendant', () => {
    const t = mkTree()
    const c1 = t.getRow('c1')!
    c1.toggleSelected(true)
    expect(t.getRow('g1')!.getIsSelected()).toBe(true)
    expect(t.getRow('g2')!.getIsSelected()).toBe(true)
    expect(c1.getIsSelected()).toBe(true)
  })

  it('toggling a branch OFF deselects every leaf descendant', () => {
    const t = mkTree()
    const c1 = t.getRow('c1')!
    c1.toggleSelected(true)
    c1.toggleSelected(false)
    expect(t.getRow('g1')!.getIsSelected()).toBe(false)
    expect(t.getRow('g2')!.getIsSelected()).toBe(false)
    expect(c1.getIsSomeSelected()).toBe(false)
  })

  it('toggling the root cascades to ALL leaf descendants (g1, g2, c2)', () => {
    const t = mkTree()
    t.getRow('root')!.toggleSelected(true)
    for (const id of ['g1', 'g2', 'c2']) {
      expect(t.getRow(id)!.getIsSelected()).toBe(true)
    }
    expect(t.getRow('root')!.getIsSelected()).toBe(true)
  })

  it('a no-arg toggle on a fully-selected branch deselects it (cascade)', () => {
    const t = mkTree()
    const c1 = t.getRow('c1')!
    c1.toggleSelected(true)
    c1.toggleSelected() // no arg → flip; fully-selected → off
    expect(t.getRow('g1')!.getIsSelected()).toBe(false)
    expect(t.getRow('g2')!.getIsSelected()).toBe(false)
  })

  it('a disabled leaf is skipped by the branch cascade (enableRowSelection)', () => {
    const t = mkTree({ enableRowSelection: (row: { original: Node }) => row.original.id !== 'g2' })
    const c1 = t.getRow('c1')!
    c1.toggleSelected(true)
    expect(t.getRow('g1')!.getIsSelected()).toBe(true)
    expect(t.getRow('g2')!.getIsSelected()).toBe(false) // disabled
    // c1 counts only SELECTABLE leaves → g1 selected = all selectable → fully selected.
    expect(c1.getIsSelected()).toBe(true)
    expect(c1.getIsSomeSelected()).toBe(false)
  })
})

// ── (2) getPosInSet() / getSetSize() ──────────────────────────────────────────

describe('getPosInSet / getSetSize — 1-based sibling index + sibling count', () => {
  it('a depth-0 row reports its position among the root rows', () => {
    const flat = createTable<Node>({
      data: [
        { id: 'a', label: 'A' },
        { id: 'b', label: 'B' },
        { id: 'c', label: 'C' },
      ],
      columns: COLS,
      getRowId: (r) => r.id,
    })
    expect(flat.getRow('a')!.getPosInSet()).toBe(1)
    expect(flat.getRow('b')!.getPosInSet()).toBe(2)
    expect(flat.getRow('c')!.getPosInSet()).toBe(3)
    expect(flat.getRow('a')!.getSetSize()).toBe(3)
    expect(flat.getRow('c')!.getSetSize()).toBe(3)
  })

  it('the sole root tree row is posinset 1 of setsize 1', () => {
    const t = mkTree()
    expect(t.getRow('root')!.getPosInSet()).toBe(1)
    expect(t.getRow('root')!.getSetSize()).toBe(1)
  })

  it("a child reports its position among its PARENT's subRows", () => {
    const t = mkTree()
    expect(t.getRow('c1')!.getPosInSet()).toBe(1)
    expect(t.getRow('c2')!.getPosInSet()).toBe(2)
    expect(t.getRow('c1')!.getSetSize()).toBe(2) // root has 2 children
    expect(t.getRow('c2')!.getSetSize()).toBe(2)
  })

  it('grandchildren report their position among their parent (c1) subRows', () => {
    const t = mkTree()
    expect(t.getRow('g1')!.getPosInSet()).toBe(1)
    expect(t.getRow('g2')!.getPosInSet()).toBe(2)
    expect(t.getRow('g1')!.getSetSize()).toBe(2)
    expect(t.getRow('g2')!.getSetSize()).toBe(2)
  })

  it('a synthetic GROUP row reports its position among its sibling groups', () => {
    interface Person {
      id: string
      region: string
      n: number
    }
    const t = createTable<Person>({
      data: [
        { id: '1', region: 'east', n: 1 },
        { id: '2', region: 'west', n: 2 },
        { id: '3', region: 'south', n: 3 },
      ],
      columns: [
        { id: 'region', accessorKey: 'region' },
        { id: 'n', accessorKey: 'n' },
      ],
      getRowId: (r) => r.id,
      enableGrouping: true,
      grouping: ['region'],
      expanded: true,
    })
    const groups = t.rows().filter((r) => r.getIsGrouped())
    expect(groups.length).toBe(3)
    expect(groups.map((g) => g.getPosInSet())).toEqual([1, 2, 3])
    expect(groups.every((g) => g.getSetSize() === 3)).toBe(true)
  })
})
