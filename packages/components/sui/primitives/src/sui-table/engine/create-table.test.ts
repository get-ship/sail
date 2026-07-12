// TESTS-FIRST — Backbone #1, Slice 1b: the createTable() engine assembler.
//
// Status: RED. `./create-table` does not exist yet (engine is greenfield); the
// import is import-not-found until Slice 1b ships the assembler + pipeline +
// minimal Row factory. DO NOT add the impl here. DO NOT weaken these into
// passing stubs.
//
// Authority: build-spec §"Backbone #1 — createTable() engine", §"API sketch"
// (`TableOptions`/`Table<T>`), §"getRowId is REQUIRED", §"Signal graph"
// ("touching one slice recomputes only the *suffix*"), §Features #2 (multi-sort,
// getSortIndex), Slice 1b ("row model wraps originals with the required getRowId;
// state.sorting is SortState[] and getSortIndex() reports priority; the derived
// chain recomputes only its suffix — proven here with a recompute counter").
//
// Engine purity (Cardinal 4): imports ONLY @ship/sui-runtime. No ship.*, no fetch,
// no <data>. The purity GREP-test is Slice 4's contract, not this file's.

import { describe, expect, it, vi } from 'vitest'
import { signal, derived } from '@ship/sui-runtime'
// RED: greenfield module.
import { createTable } from './create-table'

interface Person {
  id: string
  name: string
  count: number // numeric column that triggers the '10' before '2' bug
}

const COLUMNS = [
  { id: 'name', accessorKey: 'name' as const },
  { id: 'count', accessorKey: 'count' as const },
]

const PEOPLE: Person[] = [
  { id: 'a', name: 'Bob', count: 10 },
  { id: 'b', name: 'Alice', count: 2 },
  { id: 'c', name: 'Carol', count: 100 },
]

describe('createTable — row model', () => {
  it('wraps originals into Row<T> keyed by the REQUIRED getRowId', () => {
    const table = createTable<Person>({
      data: PEOPLE,
      columns: COLUMNS,
      getRowId: (r) => r.id,
    })
    const rows = table.rows()
    expect(rows.map((r) => r.id)).toEqual(['a', 'b', 'c'])
    expect(rows[0]!.original).toEqual(PEOPLE[0])
    expect(rows[0]!.index).toBe(0)
  })

  it('Row.getValue(columnId) reads through the accessor', () => {
    const table = createTable<Person>({
      data: PEOPLE,
      columns: COLUMNS,
      getRowId: (r) => r.id,
    })
    expect(table.getRow('b')!.getValue('name')).toBe('Alice')
    expect(table.getRow('b')!.getValue('count')).toBe(2)
  })

  it('lifts a raw array AND tracks a live data signal identity', () => {
    const data = signal<Person[]>([PEOPLE[0]!])
    const table = createTable<Person>({
      data,
      columns: COLUMNS,
      getRowId: (r) => r.id,
    })
    expect(table.rows()).toHaveLength(1)
    // Growing the data signal adds rows WITHOUT remount (behavioral spec #1).
    data.set([PEOPLE[0]!, PEOPLE[1]!])
    expect(table.rows()).toHaveLength(2)
  })
})

describe('createTable — sorting state', () => {
  it('starts with empty sorting (SortState[])', () => {
    const table = createTable<Person>({
      data: PEOPLE,
      columns: COLUMNS,
      getRowId: (r) => r.id,
    })
    expect(table.state.sorting()).toEqual([])
  })

  it('numeric column sorts 2, 10, 100 — not lexicographic (the bug fix)', () => {
    const table = createTable<Person>({
      data: PEOPLE,
      columns: COLUMNS,
      getRowId: (r) => r.id,
    })
    table.toggleSort('count') // none → asc
    expect(table.sortedRows().map((r) => r.getValue('count'))).toEqual([2, 10, 100])
  })

  it('toggleSort cycles none → asc → desc → none', () => {
    const table = createTable<Person>({
      data: PEOPLE,
      columns: COLUMNS,
      getRowId: (r) => r.id,
    })
    table.toggleSort('count')
    expect(table.state.sorting()[0]).toMatchObject({ id: 'count', desc: false })
    table.toggleSort('count')
    expect(table.state.sorting()[0]).toMatchObject({ id: 'count', desc: true })
    table.toggleSort('count')
    expect(table.state.sorting()).toEqual([])
  })

  it('multi-sort appends and getSortIndex() reports priority', () => {
    const table = createTable<Person>({
      data: PEOPLE,
      columns: COLUMNS,
      getRowId: (r) => r.id,
    })
    table.toggleSort('name')
    table.toggleSort('count', { multi: true })
    expect(table.getColumn('name')!.getSortIndex()).toBe(0)
    expect(table.getColumn('count')!.getSortIndex()).toBe(1)
  })
})

describe('createTable — suffix-only recompute (Signal graph contract)', () => {
  it('a state.sorting write recomputes sortedRows; an unrelated no-op write does not', () => {
    const noise = signal(0) // an existing slice unrelated to the row pipeline
    const table = createTable<Person>({
      data: PEOPLE,
      columns: COLUMNS,
      getRowId: (r) => r.id,
    })

    const recompute = vi.fn()
    // A derived reading BOTH sortedRows and the unrelated `noise` signal.
    const probe = derived(() => {
      recompute()
      noise() // track the unrelated slice too
      return table.sortedRows().length
    })
    probe() // seed
    expect(recompute).toHaveBeenCalledTimes(1)

    // Writing the sorting slice MUST recompute the suffix.
    table.toggleSort('count')
    probe()
    expect(recompute).toHaveBeenCalledTimes(2)

    // A no-op write to the unrelated slice (same value) must NOT recompute.
    noise.set(0)
    probe()
    expect(recompute).toHaveBeenCalledTimes(2)
  })
})
