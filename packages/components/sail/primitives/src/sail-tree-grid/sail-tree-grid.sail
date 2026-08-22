<!--
  <sail-tree-grid> — role=treegrid over createTable (getSubRows+expanded).
  Live Table via `tree` identity; nodes/columns JSON is SSR fallback.
  Geometry via use:virtualRow / virtualSpacer / indent / gridTracks actions.
-->

<scope>
  props: {
    nodes = ''
    columns = ''
    label = ''
    selectable: 'none' | 'single' | 'multi' = 'none'
    'expand-on-select' = false
    size: 'md' | 'sm' | 'lg' = 'md'
    loading = false
    error = ''
  }
</scope>

<controller>
// createTable navMode:cell + virtualizer; column wire in sail-tree-grid.columns.

import { createTable, type Table } from '../sail-table/engine/create-table'
import type { TreeNode } from '../sail-tree/tree-node'
import type { Action, ControllerScope } from '@ship/sail-runtime'
import type { Virtual } from '@ship/sail-runtime/virtual'
import { parseJsonColumns, parseJsonNodes } from '../sail-tree/decode'
import { treeTableOptions } from '../sail-tree/sail-tree.engine'
import { treeHeaderViews, windowTreeRows, type TreeHeaderView } from '../sail-tree/sail-tree.view'
import { wireTreeInteractions } from '../sail-tree/sail-tree.interactions'
import { wireGridColumns } from './sail-tree-grid.columns'
import type { Scope } from './sail-tree-grid.sail'

type AnyTreeNode = TreeNode & Record<string, unknown>

const INDENT_STEP = 16
const ROW_HEIGHT = 32

function emptyTree(): Table<AnyTreeNode> {
  return createTable<AnyTreeNode>(treeTableOptions<AnyTreeNode>({ data: [], navMode: 'cell' }))
}

export default controller((_ctx: ControllerScope<Scope>) => {
    const treeSig = signal<Table<AnyTreeNode> | null>(null)
    const viewportEl = signal<HostBag | null>(null)

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
    // fall back to the equal-fraction default the CSS seeds. The number-string
    // crosses JS through `use:gridTracks`, never `style:`.
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

    mount((host) => {
      // Capture pre-connect `tree={table}` identity (bindDynamic may fire first).
      const preTree = (host as { tree?: Table<AnyTreeNode> | null }).tree
      if (preTree != null) treeSig.set(preTree)
      defineHostProperty(asSailHost(host) as HostBag, 'tree', {
        get: () => tree(),
        set: (t: Table<AnyTreeNode> | null) => {
          treeSig.set(t)
        },
      })

      // SSR fallback: build an engine from `nodes` + `columns` JSON attributes.
      if (treeSig() == null) {
        const rawNodes = (host as { nodes?: string }).nodes ?? hostGetAttr(host, 'nodes') ?? ''
        const rawCols = (host as { columns?: string }).columns ?? hostGetAttr(host, 'columns') ?? ''
        const parsedNodes = parseJsonNodes(rawNodes)
        const parsedCols = parseJsonColumns(rawCols)
        if (parsedNodes.length > 0 && parsedCols.length > 0) {
          const selAttr = hostGetAttr(host, 'selectable') ?? 'none'
          treeSig.set(
            createTable<AnyTreeNode>(
              treeTableOptions<AnyTreeNode>({
                data: parsedNodes as AnyTreeNode[],
                columns: parsedCols as never,
                navMode: 'cell',
                selectable: selAttr !== 'none',
                multiSelect: selAttr === 'multi',
              }),
            ),
          )
        }
      }

      const stop = wireTreeInteractions(host, () => tree(), {
        expandOnSelect: () => hostHasAttr(host, 'expand-on-select'),
        selectable: () => (hostGetAttr(host, 'selectable') ?? 'none') !== 'none',
        multi: () => (hostGetAttr(host, 'selectable') ?? 'none') === 'multi',
        navRoleSelector: '[role="treegrid"]',
        getWindow: () => windowItems(),
      })

      // The ORTHOGONAL column lane: header sort/hide + a pointer-drag resize route
      // through the REUSED engine's column mutators (no tree-specific column code).
      const stopColumns = wireGridColumns(host, () => tree())

      return () => {
        stop()
        stopColumns()
        virt.dispose()
      }
    })

    const captureViewport: Action<void> = (node) => {
      viewportEl.set(node)
      return { destroy: () => viewportEl.set(null) }
    }
    return {
      treeSig,
      captureViewport,
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
  })

</controller>
<if {error() !== ''}>
<div role="alert" style({ display(block).size({ w: 100% }).fg(foreground) })>
  <error-state>{error()}</error-state>
</div>
{else-if {isEmptyInitial()} => <div role="status">
  <slot name="empty"><empty-state></empty-state></slot>
</div>}{else-if {isEmptyFiltered()} => <div role="status">
  <slot name="empty">
    <empty-state>
      <button slot="action" type="button" data-action="clear-filters">Clear filters</button>
    </empty-state>
  </slot>
</div>}{else => <column role="treegrid" gap="0" use:gridTracks={gridTemplate()} aria-label={label() || null} aria-rowcount={rowCount()} aria-multiselectable={selectable() === 'multi' ? 'true' : null} style({ size({ w: 100% }).overflow({ all: hidden }).bg(surface).border({ color: border, width: 1 }).radius(lg) })>  <span style({ display(contents).type({ family: body, size: density }) })>    <grid tracks="--sail-grid-template" gap="0" role="row" style({ position({ scheme: sticky, t: 0 }).position({ z: sticky }).bg(surface).border({ edge: b, color: border, width: 1 }) })>
    <for {headers() as h} key={h.id}>
      <row gap="1" align="center" role="columnheader" data-col-id={h.id} data-sortable={h.canSort ? 'true' : null} data-sorted={h.sorted || null} aria-sort={h.canSort ? h.ariaSort : null} style({ position(relative).pad({ x: density-row-px, y: density-row-py }).fg(foreground).border({ edge: b, color: border-subtle, width: 1 }), when: { sortable: cursor(pointer) } })>        <span style({ display(contents).type({ size: xs, weight: semibold, tracking: wide, case: uppercase, wrap: nowrap }) })>          <span style({ grow(1).size({ min-w: 0 }).type({ wrap: truncate }) })>{h.label}</span>
        <if {h.sorted === 'asc'}>
          <span aria-hidden="true" style({ fg(primary) })><icon name="chevron-up" size="xs"></icon></span>
        {else-if {h.sorted === 'desc'} => <span aria-hidden="true" style({ fg(primary) })><icon name="chevron-down" size="xs"></icon></span>}</if>
        <if {h.canHide}>
          <button variant="ghost" size="xs" type="button" data-action="column-menu" data-col-id={h.id} aria-label="Column options" style({ size({ w: density-icon, h: density-icon }).pad(0).radius(sm) })>
            <icon name="more-vertical" size="xs"></icon>
          </button>
        </if>
        <if {h.canResize}>
          <span data-action="resize" data-col-id={h.id} aria-hidden="true" style({ position({ scheme: absolute, t: 0, b: 0, r: 0 }).size({ w: 1 }).cursor(col-resize), when: { hover: bg(primary) } })></span>
        </if>
        </span>
</row>
    </for>
  </grid>
  <div use:captureViewport style({ position(relative).size({ w: 100%, h: 100%, min-h: 0 }).overflow({ y: auto }) })>
      <for {windowItems() as w} key={`${w.key}|${w.cellsKey}`}>
        <grid tracks="--sail-grid-template" gap="0" role="row" data-node-id={w.id} aria-level={w.level} aria-posinset={w.posInSet} aria-setsize={w.setSize} aria-rowindex={w.index + 1} aria-expanded={w.canExpand ? (w.expanded ? 'true' : 'false') : null} aria-selected={selectable() !== 'none' ? (w.selected ? 'true' : 'false') : null} data-can-expand={w.canExpand ? 'true' : null} data-expanded={w.expanded ? 'true' : null} data-selected={w.selected ? 'true' : null} tabindex={w.id === activeRowId() ? '0' : '-1'} use:virtualRow={w.start} style({ position({ scheme: absolute, l: 0 }).size({ w: 100% }).group().translateY(bind(rowY)).cursor(pointer), when: { hover: bg(hover-bg), selected: bg(selected-bg), focus: border({ kind: outline, color: ring }) } })>
          <span aria-hidden="true" style({ position({ scheme: absolute, t: 1, b: 1, l: 0 }).size({ w: half }).radius(full).bg(primary).filter({ opacity: 0 }), when: { groupSelected: filter({ opacity: 100 }) } })></span>
          <for {w.cells as c} key={c.columnId}>
            <if {c.first}>
              <row role="gridcell" use:cellPad={w.indent} style({ pad({ x: density-row-px, y: density-row-py, l: bind(cellPs) }).size({ min-w: 0 }).border({ edge: b, color: border-subtle, width: 1 }).fg(foreground) })>
                <if {w.canExpand}>
                  <span aria-hidden="true" data-tree-caret style({ size({ w: density-icon, h: density-icon, min-w: density-icon }).fg(muted-foreground), when: { groupExpanded: rotate(90) } })>
                    <icon name="chevron-right" size="xs"></icon>
                  </span>
                </if>
                <span style({ grow(1).size({ min-w: 0 }).type({ wrap: truncate }) })>{c.value}</span>
              </row>
            {else => <row role="gridcell" style({ pad({ x: density-row-px, y: density-row-py }).size({ min-w: 0 }).border({ edge: b, color: border-subtle, width: 1 }).fg(foreground) })>
                <span style({ grow(1).size({ min-w: 0 }).type({ wrap: truncate }) })>{c.value}</span>
              </row>}</if>
          </for>
        </grid>
      </for>
    </div>
  </span>
</column>}</if>
