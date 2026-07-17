// TESTS-FIRST — Backbone #1, Slice 4: cell editing model.
//
// Status: RED. `./create-table` (which wires `./editing`) does not exist yet;
// import-not-found is the expected red. DO NOT add the impl here.
//
// Authority: build-spec §Features #15 ("Cell editing": per-column enableEditing;
// editing state slice = { rowId, columnId } | null; startEditing/commitEdit(value)/
// cancelEdit; an optional validation hook on the column; this is the REAL model
// backing the sail-cell-edit event — the event is NOT shipped without it),
// §"Cell<T>" API (getIsEditing), Slice 4 ("editing model start/commit/cancel +
// validation; nav suspended during edit").

import { describe, expect, it, vi } from 'vitest'
// RED: greenfield module.
import { createTable } from './create-table'

interface Row {
  id: string
  name: string
  age: number
}

const COLUMNS = [
  { id: 'name', accessorKey: 'name' as const }, // NOT editable
  { id: 'age', accessorKey: 'age' as const, enableEditing: true },
]
const ROWS: Row[] = [
  { id: 'a', name: 'Alpha', age: 1 },
  { id: 'b', name: 'Bravo', age: 2 },
]

function mk(opts: Record<string, unknown> = {}) {
  return createTable<Row>({
    data: ROWS,
    columns: COLUMNS,
    getRowId: (r) => r.id,
    ...opts,
  })
}

describe('editing — start / state slice / getIsEditing', () => {
  it('editing state starts null', () => {
    expect(mk().state.editing()).toBeNull()
  })

  it('startEditing sets the editing slice and the cell reports getIsEditing', () => {
    const table = mk()
    table.startEditing('a', 'age')
    expect(table.state.editing()).toEqual({ rowId: 'a', columnId: 'age' })
    const cell = table
      .getRow('a')!
      .getVisibleCells()
      .find((c) => c.column.id === 'age')!
    expect(cell.getIsEditing()).toBe(true)
  })

  it('a non-editable column cannot enter edit (enableEditing gate)', () => {
    const table = mk()
    table.startEditing('a', 'name') // name has no enableEditing
    expect(table.state.editing()).toBeNull()
  })
})

describe('editing — commit / cancel', () => {
  it('commitEdit writes the value back and clears the editing slice', () => {
    const onCellEdit = vi.fn()
    const table = mk({ onCellEdit })
    table.startEditing('a', 'age')
    table.commitEdit(42)
    expect(table.getRow('a')!.getValue('age')).toBe(42)
    expect(table.state.editing()).toBeNull()
  })

  it('cancelEdit clears the editing slice WITHOUT writing the value', () => {
    const table = mk()
    table.startEditing('a', 'age')
    table.cancelEdit()
    expect(table.getRow('a')!.getValue('age')).toBe(1) // unchanged
    expect(table.state.editing()).toBeNull()
  })
})

describe('editing — column validation hook', () => {
  it('a failing validation REJECTS the commit (value unchanged, still editing)', () => {
    const table = createTable<Row>({
      data: ROWS,
      columns: [
        { id: 'name', accessorKey: 'name' },
        {
          id: 'age',
          accessorKey: 'age',
          enableEditing: true,
          meta: { validate: (v: unknown) => typeof v === 'number' && (v as number) >= 0 },
        },
      ],
      getRowId: (r) => r.id,
    })
    table.startEditing('a', 'age')
    table.commitEdit(-5) // invalid
    expect(table.getRow('a')!.getValue('age')).toBe(1) // not written
    expect(table.state.editing()).toEqual({ rowId: 'a', columnId: 'age' }) // still open
  })

  it('a passing validation commits normally', () => {
    const table = createTable<Row>({
      data: ROWS,
      columns: [
        { id: 'name', accessorKey: 'name' },
        {
          id: 'age',
          accessorKey: 'age',
          enableEditing: true,
          meta: { validate: (v: unknown) => typeof v === 'number' && (v as number) >= 0 },
        },
      ],
      getRowId: (r) => r.id,
    })
    table.startEditing('a', 'age')
    table.commitEdit(7)
    expect(table.getRow('a')!.getValue('age')).toBe(7)
    expect(table.state.editing()).toBeNull()
  })
})

describe('editing — only one cell edits at a time', () => {
  it('starting a new edit replaces the previous editing target', () => {
    const table = createTable<Row>({
      data: ROWS,
      columns: [
        { id: 'name', accessorKey: 'name' },
        { id: 'age', accessorKey: 'age', enableEditing: true },
      ],
      getRowId: (r) => r.id,
    })
    table.startEditing('a', 'age')
    table.startEditing('b', 'age')
    expect(table.state.editing()).toEqual({ rowId: 'b', columnId: 'age' })
  })
})
