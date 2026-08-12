// sail-tree — engine-config builder (the REUSED createTable() options, no createTree()).
//
// A hierarchy is the Table engine with `getSubRows` + `expanded` turned on —
// NOT a parallel engine. This pure helper produces the `TableOptions<T>` both
// `sail-tree` (navMode:'row', label-only) and `sail-tree-grid` (navMode:'cell',
// real columns) feed to `createTable`. `getSubRows` defaults to `r => r.children`
// for the `TreeNode` floor shape; `getRowId` is REQUIRED by the contract.
//
// PURE TypeScript — imports the engine TYPES only; no DOM, no SDK.
//
// Authority: build-spec §"Backbone #1 — createTable() with hierarchy turned on",
// §"Controller shape", Slice 1 + Slice 3.

import type { TreeNode } from './tree-node'
import type { ColumnDef, ExpandedState } from '../sail-table/engine/types'
import type { TableOptions } from '../sail-table/engine/create-table'

export interface TreeEngineConfig<T extends TreeNode> {
  data: TableOptions<T>['data']
  columns?: ColumnDef<T>[]
  navMode?: 'row' | 'cell'
  selectable?: boolean
  multiSelect?: boolean
  expanded?: ExpandedState
  getRowId?: (row: T, index: number) => string
  getSubRows?: (row: T) => T[] | undefined
}

/** Build the `createTable` options for a tree/tree-grid. The hierarchy is
 *  configured, not coded: `getSubRows` + `enableExpanding` + `filterFromLeafRows`. */
export function treeTableOptions<T extends TreeNode>(
  config: TreeEngineConfig<T>,
): TableOptions<T> {
  return {
    data: config.data,
    columns: config.columns ?? [],
    getRowId: config.getRowId ?? ((r) => r.id),
    getSubRows: config.getSubRows ?? ((r) => r.children as T[] | undefined),
    enableExpanding: true,
    enableRowSelection: config.selectable ?? false,
    filterFromLeafRows: true,
    navMode: config.navMode ?? 'row',
    initialState: { expanded: config.expanded ?? {} },
  }
}
