// TESTS-FIRST — Backbone #1, Slice 3a: the ORTHOGONAL column lane (Features #8-#11).
//
// Status: RED. `./columns` and the Slice-3a `ColumnApi` additions / `setColumnSizing`
// mutator do not exist yet (engine is greenfield). The import below fails to resolve
// until Slice 3a ships — that import-not-found is the expected red for this contract.
// DO NOT add the impl here. DO NOT weaken these into passing stubs.
//
// Authority: build-spec §Features #8 (column sizing/resize: resize mode onChange/onEnd,
// flexible px/'auto'/grow, getTotalSize, min/max clamp + autofit, ONE `--col-<id>-size`
// source, "Presentational — never re-runs the row pipeline"), #9 (column ordering),
// #10 (column visibility), #11 (column pinning: `getStart('left'|'right')` cumulative
// offsets from the resolved sizing map, `getIsPinned`/`getPinnedIndex`/
// `getIsLastColumn`), §"Signal graph" ("ORTHOGONAL (presentational, NOT in the row
// chain)" — `setColumnSizing recomputes nothing in the row pipeline`), §API sketch
// (`visibleLeafColumns`, `setColumnSizing(u, opts?: {mode?})`, `setColumnOrder`,
// `setColumnVisibility`, `setColumnPinning`, `pinColumn`, `getTotalSize`),
// Slice 3a ("visibleLeafColumns honors order then visibility then pin-bucket; getStart
// offsets from the resolved sizing map; … The setColumnSizing does NOT recompute rows
// assertion lands here").
//
// Engine purity (Cardinal 4): imports ONLY @ship/sui-runtime.

import { describe, expect, it, vi } from 'vitest'
import { derived } from '@ship/sui-runtime'
// RED: greenfield module — resolves only once Slice 3a ships.
import { createTable } from './create-table'

interface Row {
  id: string
  a: string
  b: string
  c: string
  d: string
}

const DATA: Row[] = [
  { id: 'r1', a: '1', b: '2', c: '3', d: '4' },
  { id: 'r2', a: '5', b: '6', c: '7', d: '8' },
]

// Each column declares an explicit px size so the resolved sizing map is deterministic.
const COLUMNS = [
  { id: 'a', accessorKey: 'a' as const, size: 100 },
  { id: 'b', accessorKey: 'b' as const, size: 120 },
  { id: 'c', accessorKey: 'c' as const, size: 80 },
  { id: 'd', accessorKey: 'd' as const, size: 60 },
]

function table() {
  return createTable<Row>({ data: DATA, columns: COLUMNS, getRowId: (r) => r.id })
}

describe('visibleLeafColumns — order → visibility → pin-bucket', () => {
  it('default visible columns are in def order', () => {
    const t = table()
    expect(t.visibleLeafColumns().map((c) => c.id)).toEqual(['a', 'b', 'c', 'd'])
  })

  it('columnOrder reorders the visible leaves; absent ids fall back to def position', () => {
    const t = table()
    t.setColumnOrder(['d', 'b']) // partial order: d, b first; a, c keep def order
    expect(t.visibleLeafColumns().map((c) => c.id)).toEqual(['d', 'b', 'a', 'c'])
  })

  it('columnVisibility drops hidden columns from visibleLeafColumns', () => {
    const t = table()
    t.setColumnVisibility({ b: false })
    expect(t.visibleLeafColumns().map((c) => c.id)).toEqual(['a', 'c', 'd'])
    expect(t.getColumn('b')!.getIsVisible()).toBe(false)
  })

  it('pin-bucket wins over order: left-pinned leads, right-pinned trails', () => {
    const t = table()
    t.pinColumn('c', 'left')
    t.pinColumn('a', 'right')
    // left bucket: c ; center: b, d ; right bucket: a
    expect(t.visibleLeafColumns().map((c) => c.id)).toEqual(['c', 'b', 'd', 'a'])
    expect(t.leftLeafColumns().map((c) => c.id)).toEqual(['c'])
    expect(t.rightLeafColumns().map((c) => c.id)).toEqual(['a'])
  })
})

describe('getStart — cumulative offsets from the resolved sizing map', () => {
  it("getStart('left') accumulates left-pinned widths in pin order", () => {
    const t = table()
    t.pinColumn('a', 'left') // 100
    t.pinColumn('b', 'left') // 120, after a
    expect(t.getColumn('a')!.getStart('left')).toBe(0)
    expect(t.getColumn('b')!.getStart('left')).toBe(100)
  })

  it('resizing a left-pinned column reactively reflows subsequent offsets', () => {
    const t = table()
    t.pinColumn('a', 'left')
    t.pinColumn('b', 'left')
    expect(t.getColumn('b')!.getStart('left')).toBe(100)
    t.setColumnSizing({ a: 140 })
    expect(t.getColumn('b')!.getStart('left')).toBe(140)
  })

  it("getStart('right') accumulates right-pinned widths from the right edge", () => {
    const t = table()
    t.pinColumn('d', 'right') // 60
    t.pinColumn('c', 'right') // 80, inside d
    expect(t.getColumn('d')!.getStart('right')).toBe(0)
    expect(t.getColumn('c')!.getStart('right')).toBe(60)
  })
})

describe('flexible sizing + getTotalSize', () => {
  it('getTotalSize sums the resolved px widths of visible columns', () => {
    const t = table()
    expect(t.getTotalSize()).toBe(360) // 100 + 120 + 80 + 60
  })

  it('a hidden column drops out of getTotalSize', () => {
    const t = table()
    t.setColumnVisibility({ c: false })
    expect(t.getTotalSize()).toBe(280) // 360 - 80
  })

  it("'auto' and grow columns still resolve to a numeric width in the sizing map", () => {
    const t = createTable<Row>({
      data: DATA,
      getRowId: (r) => r.id,
      columns: [
        { id: 'a', accessorKey: 'a', size: 100 },
        { id: 'b', accessorKey: 'b', size: 'auto' },
        { id: 'c', accessorKey: 'c', grow: 1 },
        { id: 'd', accessorKey: 'd', size: 60 },
      ],
    })
    for (const c of t.visibleLeafColumns()) {
      expect(typeof c.getSize()).toBe('number')
      expect(c.getSize()).toBeGreaterThan(0)
    }
    expect(typeof t.getTotalSize()).toBe('number')
  })
})

describe('resize — mode onChange/onEnd + min/max clamp + autofit', () => {
  it('default onChange mode writes the new size immediately', () => {
    const t = table()
    t.setColumnSizing({ a: 150 })
    expect(t.getColumn('a')!.getSize()).toBe(150)
  })

  it('min/max clamp the resolved size', () => {
    const t = createTable<Row>({
      data: DATA,
      getRowId: (r) => r.id,
      columns: [
        { id: 'a', accessorKey: 'a', size: 100, minSize: 80, maxSize: 200 },
        { id: 'b', accessorKey: 'b', size: 120 },
        { id: 'c', accessorKey: 'c', size: 80 },
        { id: 'd', accessorKey: 'd', size: 60 },
      ],
    })
    t.setColumnSizing({ a: 500 }) // over max
    expect(t.getColumn('a')!.getSize()).toBe(200)
    t.setColumnSizing({ a: 10 }) // under min
    expect(t.getColumn('a')!.getSize()).toBe(80)
  })

  it('onEnd mode commits only on the end signal, not on every interim write', () => {
    const onColumnSizingChange = vi.fn()
    const t = createTable<Row>({
      data: DATA,
      columns: COLUMNS,
      getRowId: (r) => r.id,
      onColumnSizingChange,
    })
    t.setColumnSizing({ a: 130 }, { mode: 'onEnd' })
    t.setColumnSizing({ a: 145 }, { mode: 'onEnd' })
    expect(onColumnSizingChange).not.toHaveBeenCalled() // deferred until end
    t.setColumnSizing({ a: 160 }) // onChange (the commit)
    expect(onColumnSizingChange).toHaveBeenCalledTimes(1)
    expect(t.getColumn('a')!.getSize()).toBe(160)
  })
})

describe('one --col-<id>-size source feeds header + body', () => {
  it('getSize() (header) and the body var derive from the SAME resolved map', () => {
    const t = table()
    t.setColumnSizing({ b: 200 })
    const col = t.getColumn('b')!
    // The header reads getSize(); the body reads the --col-b-size var. ONE source.
    expect(col.getSize()).toBe(200)
    expect(t.getColumnSizeVar('b')).toBe('--col-b-size')
    // ONE source: the body var resolves to the SAME size the header reads —
    // assert that invariant, not the px-formatted literal.
    expect(parseFloat(t.getColumnSizeVarValue('b')!)).toBe(col.getSize())
  })
})

describe('pin queries — getIsPinned / getPinnedIndex / getIsLastColumn', () => {
  it('getIsPinned reports the bucket; getPinnedIndex is the within-bucket index', () => {
    const t = table()
    t.pinColumn('a', 'left')
    t.pinColumn('c', 'left')
    expect(t.getColumn('a')!.getIsPinned()).toBe('left')
    expect(t.getColumn('a')!.getPinnedIndex()).toBe(0)
    expect(t.getColumn('c')!.getPinnedIndex()).toBe(1)
    expect(t.getColumn('b')!.getIsPinned()).toBe(false)
  })

  it("getIsLastColumn('left') marks the pin-boundary column", () => {
    const t = table()
    t.pinColumn('a', 'left')
    t.pinColumn('c', 'left')
    expect(t.getColumn('a')!.getIsLastColumn('left')).toBe(false)
    expect(t.getColumn('c')!.getIsLastColumn('left')).toBe(true)
  })

  it('unpinning returns the column to the center bucket', () => {
    const t = table()
    t.pinColumn('a', 'left')
    expect(t.getColumn('a')!.getIsPinned()).toBe('left')
    t.pinColumn('a', false)
    expect(t.getColumn('a')!.getIsPinned()).toBe(false)
  })
})

describe('ORTHOGONALITY PROOF — setColumnSizing does NOT recompute the row pipeline', () => {
  it('a sizing write recomputes the column lane but never re-derives rows', () => {
    const t = table()

    const rowRecompute = vi.fn()
    const rowProbe = derived(() => {
      rowRecompute()
      return t.rows().length
    })
    rowProbe() // seed
    expect(rowRecompute).toHaveBeenCalledTimes(1)

    const sizeRecompute = vi.fn()
    const sizeProbe = derived(() => {
      sizeRecompute()
      return t.getTotalSize()
    })
    sizeProbe() // seed
    expect(sizeRecompute).toHaveBeenCalledTimes(1)

    // The orthogonality contract: sizing is presentational only.
    t.setColumnSizing({ a: 175 })

    rowProbe()
    sizeProbe()
    // Rows did NOT recompute — sizing is off the row chain.
    expect(rowRecompute).toHaveBeenCalledTimes(1)
    // The column lane DID recompute — total size changed.
    expect(sizeRecompute).toHaveBeenCalledTimes(2)
    expect(t.getTotalSize()).toBe(435) // 175 + 120 + 80 + 60
  })
})
