// sail-table engine — grouping + expansion + the footer model (Slice 3b).
//
// Two row-tree sources collapse here into ONE flatten contract:
//
//   • TREE data (`getSubRows`): the core builds a depth-tagged Row tree; the
//     flatten walks it depth-first emitting only the visible descendants (a
//     collapsed parent hides its subtree). `filterFromLeafRows` surfaces a
//     matching leaf's ancestor chain; sort recurses per level.
//   • GROUPING (`grouping: string[]`): the flat sorted leaves are bucketed —
//     recursively, one level per grouping id — into synthetic group rows whose
//     `getValue(colId)` is the per-group AGGREGATE.
//
// Wired AFTER sort, BEFORE paginate (build-spec signal graph). The grand-total
// FOOTER model (`footerGroups` + `column.getFooterAggregate()`) is computed over
// `filteredRows`, orthogonal to the row chain.
//
// PURE TypeScript — imports `derived` from `@ship/sail-runtime` ONLY. No DOM, no
// SDK.
//
// Authority: build-spec §Features #5 (grouping + aggregation + footer over
// filteredRows), #13 (recursive flatten, depth/subRows/getCanExpand/getLeafRows/
// getParentRows, expanded:true expand-all, filterFromLeafRows + per-level sort),
// §"Signal graph", Slice 3b.

import { derived, type Derived, type Signal } from '@ship/sail-runtime'
import { hasAggregation, resolveAggregator } from './aggregation'
import { sortRows } from './pipeline'
import { makeGroupRow } from './row-api'
import type { RowBindings } from './row-api'
import type {
  CellValue,
  ColumnDef,
  ExpandedState,
  HeaderGroup,
  Row,
  SortState,
} from './types'

/** Is the expansion state expand-all (`true`) or does it gate `id`? */
export function isExpanded(state: ExpandedState, id: string): boolean {
  return state === true ? true : state[id] === true
}

/** Custom bucketing value for a column, else its accessor value. Exported so
 *  faceting buckets by the SAME value the grouping stage does (getGroupingValue
 *  honored — one bucketing contract). */
export function groupingValueOf<T>(
  row: Row<T>,
  def: ColumnDef<T> | undefined,
  colId: string,
): unknown {
  const custom = def && (def as { getGroupingValue?: (row: T) => unknown }).getGroupingValue
  return custom ? custom(row.original) : row.getValue(colId)
}

// ── TREE flatten (getSubRows source) ──────────────────────────────────────────

/**
 * Depth-first flatten of a Row tree: emit each row, then (when expanded) recurse
 * into its `subRows`. Sorting orders a node's CHILDREN within their parent (each
 * sibling band is ordered) while the ROOT order is preserved as received — the
 * top-level rows were already ordered by the upstream sort stage / source, and
 * re-sorting them here would shuffle the tree's top-level structure on a column
 * sort (a tree-grid sorts within a branch, it does not reshuffle the roots).
 */
export function flattenTree<T>(
  roots: Row<T>[],
  expanded: ExpandedState,
  order: SortState[],
): Row<T>[] {
  const out: Row<T>[] = []
  const walk = (rows: Row<T>[], isRoot: boolean): void => {
    const ordered = !isRoot && order.length > 0 ? sortRows(rows, order) : rows
    for (const row of ordered) {
      out.push(row)
      if (row.subRows.length > 0 && isExpanded(expanded, row.id)) walk(row.subRows, false)
    }
  }
  walk(roots, true)
  return out
}

/**
 * Tree filter (`filterFromLeafRows`): keep a subtree when the row itself OR any
 * descendant matches, so a matching leaf surfaces with its full ancestor chain.
 * Returns the surviving roots (children pruned to the matching set). Operates on
 * the ALREADY-BUILT tree so parent links stay intact (it never rebuilds rows —
 * it filters the existing `subRows` arrays in place via a structural mirror).
 */
export function filterTreeFromLeaves<T>(
  roots: Row<T>[],
  predicate: (row: Row<T>) => boolean,
): Row<T>[] {
  const keep = new Set<string>()
  const mark = (row: Row<T>): boolean => {
    let anyChild = false
    for (const child of row.subRows) if (mark(child)) anyChild = true
    const self = predicate(row)
    if (self || anyChild) {
      keep.add(row.id)
      return true
    }
    return false
  }
  for (const r of roots) mark(r)
  // Prune subRows arrays to the kept set (mutates the throwaway per-frame rows).
  const prune = (rows: Row<T>[]): Row<T>[] => {
    const kept = rows.filter((r) => keep.has(r.id))
    for (const r of kept) {
      const childrenKept = prune(r.subRows)
      r.subRows.length = 0
      r.subRows.push(...childrenKept)
    }
    return kept
  }
  return prune(roots)
}

// ── GROUPING flatten (grouping: string[] source) ──────────────────────────────

export interface GroupBuildInputs<T> {
  columns: ColumnDef<T>[]
  bindings: RowBindings<T>
}

/**
 * Recursively bucket `leaves` by the grouping ids, building one level of
 * synthetic group rows per id. A group row's leaf domain is all leaf
 * descendants; its `getValue(colId)` returns the per-group aggregate. The
 * flatten then emits each group row followed (when expanded) by its children.
 */
export function buildGroupedRows<T>(
  leaves: Row<T>[],
  groupIds: string[],
  expanded: ExpandedState,
  inputs: GroupBuildInputs<T>,
): Row<T>[] {
  const byId = new Map(inputs.columns.map((c) => [c.id, c]))
  const aggregators = new Map<string, ReturnType<typeof resolveAggregator<T>>>()
  for (const c of inputs.columns) aggregators.set(c.id, resolveAggregator(c))

  const aggregateOver = (leafDomain: Row<T>[], colId: string): CellValue => {
    const agg = aggregators.get(colId)
    if (!agg) return undefined
    return agg(colId, leafDomain.map((l) => l.getValue(colId)))
  }

  let counter = 0
  // Build one grouping level: bucket `rows` by groupIds[level], recurse for
  // deeper levels, attach leaf children at the last level.
  const buildLevel = (rows: Row<T>[], level: number, depth: number): Row<T>[] => {
    const colId = groupIds[level]!
    const def = byId.get(colId)
    const buckets = new Map<unknown, Row<T>[]>()
    const orderedKeys: unknown[] = []
    for (const row of rows) {
      const value = groupingValueOf(row, def, colId)
      if (!buckets.has(value)) {
        buckets.set(value, [])
        orderedKeys.push(value)
      }
      buckets.get(value)!.push(row)
    }

    const out: Row<T>[] = []
    for (const value of orderedKeys) {
      const memberLeaves = buckets.get(value)!
      const children =
        level + 1 < groupIds.length
          ? buildLevel(memberLeaves, level + 1, depth + 1)
          : memberLeaves
      out.push(
        makeGroupRow<T>({
          groupColId: colId,
          groupingValue: value,
          subRows: children,
          leafRows: memberLeaves,
          index: counter++,
          depth,
          bindings: inputs.bindings,
          aggregate: (cid) => aggregateOver(memberLeaves, cid),
          // Sibling band = the group rows at THIS level (posinset/setsize). Late-
          // bound so each group can reference the array still being assembled.
          siblings: () => out,
        }),
      )
    }
    return out
  }

  const groupRows = buildLevel(leaves, 0, 0)
  return flattenGroups(groupRows, expanded)
}

/** Depth-first flatten of synthetic group rows respecting expansion. */
function flattenGroups<T>(groupRows: Row<T>[], expanded: ExpandedState): Row<T>[] {
  const out: Row<T>[] = []
  const walk = (rows: Row<T>[]): void => {
    for (const row of rows) {
      out.push(row)
      if (row.subRows.length > 0 && isExpanded(expanded, row.id)) {
        if (row.subRows.some((c) => c.getIsGrouped())) walk(row.subRows)
        else for (const leaf of row.subRows) out.push(leaf)
      }
    }
  }
  walk(groupRows)
  return out
}

// ── The grouped/expansion stage (replaces the minimal Slice-2 group stage) ────

export interface GroupStageInputs<T> {
  /** Sorted leaf rows (or sorted top-level tree rows). */
  sortedRows: Derived<Row<T>[]>
  grouping: Signal<string[]>
  expanded: Signal<ExpandedState>
  sorting: Signal<SortState[]>
  columns: ColumnDef<T>[]
  bindings: RowBindings<T>
  /** True when the rows are a tree (getSubRows) — flatten the tree instead. */
  isTree: boolean
  manualGrouping: boolean
}

/**
 * `prePaginationRows`: when `grouping` is non-empty, bucket into synthetic group
 * rows; when the data is a tree, flatten the tree depth-first; otherwise pass
 * the sorted rows through. Manual grouping skips bucketing (server pre-grouped).
 */
export function buildGroupStage<T>(inputs: GroupStageInputs<T>): Derived<Row<T>[]> {
  return derived<Row<T>[]>(() => {
    const rows = inputs.sortedRows()
    const groupIds = inputs.grouping()
    const expanded = inputs.expanded()

    if (!inputs.manualGrouping && groupIds.length > 0) {
      return buildGroupedRows(rows, groupIds, expanded, {
        columns: inputs.columns,
        bindings: inputs.bindings,
      })
    }
    if (inputs.isTree) {
      return flattenTree(rows, expanded, inputs.sorting())
    }
    return rows
  })
}

// ── Footer (grand-total) model — computed over filteredRows ───────────────────

/**
 * Build the per-column footer model: one `HeaderGroup` whose headers carry the
 * column ids; the grand total for an aggregating column is read via
 * `column.getFooterAggregate()` (wired in the ColumnApi). The footer aggregate
 * domain is the table's `filteredRows` — NOT a single page — so the tfoot
 * reflects the whole filtered set.
 */
export function footerAggregateFor<T>(
  def: ColumnDef<T> | undefined,
  filteredRows: Row<T>[],
): CellValue {
  if (!def || !hasAggregation(def)) return undefined
  const agg = resolveAggregator(def)
  if (!agg) return undefined
  return agg(def.id, filteredRows.map((r) => r.getValue(def.id)))
}

/** Build the footer header-group model (the real tfoot data). Headers reuse the
 *  table's existing per-column `ColumnApi` (one source of truth — its
 *  `getFooterAggregate()` is the grand total over `filteredRows`). */
export function buildFooterGroups<T>(
  columns: ColumnDef<T>[],
  getColumnApi: (id: string) => import('./types').ColumnApi<T> | undefined,
): Derived<HeaderGroup<T>[]> {
  return derived<HeaderGroup<T>[]>(() => {
    const leaves = columns.filter(
      (c) => !('columns' in c && (c as { columns?: unknown }).columns != null),
    )
    const headers = leaves
      .map((def) => getColumnApi(def.id))
      .filter((c): c is import('./types').ColumnApi<T> => c != null)
      .map((column) => {
        const header: import('./types').Header<T> = {
          id: `footer:${column.id}`,
          column,
          colSpan: 1,
          rowSpan: 1,
          depth: 0,
          isPlaceholder: false,
          getContext: () => ({ column, table: undefined }),
          subHeaders: [],
          getLeafHeaders: () => [header],
        }
        return header
      })
    return [{ id: 'footer', depth: 0, headers }]
  })
}
