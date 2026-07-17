// TESTS-FIRST — Backbone #1, Slice 3b: grouping + aggregation + expansion.
//
// Status: RED. `./create-table` (which wires `./grouping` + the Row tree members)
// does not exist yet; import-not-found is the expected red. DO NOT add the impl
// here.
//
// Authority: build-spec §Features #5 ("Grouping + aggregation": the AggFn set
// count|sum|min|max|mean|extent|median|unique|uniqueCount|mode|custom; grand-total
// footer over filteredRows via getFooterAggregate/footerGroups; a group row's
// getValue returns the aggregate), §Features #13 ("Row expansion": recursive
// flatten, depth/subRows/getCanExpand/getLeafRows/getParentRows, expanded:true =
// expand-all, filterFromLeafRows ancestor surfacing), Slice 3b.

import { describe, expect, it } from 'vitest'
// RED: greenfield module.
import { createTable } from './create-table'

interface Sale {
  id: string
  region: string
  amount: number
  month: string // e.g. '2026-01-15' → bucketed by month
}

const SALES: Sale[] = [
  { id: 's1', region: 'east', amount: 100, month: '2026-01-15' },
  { id: 's2', region: 'east', amount: 300, month: '2026-01-20' },
  { id: 's3', region: 'west', amount: 50, month: '2026-02-01' },
  { id: 's4', region: 'west', amount: 250, month: '2026-02-10' },
  { id: 's5', region: 'west', amount: 200, month: '2026-02-11' },
]

const COLUMNS = [
  { id: 'region', accessorKey: 'region' as const, enableGrouping: true },
  { id: 'amount', accessorKey: 'amount' as const, aggregationFn: 'sum' as const },
  { id: 'month', accessorKey: 'month' as const },
]

describe('grouping — synthetic group rows + nested levels', () => {
  it('grouping by region builds one group row per distinct value', () => {
    const table = createTable<Sale>({
      data: SALES,
      columns: COLUMNS,
      getRowId: (r) => r.id,
      grouping: ['region'],
      enableExpanding: true,
      expanded: true,
    })
    const groupRows = table.rows().filter((r) => r.getIsGrouped())
    expect(groupRows.map((r) => r.getGroupingValue('region')).sort()).toEqual([
      'east',
      'west',
    ])
  })

  it("a group row's getValue(aggColumn) returns the AGGREGATE for the group", () => {
    const table = createTable<Sale>({
      data: SALES,
      columns: COLUMNS,
      getRowId: (r) => r.id,
      grouping: ['region'],
      expanded: true,
    })
    const east = table.rows().find(
      (r) => r.getIsGrouped() && r.getGroupingValue('region') === 'east',
    )!
    expect(east.getValue('amount')).toBe(400) // 100 + 300
  })

  it('per-column getGroupingValue allows custom bucketing (e.g. by month)', () => {
    const table = createTable<Sale>({
      data: SALES,
      columns: [
        { id: 'region', accessorKey: 'region' },
        { id: 'amount', accessorKey: 'amount', aggregationFn: 'sum' },
        {
          id: 'month',
          accessorKey: 'month',
          enableGrouping: true,
          getGroupingValue: (r: Sale) => r.month.slice(0, 7), // YYYY-MM
        },
      ],
      getRowId: (r) => r.id,
      grouping: ['month'],
      expanded: true,
    })
    const buckets = table
      .rows()
      .filter((r) => r.getIsGrouped())
      .map((r) => r.getGroupingValue('month'))
      .sort()
    expect(buckets).toEqual(['2026-01', '2026-02'])
  })
})

describe('aggregation — the full AggFn set', () => {
  const westAmounts = [50, 250, 200] // sum 500, min 50, max 250, mean ~166.67

  function aggFor(fn: string) {
    const table = createTable<Sale>({
      data: SALES,
      columns: [
        { id: 'region', accessorKey: 'region' },
        { id: 'amount', accessorKey: 'amount', aggregationFn: fn as never },
      ],
      getRowId: (r) => r.id,
      grouping: ['region'],
      expanded: true,
    })
    return table
      .rows()
      .find((r) => r.getIsGrouped() && r.getGroupingValue('region') === 'west')!
      .getValue('amount')
  }

  it('sum', () => expect(aggFor('sum')).toBe(westAmounts.reduce((a, b) => a + b, 0)))
  it('min', () => expect(aggFor('min')).toBe(50))
  it('max', () => expect(aggFor('max')).toBe(250))
  it('mean', () => expect(aggFor('mean')).toBeCloseTo(500 / 3, 5))
  it('count', () => expect(aggFor('count')).toBe(3))
  it('extent → [min, max]', () => expect(aggFor('extent')).toEqual([50, 250]))
  it('median', () => expect(aggFor('median')).toBe(200))
  it('uniqueCount', () => expect(aggFor('uniqueCount')).toBe(3))
})

describe('aggregation — grand-total footer over filteredRows', () => {
  it('getFooterAggregate computes the grand total over filteredRows (not just a page)', () => {
    const table = createTable<Sale>({
      data: SALES,
      columns: COLUMNS,
      getRowId: (r) => r.id,
    })
    // grand total of all amounts = 100+300+50+250+200 = 900
    expect(table.getColumn('amount')!.getFooterAggregate()).toBe(900)
  })

  it('the grand total tracks the filtered set (footer over filteredRows)', () => {
    const table = createTable<Sale>({
      data: SALES,
      columns: COLUMNS,
      getRowId: (r) => r.id,
      enableFilters: true,
    })
    table.setColumnFilter('region', 'east') // east only: 100 + 300
    expect(table.getColumn('amount')!.getFooterAggregate()).toBe(400)
  })

  it('footerGroups exposes the per-column footer model (the real tfoot data)', () => {
    const table = createTable<Sale>({
      data: SALES,
      columns: COLUMNS,
      getRowId: (r) => r.id,
    })
    expect(table.footerGroups().length).toBeGreaterThan(0)
  })
})

describe('expansion — recursive flatten + tree members', () => {
  interface Node {
    id: string
    label: string
    children?: Node[]
  }
  const TREE: Node[] = [
    {
      id: 'root',
      label: 'Root',
      children: [
        { id: 'c1', label: 'Child 1', children: [{ id: 'g1', label: 'Grandchild' }] },
        { id: 'c2', label: 'Child 2' },
      ],
    },
  ]
  const TREE_COLS = [{ id: 'label', accessorKey: 'label' as const }]

  function mkTree(opts: Record<string, unknown> = {}) {
    return createTable<Node>({
      data: TREE,
      columns: TREE_COLS,
      getRowId: (r) => r.id,
      getSubRows: (r) => r.children,
      enableExpanding: true,
      ...opts,
    })
  }

  it('collapsed root shows only the top-level row', () => {
    const table = mkTree()
    expect(table.rows().map((r) => r.id)).toEqual(['root'])
    expect(table.getRow('root')!.getCanExpand()).toBe(true)
  })

  it('toggleExpanded reveals children depth-first; depth is tagged', () => {
    const table = mkTree()
    table.toggleExpanded('root')
    expect(table.rows().map((r) => r.id)).toEqual(['root', 'c1', 'c2'])
    expect(table.getRow('c1')!.depth).toBe(1)
  })

  it('expanded: true expands ALL levels (expand-all)', () => {
    const table = mkTree({ expanded: true })
    expect(table.rows().map((r) => r.id)).toEqual(['root', 'c1', 'g1', 'c2'])
  })

  it('getLeafRows / getParentRows / subRows expose the tree', () => {
    const table = mkTree({ expanded: true })
    const root = table.getRow('root')!
    expect(root.subRows.map((r) => r.id)).toEqual(['c1', 'c2'])
    expect(root.getLeafRows().map((r) => r.id).sort()).toEqual(['c2', 'g1'])
    expect(table.getRow('g1')!.getParentRows().map((r) => r.id)).toEqual(['root', 'c1'])
  })

  it('getCanExpand is false for a leaf with no subRows', () => {
    const table = mkTree({ expanded: true })
    expect(table.getRow('c2')!.getCanExpand()).toBe(false)
  })

  it('filterFromLeafRows surfaces a matching leaf with its ancestor chain', () => {
    const table = mkTree({
      expanded: true,
      enableFilters: true,
      filterFromLeafRows: true,
      columns: [{ id: 'label', accessorKey: 'label' }],
    })
    table.setGlobalFilter('Grandchild')
    // the matching leaf g1 surfaces along with its ancestors root → c1.
    const ids = table.rows().map((r) => r.id)
    expect(ids).toContain('g1')
    expect(ids).toContain('c1')
    expect(ids).toContain('root')
    expect(ids).not.toContain('c2')
  })
})
