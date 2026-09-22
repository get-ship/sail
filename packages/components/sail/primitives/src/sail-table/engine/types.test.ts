// TESTS-FIRST — Backbone #1, Slice 1a: the ColumnDef discriminated union + its
// runtime validator.
//
// Status: RED. `./types` does not exist yet. The import is import-not-found until
// Slice 1a ships the union + `validateColumnDef`. DO NOT add the impl here.
//
// Authority: build-spec §"API sketch" (the `ColumnDef<T>` discriminated union:
// AccessorColumn | DisplayColumn | GroupColumn; the inline note "Column kind is a
// DISCRIMINATED UNION (not a 'exactly one of' comment) — validated by a test"),
// Slice 1a ("a validation test that an ill-formed column (two of
// accessorKey/accessorFn/columns) is rejected").

import { describe, expect, it } from 'vitest'
// RED: greenfield module.
import { validateColumnDef, type ColumnDef } from './types'

interface Person {
  id: string
  name: string
  age: number
}

describe('validateColumnDef — well-formed kinds accepted', () => {
  it('accepts an accessor column (accessorKey only)', () => {
    const col: ColumnDef<Person> = { id: 'name', accessorKey: 'name' }
    expect(validateColumnDef(col).ok).toBe(true)
  })

  it('accepts a display column (accessorFn, no accessorKey/columns)', () => {
    const col: ColumnDef<Person> = { id: 'actions', accessorFn: (r) => r.id }
    expect(validateColumnDef(col).ok).toBe(true)
  })

  it('accepts a group column (columns only)', () => {
    const col: ColumnDef<Person> = {
      id: 'grp',
      columns: [{ id: 'name', accessorKey: 'name' }],
    }
    expect(validateColumnDef(col).ok).toBe(true)
  })
})

describe('validateColumnDef — ill-formed columns rejected (the discriminant test)', () => {
  it('rejects a column carrying BOTH accessorKey and columns', () => {
    const bad = {
      id: 'oops',
      accessorKey: 'name',
      columns: [{ id: 'x', accessorKey: 'age' }],
    } as unknown as ColumnDef<Person>
    const res = validateColumnDef(bad)
    expect(res.ok).toBe(false)
  })

  it('rejects a column carrying BOTH accessorKey and accessorFn', () => {
    const bad = {
      id: 'oops',
      accessorKey: 'name',
      accessorFn: (r: Person) => r.name,
    } as unknown as ColumnDef<Person>
    expect(validateColumnDef(bad).ok).toBe(false)
  })

  it('rejects a column carrying BOTH accessorFn and columns', () => {
    const bad = {
      id: 'oops',
      accessorFn: (r: Person) => r.name,
      columns: [{ id: 'x', accessorKey: 'age' }],
    } as unknown as ColumnDef<Person>
    expect(validateColumnDef(bad).ok).toBe(false)
  })

  it('rejects a column with NO id', () => {
    const bad = { accessorKey: 'name' } as unknown as ColumnDef<Person>
    expect(validateColumnDef(bad).ok).toBe(false)
  })
})
