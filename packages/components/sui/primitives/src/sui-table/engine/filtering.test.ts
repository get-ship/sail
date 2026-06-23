// TESTS-FIRST — Backbone #1, Slice 2: column + global filtering (Feature #3).
//
// Status: RED. `./filtering` and the Slice-2 filter wiring on `createTable` do not
// exist yet (engine is greenfield). The imports below fail to resolve until Slice 2
// ships — that import-not-found is the expected red for this contract. DO NOT add the
// impl here. DO NOT weaken these into passing stubs.
//
// Authority: build-spec §Features #3 ("Column + global filtering":
// `filteredRows = data ▸ globalFilter ▸ columnFilters`; value-type → default fn
// (`string → includesString`, `number → inNumberRange`, `array → arrIncludes`,
// primitive → `equals`); a string `filterFn:'fuzzy'` resolves against built-ins;
// per-fn value shapes text=string / range=[min,max] / set=unknown[]; a column opts
// out of global filter with `enableGlobalFilter:false`), §"Signal graph"
// (`preColumnFilteredRows` global-filtered pre-column-filter — the facet source),
// §API sketch (`setColumnFilter`, `setGlobalFilter`, `preColumnFilteredRows`,
// `filteredRows`), Slice 2.
//
// Engine purity (Cardinal 4): imports ONLY @ship/sui-runtime.

import { describe, expect, it } from 'vitest'
// RED: greenfield module — resolves only once Slice 2 ships.
import { createTable } from './create-table'

interface Item {
  id: string
  name: string
  qty: number
  tags: string[]
  active: boolean
}

const COLUMNS = [
  { id: 'name', accessorKey: 'name' as const },
  { id: 'qty', accessorKey: 'qty' as const },
  { id: 'tags', accessorKey: 'tags' as const },
  { id: 'active', accessorKey: 'active' as const },
]

const ITEMS: Item[] = [
  { id: 'a', name: 'Apple', qty: 3, tags: ['fruit', 'red'], active: true },
  { id: 'b', name: 'Banana', qty: 12, tags: ['fruit', 'yellow'], active: false },
  { id: 'c', name: 'Carrot', qty: 7, tags: ['veg', 'orange'], active: true },
  { id: 'd', name: 'Date', qty: 20, tags: ['fruit', 'brown'], active: false },
]

function table() {
  return createTable<Item>({ data: ITEMS, columns: COLUMNS, getRowId: (r) => r.id })
}

describe('global → column composition order (data ▸ globalFilter ▸ columnFilters)', () => {
  it('global filter narrows across filterable columns', () => {
    const t = table()
    t.setGlobalFilter('an') // matches Banana (name)
    expect(t.filteredRows().map((r) => r.id)).toEqual(['b'])
  })

  it('column filter narrows the global-filtered set, not the raw data', () => {
    const t = table()
    t.setGlobalFilter('a') // Apple, Banana, Carrot, Date all contain 'a'
    t.setColumnFilter('name', 'Ca') // then narrow to Carrot
    expect(t.filteredRows().map((r) => r.id)).toEqual(['c'])
  })

  it('preColumnFilteredRows is global-filtered but PRE-column-filter (the facet source)', () => {
    const t = table()
    t.setGlobalFilter('a') // Apple, Banana, Carrot, Date
    t.setColumnFilter('name', 'Apple')
    expect(t.preColumnFilteredRows().map((r) => r.id)).toEqual(['a', 'b', 'c', 'd'])
    expect(t.filteredRows().map((r) => r.id)).toEqual(['a'])
  })
})

describe('value-type → default filter-fn resolution', () => {
  it('string column → includesString (substring, case-insensitive)', () => {
    const t = table()
    t.setColumnFilter('name', 'rr') // Carrot
    expect(t.filteredRows().map((r) => r.id)).toEqual(['c'])
  })

  it('number column → inNumberRange with a [min, max] value shape', () => {
    const t = table()
    t.setColumnFilter('qty', [5, 15]) // Carrot(7), Banana(12)
    expect(new Set(t.filteredRows().map((r) => r.id))).toEqual(new Set(['b', 'c']))
  })

  it('inNumberRange treats an open bound (undefined) as unbounded', () => {
    const t = table()
    t.setColumnFilter('qty', [10, undefined]) // qty >= 10 → Banana(12), Date(20)
    expect(new Set(t.filteredRows().map((r) => r.id))).toEqual(new Set(['b', 'd']))
  })

  it('array column → arrIncludes with an unknown[] (set) value shape', () => {
    const t = table()
    t.setColumnFilter('tags', ['veg']) // Carrot has 'veg'
    expect(t.filteredRows().map((r) => r.id)).toEqual(['c'])
  })

  it('primitive (boolean) column → equals', () => {
    const t = table()
    t.setColumnFilter('active', true)
    expect(new Set(t.filteredRows().map((r) => r.id))).toEqual(new Set(['a', 'c']))
  })
})

describe('named filterFn resolution', () => {
  it("a column declaring filterFn:'fuzzy' resolves from built-ins", () => {
    const t = createTable<Item>({
      data: ITEMS,
      columns: [
        { id: 'name', accessorKey: 'name', filterFn: 'fuzzy' },
        { id: 'qty', accessorKey: 'qty' },
        { id: 'tags', accessorKey: 'tags' },
        { id: 'active', accessorKey: 'active' },
      ],
      getRowId: (r) => r.id,
    })
    // Fuzzy matches non-contiguous chars in order: "crt" ⊂ "Carrot".
    t.setColumnFilter('name', 'crt')
    expect(t.filteredRows().map((r) => r.id)).toEqual(['c'])
  })
})

describe('enableGlobalFilter:false opts a column out of the global scope', () => {
  it('a global query does NOT match through an opted-out column', () => {
    const t = createTable<Item>({
      data: ITEMS,
      columns: [
        { id: 'name', accessorKey: 'name' },
        { id: 'qty', accessorKey: 'qty' },
        // 'orange' lives only in tags; opting tags out hides that match.
        { id: 'tags', accessorKey: 'tags', enableGlobalFilter: false },
        { id: 'active', accessorKey: 'active' },
      ],
      getRowId: (r) => r.id,
    })
    t.setGlobalFilter('orange') // would match Carrot via tags, but tags is opted out
    expect(t.filteredRows()).toEqual([])
  })

  it('the remaining columns still answer the same global query', () => {
    const t = createTable<Item>({
      data: ITEMS,
      columns: [
        { id: 'name', accessorKey: 'name' },
        { id: 'qty', accessorKey: 'qty' },
        { id: 'tags', accessorKey: 'tags', enableGlobalFilter: false },
        { id: 'active', accessorKey: 'active' },
      ],
      getRowId: (r) => r.id,
    })
    t.setGlobalFilter('Carrot') // matches via name, not the opted-out tags
    expect(t.filteredRows().map((r) => r.id)).toEqual(['c'])
  })
})

describe('resetFilters + reactive recompute', () => {
  it('resetFilters clears global + column filters and restores all rows', () => {
    const t = table()
    t.setGlobalFilter('a')
    t.setColumnFilter('name', 'Apple')
    expect(t.filteredRows()).toHaveLength(1)
    t.resetFilters()
    expect(t.filteredRows().map((r) => r.id)).toEqual(['a', 'b', 'c', 'd'])
    expect(t.hasActiveFilters()).toBe(false)
  })

  it('hasActiveFilters reflects an active column filter', () => {
    const t = table()
    expect(t.hasActiveFilters()).toBe(false)
    t.setColumnFilter('qty', [5, 15])
    expect(t.hasActiveFilters()).toBe(true)
  })
})
