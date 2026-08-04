// sail-tree — windowed row-view builders (the reactive glue, kept off the
// controller to hold the soft cap).
//
// Maps the engine's flattened, depth-first, expansion-respecting `tree.rows()`
// (already depth-tagged, with getCanExpand/getIsExpanded/getPosInSet/getSetSize/
// getIsSelected/getIsSomeSelected from the #240 follow-up) to flat plain-data
// view objects the templates render in a keyed `{#for}`. ONLY the ~window of
// visible rows is mapped (never all of `tree.rows()`), so a 100k-node tree
// allocates ~overscan+viewport views.
//
// PURE `@ship/sail-runtime` — no DOM, no SDK. The flat-list ARIA (level/posinset/
// setsize) reads the engine's `Row<T>` helpers directly; there is no sibling
// walk to reinvent.
//
// Authority: build-spec §"Accessibility" (flat-list ARIA), §"The view", Slices
// 3-5; behavioral specs #1/#2/#4/#6/#8.

import { derived, type Derived } from '@ship/sail-runtime'
import type { VirtualItem } from '@ship/sail-runtime/virtual'
import type { Table } from '../sail-table/engine/create-table'
import type { CellValue, ColumnApi, Row } from '../sail-table/engine/types'
import type { TreeNode } from '../sail-tree-view/sail-tree-view.controller'

/** A rendered cell in a tree-grid row (the disclosure lives in the FIRST one). */
export interface TreeCellView {
  columnId: string
  value: CellValue
  /** True for the first visible column — carries the caret + indent. */
  first: boolean
}

/** A windowed tree/tree-grid row: virtual-item geometry zipped with the engine
 *  row's flat-list ARIA + disclosure + selection so the template renders with no
 *  per-row engine reads (declarative aria, per `controller.md`). */
export interface TreeRowView {
  key: VirtualItem['key']
  /** Virtualizer index into `tree.rows()` (drives aria-rowindex + transform). */
  index: number
  start: number
  size: number
  id: string
  label: string
  /** Per-node `<sail-icon>` name from `node.icon` (`''` when unset) — the
   *  template renders it ahead of the label, else falls back to the slot. */
  icon: string
  /**
   * Per-node trailing meta from `node.trailing` (`''` when unset) — rendered
   * in the row's trail slot (dirty dots, badges) without cramming the label.
   */
  trailing: string
  depth: number
  level: number
  posInSet: number
  setSize: number
  canExpand: boolean
  expanded: boolean
  /** Row HIGHLIGHT (`data-selected` / `aria-selected`): the EXPLICIT engine
   *  selection membership (`selectedRowIds`), NOT the branch leaf-cascade — a
   *  branch highlights only when it is itself in the selected set, never merely
   *  because all its leaf descendants are (that surfaces as the checkbox tri-state
   *  below). Matches the interactions reflect exactly, so the two never disagree. */
  selected: boolean
  /** Checkbox CHECKED state: the branch leaf-cascade (`row.getIsSelected()`) — a
   *  branch is checked when every selectable leaf descendant is selected. */
  checked: boolean
  someSelected: boolean
  /** A synthetic "loading children" placeholder row (consumer-injected for the
   *  lazy seam) — the view styles it as a skeleton, not a real selectable node. */
  pending: boolean
  /** Per-level indent (px) — depth × the density-derived step, set inline. */
  indent: number
  /** tree-grid only — the row's visible cells (label-only `sail-tree` ignores). */
  cells: TreeCellView[]
  /** The visible-column signature (tree-grid) — folded into the row `{#for}` key
   *  so a column hide/show re-keys the row and the cells re-render (the keyed-for
   *  reuses a same-key row's DOM and would otherwise keep the stale cell set). */
  cellsKey: string
}

/** The label of a row — its first visible cell value, or the engine row label
 *  for the label-only `sail-tree` (no columns). */
function rowLabel<T extends TreeNode>(row: Row<T>, cols: ColumnApi<T>[]): string {
  if (cols.length > 0) {
    const v = row.getValue(cols[0]!.id)
    return v == null ? '' : String(v)
  }
  return row.original?.label ?? ''
}

function rowCells<T>(row: Row<T>, cols: ColumnApi<T>[]): TreeCellView[] {
  if (cols.length === 0) return []
  return row.getVisibleCells().map((cell, i) => ({
    columnId: cell.column.id,
    value: cell.getValue(),
    first: i === 0,
  }))
}

/** The visible-column signature for a tree-grid row's `{#for}` key — empty for
 *  the label-only `sail-tree` (which has no columns). */
function cellsKeyOf(cells: TreeCellView[]): string {
  return cells.map((c) => c.columnId).join(',')
}

/** Map a single engine `Row<T>` + its virtual item to a flat `TreeRowView`. */
function toRowView<T extends TreeNode>(
  row: Row<T>,
  item: VirtualItem,
  cols: ColumnApi<T>[],
  indentStep: number,
  selectedSet: Set<string>,
): TreeRowView {
  // The lazy seam: a consumer marks a branch whose children load on first expand
  // with `original.lazy`. The engine reports `getCanExpand()=false` while that
  // branch carries no sub-rows yet, so the view honors the consumer's `lazy` flag
  // to keep the disclosure caret present before the first load.
  const orig = row.original as {
    lazy?: boolean
    pending?: boolean
    icon?: string
    trailing?: string
  } | undefined
  const pending = orig?.pending === true
  const canExpand = row.getCanExpand() || orig?.lazy === true
  const cells = rowCells(row, cols)
  return {
    key: item.key,
    index: item.index,
    start: item.start,
    size: item.size,
    id: row.id,
    label: rowLabel(row, cols),
    icon: orig?.icon != null ? String(orig.icon) : '',
    trailing: orig?.trailing != null ? String(orig.trailing) : '',
    depth: row.depth,
    level: row.depth + 1,
    posInSet: row.getPosInSet(),
    setSize: row.getSetSize(),
    canExpand,
    expanded: canExpand && row.getIsExpanded(),
    // Row highlight tracks EXPLICIT selection membership (matches the reflect);
    // the checkbox tracks the branch leaf-cascade.
    selected: selectedSet.has(row.id),
    checked: row.getIsSelected(),
    someSelected: row.getIsSomeSelected(),
    pending,
    indent: row.depth * indentStep,
    cells,
    cellsKey: cellsKeyOf(cells),
  }
}

/** A tree-grid column header (the `role=columnheader` cell + its sort/hide/resize
 *  affordance flags, derived from the engine's column lane). */
export interface TreeHeaderView {
  id: string
  label: string
  /** Sort state for `aria-sort` + the sort glyph (`asc`/`desc`/false). */
  sorted: 'asc' | 'desc' | false
  /** `aria-sort` token mapping the engine's `getIsSorted()`. */
  ariaSort: 'ascending' | 'descending' | 'none'
  canSort: boolean
  canHide: boolean
  canResize: boolean
  /** Resolved column width var value (e.g. `'150px'`) — the grid template track. */
  size: string
}

function headerLabel<T>(col: ColumnApi<T>): string {
  const h = col.columnDef.header
  return typeof h === 'string' ? h : col.id
}

/** The visible leaf headers as flat `TreeHeaderView`s (tree-grid only). Reads the
 *  engine's column lane (sort/visibility/resize) so a header reflects the live
 *  state with no per-column engine read in the template (declarative aria). */
export function treeHeaderViews<T extends TreeNode>(table: Table<T>): TreeHeaderView[] {
  return table.visibleLeafColumns().map((col) => {
    const sorted = col.getIsSorted()
    return {
      id: col.id,
      label: headerLabel(col),
      sorted,
      ariaSort: sorted === 'asc' ? 'ascending' : sorted === 'desc' ? 'descending' : 'none',
      canSort: col.getCanSort(),
      canHide: col.getCanHide(),
      canResize: col.getCanResize(),
      size: table.getColumnSizeVarValue(col.id),
    }
  })
}

/** Build the windowed `TreeRowView[]` derived. Recomputes when the window
 *  identity breaks (a real move / measure / expand-collapse) or the engine rows
 *  change; pure-scroll frames short-circuit upstream in `virtualItems`. */
export function windowTreeRows<T extends TreeNode>(
  table: () => Table<T>,
  items: Derived<readonly VirtualItem[]>,
  indentStep: () => number,
): Derived<TreeRowView[]> {
  return derived<TreeRowView[]>(() => {
    const tab = table()
    const rows = tab.rows()
    const cols = tab.visibleLeafColumns()
    const step = indentStep()
    // EXPLICIT selection membership (the displayed selectedRowIds, no branch
    // cascade) drives the per-row highlight — the same source the interactions
    // reflect reads, so template + reflect never fight over `data-selected`.
    const selectedSet = new Set(tab.selectedRowIds())
    const out: TreeRowView[] = []
    for (const it of items()) {
      const row = rows[it.index]
      if (!row) continue
      out.push(toRowView(row, it, cols, step, selectedSet))
    }
    return out
  })
}

/**
 * Full-list (non-virtual) row views — document flow.
 * Prefer for explorer-scale trees: collapse reflows for free; no absolute
 * --row-y / estimateSize hitbox bugs (SPEC-TREE-1 path A).
 */
export function flowTreeRows<T extends TreeNode>(
  table: () => Table<T>,
  indentStep: () => number,
): Derived<TreeRowView[]> {
  return derived<TreeRowView[]>(() => {
    const tab = table()
    // expanded + rows so collapse re-derives the flat list.
    void tab.state.expanded()
    const rows = tab.rows()
    const cols = tab.visibleLeafColumns()
    const step = indentStep()
    const selectedSet = new Set(tab.selectedRowIds())
    const out: TreeRowView[] = []
    for (let i = 0; i < rows.length; i++) {
      const row = rows[i]!
      // Synthetic VirtualItem geometry — unused by flow layout (no absolute).
      const item: VirtualItem = {
        key: row.id,
        index: i,
        start: 0,
        size: 0,
        end: 0,
        lane: 0,
      }
      out.push(toRowView(row, item, cols, step, selectedSet))
    }
    return out
  })
}
