// TESTS-FIRST — Backbone #1, Slice 2: row selection (Feature #7).
//
// Status: RED. `./selection` and the Slice-2 `Row.getIsSelected`/`getIsSomeSelected`
// additions do not exist yet (engine is greenfield). The imports below fail to
// resolve until Slice 2 ships — that import-not-found is the expected red for this
// contract. DO NOT add the impl here. DO NOT weaken these into passing stubs.
//
// Authority: build-spec §Features #7 ("Row selection"), §"Signal graph"
// (rowSelection +anchor, include/exclude mode → selectedRowIds / getIsAll*Selected),
// §API sketch (`toggleRowSelected(id, v?, opts?: { range?, additive? })`,
// `toggleAllRowsSelected`, `enableRowSelection` per-row predicate, `Row.getIsSelected`/
// `getIsSomeSelected`/`getCanSelect`), Slice 2 ("toggleRowSelected({range}) shift-span
// over current displayed order with a selection anchor; include/exclude select-all
// model … per-row enableRowSelection predicate; group-row cascade + tri-state").
//
// Engine purity (Cardinal 4): imports ONLY @ship/sail-runtime. The purity GREP-test is
// Slice 4's contract, not this file's.

import { describe, expect, it } from 'vitest'
import { signal } from '@ship/sail-runtime'
// RED: greenfield modules — resolve only once Slice 2 ships.
import { createTable } from './create-table'

interface Person {
  id: string
  name: string
  count: number
}

const COLUMNS = [
  { id: 'name', accessorKey: 'name' as const },
  { id: 'count', accessorKey: 'count' as const },
]

const PEOPLE: Person[] = [
  { id: 'a', name: 'Bob', count: 10 },
  { id: 'b', name: 'Alice', count: 2 },
  { id: 'c', name: 'Carol', count: 100 },
  { id: 'd', name: 'Dave', count: 50 },
  { id: 'e', name: 'Eve', count: 7 },
]

function table(data: Person[] = PEOPLE) {
  return createTable<Person>({ data, columns: COLUMNS, getRowId: (r) => r.id })
}

describe('toggleRowSelected — single + include-mode set', () => {
  it('toggles one row and surfaces it in selectedRowIds', () => {
    const t = table()
    t.toggleRowSelected('b')
    expect(t.selectedRowIds()).toEqual(['b'])
    expect(t.getRow('b')!.getIsSelected()).toBe(true)
    expect(t.getRow('a')!.getIsSelected()).toBe(false)
  })

  it('a second toggle on the same row deselects it', () => {
    const t = table()
    t.toggleRowSelected('b')
    t.toggleRowSelected('b')
    expect(t.selectedRowIds()).toEqual([])
    expect(t.getRow('b')!.getIsSelected()).toBe(false)
  })

  it('explicit value forces the state regardless of prior', () => {
    const t = table()
    t.toggleRowSelected('b', true)
    t.toggleRowSelected('b', true)
    expect(t.getRow('b')!.getIsSelected()).toBe(true)
    t.toggleRowSelected('b', false)
    expect(t.getRow('b')!.getIsSelected()).toBe(false)
  })

  it('selection state is { mode:"include", ids:Set } by default', () => {
    const t = table()
    t.toggleRowSelected('b')
    const sel = t.state.rowSelection()
    expect(sel.mode).toBe('include')
    expect(sel.ids instanceof Set).toBe(true)
    expect([...sel.ids]).toEqual(['b'])
  })
})

describe('toggleRowSelected({range}) — shift-span over DISPLAYED order', () => {
  it('selects the contiguous span from the anchor in displayed (post-sort) order', () => {
    const t = table()
    // Sort by count asc → displayed order: e(7), b(2)?? actually 2,7,10,50,100 → b,e,a,d,c
    t.toggleSort('count') // asc: b(2), e(7), a(10), d(50), c(100)
    t.toggleRowSelected('b') // anchor = b (displayed index 0)
    t.toggleRowSelected('d', undefined, { range: true }) // span b..d over displayed order
    expect(new Set(t.selectedRowIds())).toEqual(new Set(['b', 'e', 'a', 'd']))
  })

  it('the anchor is the LAST plain toggle, not the first selection ever', () => {
    const t = table()
    // Default displayed order: a,b,c,d,e
    t.toggleRowSelected('a')
    t.toggleRowSelected('c') // anchor moves to c
    t.toggleRowSelected('e', undefined, { range: true }) // span c..e
    expect(new Set(t.selectedRowIds())).toEqual(new Set(['a', 'c', 'd', 'e']))
  })

  it('range works upward (anchor below the target in displayed order)', () => {
    const t = table()
    t.toggleRowSelected('d') // anchor = d (index 3)
    t.toggleRowSelected('b', undefined, { range: true }) // span b..d
    expect(new Set(t.selectedRowIds())).toEqual(new Set(['b', 'c', 'd']))
  })
})

describe('toggleAllRowsSelected — include/exclude representation', () => {
  it('selecting all flips to mode:"exclude" with an empty set (no id enumeration)', () => {
    const t = table()
    t.toggleAllRowsSelected(true)
    const sel = t.state.rowSelection()
    expect(sel.mode).toBe('exclude')
    expect([...sel.ids]).toEqual([])
    expect(t.getIsAllRowsSelected()).toBe(true)
  })

  it('exclude-mode "all minus N" is expressible by toggling rows OFF without enumerating ids', () => {
    const t = table()
    t.toggleAllRowsSelected(true) // exclude, {}
    t.toggleRowSelected('c', false) // exclude c
    const sel = t.state.rowSelection()
    expect(sel.mode).toBe('exclude')
    expect([...sel.ids]).toEqual(['c'])
    // c is the only de-selected row; everything else is selected.
    expect(t.getRow('c')!.getIsSelected()).toBe(false)
    expect(t.getRow('a')!.getIsSelected()).toBe(true)
    expect(t.getIsAllRowsSelected()).toBe(false)
  })

  it('toggleAllRowsSelected(false) clears back to include + empty', () => {
    const t = table()
    t.toggleAllRowsSelected(true)
    t.toggleAllRowsSelected(false)
    const sel = t.state.rowSelection()
    expect(sel.mode).toBe('include')
    expect([...sel.ids]).toEqual([])
    expect(t.getIsAllRowsSelected()).toBe(false)
  })
})

describe('enableRowSelection — per-row predicate (disabled rows)', () => {
  it('a row failing the predicate cannot be selected and reports getCanSelect()===false', () => {
    const t = createTable<Person>({
      data: PEOPLE,
      columns: COLUMNS,
      getRowId: (r) => r.id,
      enableRowSelection: (row) => row.original.count >= 10, // b(2), e(7) disabled
    })
    expect(t.getRow('b')!.getCanSelect()).toBe(false)
    t.toggleRowSelected('b')
    expect(t.getRow('b')!.getIsSelected()).toBe(false)
    expect(t.selectedRowIds()).toEqual([])
  })

  it('a disabled row is excluded from a range span', () => {
    const t = createTable<Person>({
      data: PEOPLE,
      columns: COLUMNS,
      getRowId: (r) => r.id,
      enableRowSelection: (row) => row.original.id !== 'c', // c disabled
    })
    t.toggleRowSelected('a') // anchor (index 0)
    t.toggleRowSelected('d', undefined, { range: true }) // span a..d, c skipped
    expect(new Set(t.selectedRowIds())).toEqual(new Set(['a', 'b', 'd']))
  })
})

describe('group-row cascade + tri-state', () => {
  it('selecting a group row cascades to its leaf children', () => {
    const t = createTable<Person>({
      data: PEOPLE,
      columns: COLUMNS,
      getRowId: (r) => r.id,
      enableGrouping: true,
      initialState: { grouping: ['count'], expanded: true },
    })
    const group = t.rows().find((r) => r.getIsGrouped())!
    group.toggleSelected(true)
    for (const leaf of group.getLeafRows()) {
      expect(leaf.getIsSelected()).toBe(true)
    }
  })

  it('getIsSomeSelected() is tri-state on a partially-selected group', () => {
    const t = createTable<Person>({
      data: [
        { id: 'a', name: 'Bob', count: 1 },
        { id: 'b', name: 'Alice', count: 1 },
      ],
      columns: COLUMNS,
      getRowId: (r) => r.id,
      enableGrouping: true,
      initialState: { grouping: ['count'], expanded: true },
    })
    const group = t.rows().find((r) => r.getIsGrouped())!
    const [first] = group.getLeafRows()
    first!.toggleSelected(true) // only one leaf selected
    expect(group.getIsSelected()).toBe(false)
    expect(group.getIsSomeSelected()).toBe(true)
  })
})

describe('selection survives reorder/repage (keyed by getRowId)', () => {
  it('a selected id stays selected across a sort that moves it', () => {
    const t = table()
    t.toggleRowSelected('c')
    expect(t.getRow('c')!.getIsSelected()).toBe(true)
    t.toggleSort('count') // reorders the displayed rows
    expect(t.getRow('c')!.getIsSelected()).toBe(true)
    expect(t.selectedRowIds()).toEqual(['c'])
  })

  it('a selected id survives a data identity change that keeps the id', () => {
    const data = signal<Person[]>([...PEOPLE])
    const t = createTable<Person>({ data, columns: COLUMNS, getRowId: (r) => r.id })
    t.toggleRowSelected('d')
    // Re-emit the same rows in a different order (repage/refetch).
    data.set([PEOPLE[4]!, PEOPLE[3]!, PEOPLE[2]!, PEOPLE[1]!, PEOPLE[0]!])
    expect(t.getRow('d')!.getIsSelected()).toBe(true)
    expect(t.selectedRowIds()).toEqual(['d'])
  })
})
