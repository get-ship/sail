// TESTS-FIRST — sui-sheet FOUNDATIONAL slice: two-axis virtualized grid.
//
// Status: RED. `./create-sheet` does not exist yet (greenfield pack). The import
// is import-not-found until the foundational slice ships. DO NOT add the impl
// here. DO NOT weaken into stubs.
//
// Authority: build-spec §Features #9 ("Infinite/large virtualized grid — both
// axes windowed via createVirtualizer(); logical extent may far exceed authored
// cells (sparse); totalWidth/totalHeight make both scrollbars honest"), §"Signal
// graph" (centerRowRange ← row-virtualizer; centerColRange ← COLUMN-virtualizer;
// frozen bands render OUTSIDE both windows), §Features #7 (frozen rows/cols
// partition into 4 quadrants; the center is the only band the virtualizers
// count), Behavioral spec #10 ("two-axis virtualization (large grid)") +
// authoring-notes ("the consumer that exercises `horizontal` column
// virtualization … two independent virtualizers over one viewport").
//
// This is the Backbone-#2 STRESS surface — the spec calls it out as a
// virtualizer FINDING to record, not a build blocker. The sheet engine exposes
// the count-source ranges (centerRowRange / centerColRange) + spacer geometry
// (totalWidth / totalHeight); the VIEW (Slice 7) creates the two actual
// `createVirtualizer` instances in `mount`. Here we pin the ENGINE-side surface
// those virtualizers bind to.
//
// Engine purity (Cardinal 4): imports ONLY @ship/sui-runtime.

import { describe, expect, it } from 'vitest'
import { createVirtualizer } from '@ship/sui-runtime/virtual'
// RED: greenfield module.
import { createSheet } from './create-sheet'
import type { WorkbookDef } from './contract'

function bigBook(opts: {
  rowCount: number
  colCount: number
  frozenRows?: number
  frozenCols?: number
}): WorkbookDef {
  return {
    sheets: [
      {
        id: 's1',
        name: 'Sheet1',
        rowCount: opts.rowCount,
        colCount: opts.colCount,
        cells: { A1: { value: 1 } }, // sparse: 1 authored cell over a huge extent
        ...(opts.frozenRows != null ? { frozenRows: opts.frozenRows } : {}),
        ...(opts.frozenCols != null ? { frozenCols: opts.frozenCols } : {}),
        defaultRowHeight: 24,
        defaultColWidth: 80,
      },
    ],
    activeSheetId: 's1',
  }
}

describe('createSheet — spacer geometry (both scrollbars honest over a sparse extent)', () => {
  it('totalHeight reflects the LOGICAL row extent, not the authored-cell count', () => {
    const sheet = createSheet({ workbook: bigBook({ rowCount: 100_000, colCount: 1_000 }) })
    // 100k rows × 24px default — the spacer must span the logical extent.
    expect(sheet.totalHeight()).toBe(100_000 * 24)
  })

  it('totalWidth reflects the LOGICAL column extent (horizontal axis)', () => {
    const sheet = createSheet({ workbook: bigBook({ rowCount: 100_000, colCount: 1_000 }) })
    expect(sheet.totalWidth()).toBe(1_000 * 80)
  })

  it('rowHeight / colWidth read per-index overrides, falling back to the defaults', () => {
    const wb: WorkbookDef = {
      sheets: [
        {
          id: 's1',
          name: 'Sheet1',
          rowCount: 10,
          colCount: 10,
          cells: {},
          defaultRowHeight: 24,
          defaultColWidth: 80,
          rowHeights: { 2: 48 },
          colWidths: { 3: 200 },
        },
      ],
      activeSheetId: 's1',
    }
    const sheet = createSheet({ workbook: wb })
    expect(sheet.rowHeight(0)).toBe(24)
    expect(sheet.rowHeight(2)).toBe(48)
    expect(sheet.colWidth(0)).toBe(80)
    expect(sheet.colWidth(3)).toBe(200)
  })
})

describe('createSheet — center ranges drive the windowed body (count sources)', () => {
  it('centerRowRange is a Derived {start,end} span the row-virtualizer counts', () => {
    const sheet = createSheet({ workbook: bigBook({ rowCount: 100_000, colCount: 1_000 }) })
    const range = sheet.centerRowRange()
    expect(typeof range.start).toBe('number')
    expect(typeof range.end).toBe('number')
    expect(range.end).toBeGreaterThanOrEqual(range.start)
  })

  it('centerColRange is a Derived {start,end} span the COLUMN-virtualizer counts (horizontal)', () => {
    const sheet = createSheet({ workbook: bigBook({ rowCount: 100_000, colCount: 1_000 }) })
    const range = sheet.centerColRange()
    expect(typeof range.start).toBe('number')
    expect(typeof range.end).toBe('number')
    expect(range.end).toBeGreaterThanOrEqual(range.start)
  })
})

describe('createSheet — frozen bands are OUTSIDE the windowed center (sibling sticky quadrants)', () => {
  it('frozenRowCount / frozenColCount surface the leading frozen-band counts', () => {
    const sheet = createSheet({
      workbook: bigBook({ rowCount: 1_000, colCount: 1_000, frozenRows: 1, frozenCols: 1 }),
    })
    expect(sheet.frozenRowCount()).toBe(1)
    expect(sheet.frozenColCount()).toBe(1)
  })

  it('the center row span STARTS after the frozen rows (the virtualizer never counts them)', () => {
    const sheet = createSheet({
      workbook: bigBook({ rowCount: 1_000, colCount: 1_000, frozenRows: 2, frozenCols: 0 }),
    })
    // The windowed center begins past the 2 frozen rows — they are sticky bands,
    // not part of the windowed count.
    expect(sheet.centerRowRange().start).toBeGreaterThanOrEqual(2)
  })

  it('the center column span STARTS after the frozen cols (horizontal frozen band)', () => {
    const sheet = createSheet({
      workbook: bigBook({ rowCount: 1_000, colCount: 1_000, frozenRows: 0, frozenCols: 3 }),
    })
    expect(sheet.centerColRange().start).toBeGreaterThanOrEqual(3)
  })

  it('zero frozen bands → the center spans from index 0 in both axes', () => {
    const sheet = createSheet({ workbook: bigBook({ rowCount: 1_000, colCount: 1_000 }) })
    expect(sheet.frozenRowCount()).toBe(0)
    expect(sheet.frozenColCount()).toBe(0)
    expect(sheet.centerRowRange().start).toBe(0)
    expect(sheet.centerColRange().start).toBe(0)
  })
})

describe('Backbone #2 — TWO createVirtualizer instances over ONE viewport (the stress)', () => {
  // The runtime util exposes `horizontal` — sui-table left it out of scope; the
  // sheet is the consumer that exercises it. Pin that two independent
  // virtualizers (vertical rows + horizontal cols) bound to the engine's center
  // counts compose over a single scroll element without conflict.
  it('a vertical + a horizontal virtualizer window independently against the engine counts', () => {
    const sheet = createSheet({ workbook: bigBook({ rowCount: 100_000, colCount: 1_000 }) })
    const viewportEl = document.createElement('div')

    const rowV = createVirtualizer({
      count: () => sheet.centerRowRange().end - sheet.centerRowRange().start,
      estimateSize: (i) => sheet.rowHeight(i),
      getScrollElement: () => viewportEl,
    })
    const colV = createVirtualizer({
      count: () => sheet.centerColRange().end - sheet.centerColRange().start,
      estimateSize: (i) => sheet.colWidth(i),
      getScrollElement: () => viewportEl,
      horizontal: true,
    })

    // Each windows a bounded slice — never the full logical extent.
    expect(rowV.virtualItems().length).toBeLessThan(100_000)
    expect(colV.virtualItems().length).toBeLessThan(1_000)
    // Both produced a non-empty window (the engine's UNMEASURED_VIEWPORT floor).
    expect(rowV.virtualItems().length).toBeGreaterThan(0)
    expect(colV.virtualItems().length).toBeGreaterThan(0)

    rowV.dispose()
    colV.dispose()
  })
})
