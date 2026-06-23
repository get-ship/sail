// <sui-tree-grid> controller — the SAME engine-backed flattened tree, rendered
// through Table's column/cell machinery (role=treegrid, navMode:'cell').
//
// REUSES createTable (getSubRows+expanded) + createVirtualizer; NO createTree().
// Differs from `sui-tree` only in navMode/role/render: it reads the same
// flattened `tree.rows()` but maps each row's `getVisibleCells()` into cells
// (the disclosure caret + indent land in the FIRST visible column). Columns
// arrive on the live `Table<T>` via the `prop:tree` identity host property; a
// `nodes`/`columns` JSON pair is the SSR fallback.
//
// Authority: build-spec §"The view", §"sui-tree-grid", Slice 5; behavioral #8.

import { controller, signal, derived, ref } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import { createVirtualizer, type Virtual } from '@ship/sui-runtime/virtual'
import { createTable, type Table } from '../sui-table/engine/create-table'
import type { TreeNode } from '../sui-tree-view/sui-tree-view.controller'
import { parseJsonColumns, parseJsonNodes } from '../sui-tree/decode'
import { treeTableOptions } from '../sui-tree/sui-tree.engine'
import { treeHeaderViews, windowTreeRows, type TreeHeaderView } from '../sui-tree/sui-tree.view'
import { wireTreeInteractions } from '../sui-tree/sui-tree.interactions'
import { wireGridColumns } from './sui-tree-grid.columns'
import type { Scope } from './sui-tree-grid.sui'

type AnyTreeNode = TreeNode & Record<string, unknown>

const INDENT_STEP = 16
const ROW_HEIGHT = 32

function emptyTree(): Table<AnyTreeNode> {
  return createTable<AnyTreeNode>(treeTableOptions<AnyTreeNode>({ data: [], navMode: 'cell' }))
}

export default controller({
  // `nodes`/`columns` (string) + `selectable` (enum) are read imperatively from
  // the host — they are not part of the compiler-emitted `Scope`.
  scope: (_ctx: ViewScope<Scope>) => {
    const treeSig = signal<Table<AnyTreeNode> | null>(null)
    const viewportEl = ref<HTMLDivElement>(null)

    const EMPTY = emptyTree()
    const tree = derived<Table<AnyTreeNode>>(() => treeSig() ?? EMPTY)

    const virt: Virtual = createVirtualizer({
      count: () => tree().rows().length,
      estimateSize: (_index) => ROW_HEIGHT,
      getScrollElement: () => viewportEl(),
      overscan: 8,
      getItemKey: (i) => tree().rows()[i]?.id ?? i,
    })

    const headers = derived<TreeHeaderView[]>(() => treeHeaderViews(tree()))
    // The grid-template track string off the resolved per-column sizes — reflects
    // a resize (setColumnSizing) live so the rendered column widens. Empty cols
    // fall back to the equal-fraction default the CSS seeds.
    const gridTemplate = derived<string>(() => {
      const hs = headers()
      if (hs.length === 0) return 'minmax(80px, 1fr)'
      return hs.map((h) => h.size).join(' ')
    })
    const windowItems = windowTreeRows<AnyTreeNode>(tree, virt.virtualItems, () => INDENT_STEP)
    const totalSize = virt.totalSize
    const rowCount = derived<number>(() => tree().rowCount())

    const activeRowId = derived<string | null>(() => {
      const active = tree().nav.active().rowId
      if (active != null) return active
      return tree().rows()[0]?.id ?? null
    })

    const isEmptyInitial = derived<boolean>(
      () => tree().rows().length === 0 && !tree().hasActiveFilters(),
    )
    const isEmptyFiltered = derived<boolean>(
      () => tree().rows().length === 0 && tree().hasActiveFilters(),
    )

    return {
      treeSig,
      viewportEl,
      tree,
      virt,
      headers,
      gridTemplate,
      windowItems,
      totalSize,
      rowCount,
      activeRowId,
      isEmptyInitial,
      isEmptyFiltered,
    }
  },

  mount(state, host) {
    Object.defineProperty(host, 'tree', {
      configurable: true,
      get: () => state.tree,
      set: (t: Table<AnyTreeNode> | null) => {
        state.treeSig = t
      },
    })

    // SSR fallback: build an engine from `nodes` + `columns` JSON attributes.
    if (state.treeSig == null) {
      const rawNodes = (host as { nodes?: string }).nodes ?? host.getAttribute('nodes') ?? ''
      const rawCols = (host as { columns?: string }).columns ?? host.getAttribute('columns') ?? ''
      const parsedNodes = parseJsonNodes(rawNodes)
      const parsedCols = parseJsonColumns(rawCols)
      if (parsedNodes.length > 0 && parsedCols.length > 0) {
        const selAttr = host.getAttribute('selectable') ?? 'none'
        state.treeSig = createTable<AnyTreeNode>(
          treeTableOptions<AnyTreeNode>({
            data: parsedNodes as AnyTreeNode[],
            columns: parsedCols as never,
            navMode: 'cell',
            selectable: selAttr !== 'none',
            multiSelect: selAttr === 'multi',
          }),
        )
      }
    }

    const stop = wireTreeInteractions(host, () => state.tree, {
      expandOnSelect: () => host.hasAttribute('expand-on-select'),
      selectable: () => (host.getAttribute('selectable') ?? 'none') !== 'none',
      multi: () => (host.getAttribute('selectable') ?? 'none') === 'multi',
      navRoleSelector: '[role="treegrid"]',
      getWindow: () => state.$signals.windowItems(),
    })

    // The ORTHOGONAL column lane: header sort/hide + a pointer-drag resize route
    // through the REUSED engine's column mutators (no tree-specific column code).
    const stopColumns = wireGridColumns(host, () => state.tree)

    return () => {
      stop()
      stopColumns()
      state.virt.dispose()
    }
  },
})
