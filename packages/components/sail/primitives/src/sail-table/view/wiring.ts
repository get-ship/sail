// sail-table view wiring — the data-variant glue between the headless engine
// (`createTable`) and the windowed `.sui` render (Backbone #1 × #2).
//
// Keeps `sail-table.controller.ts` lean (the controller is glue; this is the
// reactive plumbing it composes). PURE `@ship/sail-runtime` — no DOM, no SDK.
// The controller owns DOM-side imperative wiring (the host `table` property,
// delegated click/keydown, `host.emit`); this file owns the reactive deriveds
// the template reads.
//
// Authority: build-spec §"The view", §"Composition map", §"Behavioral spec
// list", Slices 5 + 7c.

import { derived, type Derived } from '@ship/sail-runtime'
import {
  createVirtualizer,
  type Virtual,
  type VirtualItem,
} from '@ship/sail-runtime/virtual'
import type { Table } from '../engine/create-table'
import type { CellValue, ColumnApi, Row } from '../engine/types'

export type AnyRow = Record<string, unknown> & { id: string }

/** A leaf header descriptor the template renders into a `[data-sort-key]` cell. */
export interface HeaderView {
  id: string
  label: string
  sortKey: string | null
  ariaSort: 'none' | 'ascending' | 'descending'
  canSort: boolean
  canFilter: boolean
  /** The sort caret glyph name for `sail-icon` (replaces the inline svg). */
  caret: string
  filterValue: string
}

/** A rendered cell — value + the column's align/format meta the view reads. */
export interface CellView {
  columnId: string
  value: CellValue
  align: string
}

/** A rendered row in the windowed center band. */
export interface RowView {
  id: string
  index: number
  selected: boolean
  selectable: boolean
  cells: CellView[]
}

type AriaSort = 'none' | 'ascending' | 'descending'
const ARIA_SORT: Record<'asc' | 'desc', AriaSort> = {
  asc: 'ascending',
  desc: 'descending',
}

function caretFor(sorted: 'asc' | 'desc' | false): string {
  if (sorted === 'asc') return 'chevron-up'
  if (sorted === 'desc') return 'chevron-down'
  return 'chevrons-up-down'
}

function headerLabel<T>(col: ColumnApi<T>): string {
  const h = col.columnDef.header
  return typeof h === 'string' ? h : col.id
}

function cellAlign<T>(col: ColumnApi<T>): string {
  const a = col.columnDef.meta?.align
  return typeof a === 'string' ? a : 'start'
}

/** The visible leaf headers as flat `HeaderView`s (engine is the source). A
 *  pure read — wrap in a single `derived()` in the controller scope. */
export function headerViews<T>(table: Table<T>): HeaderView[] {
  return table.visibleLeafColumns().map((col) => {
    const sorted = col.getIsSorted()
    return {
      id: col.id,
      label: headerLabel(col),
      sortKey: col.getCanSort() ? col.id : null,
      ariaSort: sorted ? ARIA_SORT[sorted] : 'none',
      canSort: col.getCanSort(),
      canFilter: col.getCanFilter(),
      caret: caretFor(sorted),
      filterValue: String(col.getFilterValue() ?? ''),
    }
  })
}

/** Map a single center-band `Row<T>` to its flat `RowView`. */
function rowView<T>(row: Row<T>, cols: ColumnApi<T>[]): RowView {
  return {
    id: row.id,
    index: row.index,
    selected: row.getIsSelected(),
    selectable: row.getCanSelect(),
    cells: cols.map((col) => ({
      columnId: col.id,
      value: row.getValue(col.id),
      align: cellAlign(col),
    })),
  }
}

/** Header select-all tri-state for the gutter header `sail-checkbox`. */
export interface HeaderSelectState {
  checked: boolean
  indeterminate: boolean
}
export function headerSelectState<T>(table: Table<T>): HeaderSelectState {
  const all = table.getIsAllPageRowsSelected()
  const some = table.getIsSomePageRowsSelected()
  return { checked: all, indeterminate: !all && some }
}

/** A windowed row: the virtual-item geometry zipped with its `RowView` so the
 *  template renders `{#for w of windowRows() key={w.key}}` with no index
 *  arithmetic. Maps ONLY the ~window of visible rows (never all `centerRows()`),
 *  so a 100k-row dataset allocates ~overscan+viewport RowViews, not 100k.
 *  Recomputes only when `virtualItems()` identity breaks (a real window move or
 *  a measure frame) or the center band changes — pure-scroll-noise frames
 *  short-circuit. */
export interface WindowRow {
  key: string | number
  index: number
  start: number
  size: number
  row: RowView
}
export function windowRows<T>(
  table: () => Table<T>,
  items: Derived<readonly VirtualItem[]>,
): Derived<WindowRow[]> {
  return derived<WindowRow[]>(() => {
    const tab = table()
    const cols = tab.visibleLeafColumns()
    const center = tab.centerRows()
    const out: WindowRow[] = []
    for (const it of items()) {
      const row = center[it.index]
      if (!row) continue
      out.push({
        key: it.key as string | number,
        index: it.index,
        start: it.start,
        size: it.size,
        row: rowView(row, cols),
      })
    }
    return out
  })
}

/** Build the virtualizer over the engine's center band. `getViewport` is the
 *  element thunk (null until `bind:this` mounts); `rowHeight` seeds the
 *  estimate. The center-row identity keys the window so `{#for}` recycles. */
export function buildViewportVirtualizer<T>(
  table: () => Table<T> | null,
  getViewport: () => HTMLElement | null,
  rowHeight: number,
): Virtual {
  return createVirtualizer({
    count: () => table()?.centerRows().length ?? 0,
    estimateSize: () => rowHeight,
    getScrollElement: getViewport,
    overscan: 8,
    getItemKey: (i) => table()?.centerRows()[i]?.id ?? i,
  })
}

/** Resolve the live `Row<T>` for an id over the engine's center band (used by
 *  the selection click path so toggles go through the engine, not the view). */
export function findCenterRow<T>(table: Table<T>, id: string): Row<T> | undefined {
  return table.centerRows().find((r) => r.id === id) ?? table.getRow(id)
}
