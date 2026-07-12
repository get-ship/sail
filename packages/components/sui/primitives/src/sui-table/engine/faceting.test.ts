// TESTS-FIRST — Backbone #1, Slice 3b: faceting (per-column faceted row model).
//
// Status: RED. `./create-table` (which wires `./faceting`) does not exist yet;
// import-not-found is the expected red. DO NOT add the impl here.
//
// Authority: build-spec §Features #4 ("Faceting": a faceted row model per column
// applies all OTHER columns' filters but not its own; wired off
// preColumnFilteredRows, NOT filteredRows; getFacetedUniqueValues (value→count),
// getFacetedMinMaxValues, a sorted-unique-values accessor; lazy + memoized;
// manual seam via the injected `facets` option), Slice 3b ("lazy memoized facets
// off preColumnFilteredRows with counts + sorted-values + minMax").

import { describe, expect, it, vi } from 'vitest'
// RED: greenfield module.
import { createTable } from './create-table'

interface Item {
  id: string
  cat: string
  price: number
}

const COLUMNS = [
  { id: 'cat', accessorKey: 'cat' as const },
  { id: 'price', accessorKey: 'price' as const },
]

const ITEMS: Item[] = [
  { id: 'a', cat: 'x', price: 10 },
  { id: 'b', cat: 'x', price: 20 },
  { id: 'c', cat: 'y', price: 30 },
  { id: 'd', cat: 'y', price: 40 },
  { id: 'e', cat: 'z', price: 50 },
]

function mk(opts: Record<string, unknown> = {}) {
  return createTable<Item>({
    data: ITEMS,
    columns: COLUMNS,
    getRowId: (r) => r.id,
    enableFilters: true,
    ...opts,
  })
}

describe('faceting — getFacetedUniqueValues (value → count)', () => {
  it('returns a Map of value→count over the faceted row model', () => {
    const facets = mk().getColumn('cat')!.getFacetedUniqueValues()
    expect(facets.get('x')).toBe(2)
    expect(facets.get('y')).toBe(2)
    expect(facets.get('z')).toBe(1)
  })
})

describe('faceting — column facet applies OTHER columns filters, NOT its own', () => {
  it("the cat facet does NOT collapse to its own selection (off preColumnFilteredRows)", () => {
    const table = mk()
    table.setColumnFilter('cat', 'x') // narrows filteredRows to x
    // the cat facet still lists ALL categories so the filter UI does not
    // collapse to the single chosen value.
    const catFacets = table.getColumn('cat')!.getFacetedUniqueValues()
    expect([...catFacets.keys()].sort()).toEqual(['x', 'y', 'z'])
  })

  it("a DIFFERENT column's facet DOES reflect the cat filter (other-column filters applied)", () => {
    const table = mk()
    table.setColumnFilter('cat', 'x') // x rows = price 10, 20
    const priceFacets = table.getColumn('price')!.getFacetedUniqueValues()
    expect([...priceFacets.keys()].sort((a, b) => (a as number) - (b as number))).toEqual([10, 20])
  })

  it('the global filter DOES narrow every facet (facets branch off preColumnFilteredRows)', () => {
    const table = mk()
    // global filter restricts the pre-column-filter set, so facets shrink with it.
    table.setGlobalFilter('y')
    const catFacets = table.getColumn('cat')!.getFacetedUniqueValues()
    expect([...catFacets.keys()]).toEqual(['y'])
  })
})

describe('faceting — getFacetedMinMaxValues', () => {
  it('returns [min, max] for a numeric column', () => {
    expect(mk().getColumn('price')!.getFacetedMinMaxValues()).toEqual([10, 50])
  })

  it('reflects other-column filters (min/max over the faceted model)', () => {
    const table = mk()
    table.setColumnFilter('cat', 'x') // price 10, 20
    expect(table.getColumn('price')!.getFacetedMinMaxValues()).toEqual([10, 20])
  })
})

describe('faceting — sorted unique values (typeahead filter UI source)', () => {
  it('getFacetedRowModel exposes the faceted rows for the column', () => {
    const rows = mk().getColumn('cat')!.getFacetedRowModel()
    expect(rows.length).toBe(ITEMS.length)
  })
})

describe('faceting — lazy + memoized', () => {
  it('does not compute facets until first accessed (lazy)', () => {
    const accessorCalls = vi.fn()
    const table = createTable<Item>({
      data: ITEMS,
      columns: [
        {
          id: 'cat',
          accessorKey: 'cat',
          getGroupingValue: (r: Item) => {
            accessorCalls()
            return r.cat
          },
        },
        { id: 'price', accessorKey: 'price' },
      ],
      getRowId: (r) => r.id,
      enableFilters: true,
    })
    // Merely constructing the table must not eagerly build the facet model.
    const before = accessorCalls.mock.calls.length
    expect(before).toBe(0)
    table.getColumn('cat')!.getFacetedUniqueValues()
    expect(accessorCalls.mock.calls.length).toBeGreaterThan(0)
  })

  it('repeated reads on a stable input return a memoized (identity-equal) Map', () => {
    const table = mk()
    const first = table.getColumn('cat')!.getFacetedUniqueValues()
    const second = table.getColumn('cat')!.getFacetedUniqueValues()
    expect(second).toBe(first)
  })
})

describe('faceting — manual seam (server-injected facets)', () => {
  it('manualFiltering reads facets from the injected facets option', () => {
    const table = createTable<Item>({
      data: ITEMS,
      columns: COLUMNS,
      getRowId: (r) => r.id,
      manualFiltering: true,
      facets: {
        cat: { uniqueValues: new Map<string | number, number>([['x', 99], ['y', 7]]) },
        price: { minMax: [5, 500] },
      } as never,
    })
    expect(table.getColumn('cat')!.getFacetedUniqueValues().get('x')).toBe(99)
    expect(table.getColumn('price')!.getFacetedMinMaxValues()).toEqual([5, 500])
  })
})
