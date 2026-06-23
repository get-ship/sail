// <sui-tree> controller — engine-backed, virtualized hierarchical tree.
//
// REUSES the Table engine (createTable with getSubRows+expanded) and the shipped
// createVirtualizer; there is NO createTree(). The live `Table<T>` arrives via
// the `prop:tree` IDENTITY host property (never serialized); a `nodes` JSON
// attribute is the SSR/declarative fallback. The windowed body renders
// `tree.rows()` (flattened, depth-first, expansion-respecting) through a
// createVirtualizer viewport, one [role=treeitem] per VISIBLE node.
//
// Templates declare, controllers do: mutations + emit live in mount's delegated
// listeners; aria/level/posinset/setsize are derived in the TEMPLATE from the
// TreeRowView this builds.
//
// Authority: build-spec §"Controller shape", §"Backbone reuse", Slices 2-4.

import { controller, signal, derived, ref } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import { createVirtualizer, type Virtual } from '@ship/sui-runtime/virtual'
import { createTable, type Table } from '../sui-table/engine/create-table'
import type { TreeNode } from '../sui-tree-view/sui-tree-view.controller'
import { parseJsonNodes } from './decode'
import { treeTableOptions } from './sui-tree.engine'
import { windowTreeRows } from './sui-tree.view'
import { wireTreeInteractions } from './sui-tree.interactions'
import type { Scope } from './sui-tree.sui'

type AnyTreeNode = TreeNode & Record<string, unknown>

/** Fallback indent step (px) when the density channel is unmeasured (jsdom). */
const INDENT_STEP = 16
/** Row-height estimate seed for the virtualizer (density-derived at runtime). */
const ROW_HEIGHT = 32

/** A stable empty engine so the windowed read never branches on null. */
function emptyTree(): Table<AnyTreeNode> {
  return createTable<AnyTreeNode>(treeTableOptions<AnyTreeNode>({ data: [] }))
}

export default controller({
  // `nodes` (string) + `selectable` (enum) are read imperatively from the host
  // in mount/interactions — they are not part of the compiler-emitted `Scope`.
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

    const windowItems = windowTreeRows<AnyTreeNode>(tree, virt.virtualItems, () => INDENT_STEP)
    const totalSize = virt.totalSize
    const rowCount = derived<number>(() => tree().rowCount())

    // Roving-tabindex: the active row is nav's focus, defaulting to the first
    // visible row so exactly one treeitem is focusable before any key.
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
      windowItems,
      totalSize,
      rowCount,
      activeRowId,
      isEmptyInitial,
      isEmptyFiltered,
    }
  },

  mount(state, host) {
    // ── prop:tree IDENTITY wiring ────────────────────────────────────────────
    // The engine is a LIVE JS reference assigned to the host `tree` property
    // (never a stringified attribute). Route it into the scope signal so the
    // windowed render reacts.
    Object.defineProperty(host, 'tree', {
      configurable: true,
      get: () => state.tree,
      set: (t: Table<AnyTreeNode> | null) => {
        state.treeSig = t
      },
    })

    // SSR fallback: build an engine from the `nodes` JSON attribute when no
    // identity engine was assigned (the sui-combobox pattern).
    if (state.treeSig == null) {
      const raw = (host as { nodes?: string }).nodes ?? host.getAttribute('nodes') ?? ''
      const parsed = parseJsonNodes(raw)
      if (parsed.length > 0) {
        const selAttr = host.getAttribute('selectable') ?? 'none'
        state.treeSig = createTable<AnyTreeNode>(
          treeTableOptions<AnyTreeNode>({
            data: parsed as AnyTreeNode[],
            selectable: selAttr !== 'none',
            multiSelect: selAttr === 'multi',
          }),
        )
      }
    }

    // ── Delegated caret/row interactions + deduped engine-state emission ──────
    const stop = wireTreeInteractions(host, () => state.tree, {
      expandOnSelect: () => host.hasAttribute('expand-on-select'),
      selectable: () => (host.getAttribute('selectable') ?? 'none') !== 'none',
      multi: () => (host.getAttribute('selectable') ?? 'none') === 'multi',
      navRoleSelector: '[role="tree"]',
      getWindow: () => state.$signals.windowItems(),
    })

    return () => {
      stop()
      state.virt.dispose()
    }
  },
})
