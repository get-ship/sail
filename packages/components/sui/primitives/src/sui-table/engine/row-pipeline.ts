// sui-table engine — the row-chain assembler (Backbone #1, Slice 3b).
//
// Composes the one-direction derived row chain so `create-table.ts` stays a thin
// facade. Wires:
//
//   coreRows ─► preColumnFiltered ─► filtered ─► sorted ─► grouped(=prePagination)
//
// For TREE data (`getSubRows`) it swaps the flat column-filter stage for a
// tree-aware filter that surfaces a matching leaf's ancestor chain
// (`filterFromLeafRows`); for grouping it delegates the bucketing/flatten to
// `grouping.ts`. Returns every row-chain derived the facade re-exports plus the
// `expanded`-keyed expansion handle the rows bind to.
//
// PURE TypeScript — imports `derived` from `@ship/sui-runtime` ONLY.
//
// Authority: build-spec §"Signal graph", §Features #5/#13, Slice 3b.

import { derived, type Derived } from '@ship/sui-runtime'
import { buildFilterStages, resolveCellPredicate } from './filtering'
import { buildGroupStage, filterTreeFromLeaves } from './grouping'
import { buildCore, buildSortStage } from './pipeline'
import type { RowBindings } from './row-api'
import type { ColumnDef, ReactiveTableState, Row, TableOptions } from './types'

export interface RowChain<T> {
  coreRows: Derived<Row<T>[]>
  preColumnFilteredRows: Derived<Row<T>[]>
  filteredRows: Derived<Row<T>[]>
  sortedRows: Derived<Row<T>[]>
  prePaginationRows: Derived<Row<T>[]>
}

export interface RowChainInputs<T> {
  options: TableOptions<T>
  state: ReactiveTableState
  bindings: RowBindings<T>
  dataSignal: import('@ship/sui-runtime').Signal<T[]>
}

/** Assemble the full row chain (filter → sort → group/flatten). */
export function buildRowChain<T>(inputs: RowChainInputs<T>): RowChain<T> {
  const { options, state, bindings, dataSignal } = inputs
  const columns = options.columns
  const isTree = typeof options.getSubRows === 'function'
  const manualFiltering = options.manualFiltering === true
  const manualSorting = options.manualSorting === true
  const manualGrouping = options.manualGrouping === true

  const coreInputs: Parameters<typeof buildCore<T>>[0] = {
    data: dataSignal,
    columns,
    getRowId: options.getRowId,
    bindings,
  }
  if (options.getSubRows) coreInputs.getSubRows = options.getSubRows
  const { coreRows } = buildCore<T>(coreInputs)

  const flat = buildFilterStages<T>({
    coreRows,
    columns,
    columnFilters: state.columnFilters,
    globalFilter: state.globalFilter,
    manual: manualFiltering,
  })

  // Tree filter surfaces a matching leaf's ancestors when filterFromLeafRows
  // (default true for trees); else it falls back to the flat per-row filter.
  const filteredRows =
    isTree && options.filterFromLeafRows !== false && !manualFiltering
      ? buildTreeFilter<T>(coreRows, columns, state, flat.preColumnFilteredRows)
      : flat.filteredRows

  // For TREE data the per-level sort happens inside `flattenTree` (a node's
  // children are sorted within their parent); the ROOT order is preserved, so the
  // top-level sort stage is skipped (sorting it would reshuffle the roots on a
  // column sort). Flat tables sort here as before.
  const sortedRows = buildSortStage<T>(filteredRows, state.sorting, manualSorting || isTree)
  const prePaginationRows = buildGroupStage<T>({
    sortedRows,
    grouping: state.grouping,
    expanded: state.expanded,
    sorting: state.sorting,
    columns,
    bindings,
    isTree,
    manualGrouping,
  })

  return {
    coreRows,
    preColumnFilteredRows: flat.preColumnFilteredRows,
    filteredRows,
    sortedRows,
    prePaginationRows,
  }
}

/**
 * A tree-aware `filteredRows`: keep a subtree when the row OR any descendant
 * matches the active column filters, so a matching leaf surfaces with its full
 * ancestor chain. (The global filter already ran in `preColumnFilteredRows`,
 * which for trees is matched per-row; the leaf-surfacing logic here re-checks
 * the WHOLE tree against the global + column predicates so an ancestor is kept.)
 */
function buildTreeFilter<T>(
  coreRows: Derived<Row<T>[]>,
  columns: ColumnDef<T>[],
  state: ReactiveTableState,
  _preColumnFilteredRows: Derived<Row<T>[]>,
): Derived<Row<T>[]> {
  const byId = new Map(columns.map((c) => [c.id, c]))
  const globalCols = columns.filter(
    (c) =>
      'accessorKey' in c &&
      (c as { accessorKey?: unknown }).accessorKey != null &&
      (c as { enableGlobalFilter?: boolean }).enableGlobalFilter !== false,
  )
  return derived<Row<T>[]>(() => {
    const roots = coreRows()
    const q = state.globalFilter()
    const filters = state.columnFilters()
    if ((q == null || q === '') && filters.length === 0) return roots

    const matches = (row: Row<T>): boolean => {
      if (q != null && q !== '') {
        const hit = globalCols.some((c) =>
          String(row.getValue(c.id) ?? '').toLowerCase().includes(q.toLowerCase()),
        )
        if (!hit) return false
      }
      return filters.every((f) => {
        const def = byId.get(f.id)
        if (!def) return true
        return resolveCellPredicate(def, f.value)(row.getValue(f.id), f.value)
      })
    }
    return filterTreeFromLeaves(roots, matches)
  })
}
