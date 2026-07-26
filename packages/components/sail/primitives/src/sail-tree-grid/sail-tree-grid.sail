<!--
  <sail-tree-grid> — engine-backed hierarchical TREE-GRID (role=treegrid).

  The SAME flattened depth-first `tree.rows()` from the REUSED Table engine
  (createTable with getSubRows+expanded — NO createTree()), rendered through
  the Table view's COLUMN/CELL machinery (row.getVisibleCells()), with the
  disclosure caret + indent in the FIRST visible column. navMode:'cell' (vs
  'row' for sail-tree); role=treegrid (vs role=tree). The live Table<T>
  (carrying the column defs) arrives via tree IDENTITY binding; a
  nodes+columns JSON pair is the SSR fallback.

  Density is the platform axis (<html data-density>). NO density/dense prop.

  Events (host.emit): sail-toggle {id, expanded}, sail-expand-change {expanded},
  sail-select {id}, sail-node-select {selected, selectedIds}, sail-node-click
  {id, depth, index}.

  Anatomy (anatomy = the dom structure the controller renders into shadow;
  every named part has a `part=` attribute the test selectors pin to, and
  every styling class below is a LINT-DISALLOWED public hook — tests anchor
  on `part=` / `data-*` / `role=`, not on class names):

    <sail-tree-grid>            shadow root
      ├─ .grid    part=grid    <column>, role=treegrid (the tablegrid)
      │   ├─ .head part=thead  sticky header row, role=row
      │   │   └─ .col-head    part=columnheader, role=columnheader (×N)
      │   │       ├─ .col-label            (text)
      │   │       ├─ .sort-glyph  sail-icon chevron-up|down (when sorted)
      │   │       ├─ <button>   data-action=column-menu, ghost xs (when canHide)
      │   │       └─ .resize-handle part=resize-handle (when canResize)
      │   └─ .viewport part=viewport
      │       └─ .spacer part=spacer (the virtualizer total-size ruler)
      │           └─ .row     part=row, role=row (per visible row)
      │               └─ .cell part=cell, role=gridcell (per visible column)
      │                   ├─ .caret  part=caret, sail-icon chevron-right
      │                   └─ .cell-label (text)
      ├─ .tree-error part=error (sail-error-state inside, when error attr)
      └─ .tree-empty part=empty (sail-empty-state inside, when no rows)

  Columns arrive on the live `Table<T>` via the `tree` identity host
  property; `nodes`/`columns` JSON pair is the SSR fallback.

  Tests pin to `part=` / `data-*` / `role=` — never to a class name.

  Runtime geometry: the row's vertical offset, per-row indent, the
  virtualizer total-size ruler, and the grid-template tracks all read
  custom properties written by named actions (`use:virtualRow`,
  `use:virtualSpacer`, `use:indent`, `use:gridTracks`). No `style:`
  attributes — runtime geometry is the seam the actions own.
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
// <sail-tree-grid> controller — the SAME engine-backed flattened tree, rendered
// through Table's column/cell machinery (role=treegrid, navMode:'cell').
//
// REUSES createTable (getSubRows+expanded) + createVirtualizer; NO createTree().
// Differs from `sail-tree` only in navMode/role/render: it reads the same
// flattened `tree.rows()` but maps each row's `getVisibleCells()` into cells
// (the disclosure caret + indent land in the FIRST visible column). Columns
// arrive on the live `Table<T>` via the `tree` identity host property; a
// `nodes`/`columns` JSON pair is the SSR fallback.
//
// Runtime geometry (track string, row offset, indent, spacer height) leaves
// the controller as typed numbers, then crosses JS into `--sail-*` custom
// properties via the runtime actions. The CSS / sx half never sees an
// untyped value — every property's NAME is in the template, only the
// NUMBER walks.
//
// Authority: build-spec §"The view", §"sail-tree-grid", Slice 5; behavioral #8.

import { controller, signal, derived} from '@ship/sail-runtime'
import type { ControllerScope } from '@ship/sail-runtime'
import { createVirtualizer, type Virtual } from '@ship/sail-runtime/virtual'
import { createTable, type Table } from '../sail-table/engine/create-table'
import type { TreeNode } from '../sail-tree-view/sail-tree-view.controller'
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
    const viewportEl = signal<HTMLDivElement | null>(null)

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
      Object.defineProperty(host, 'tree', {
        configurable: true,
        get: () => tree(),
        set: (t: Table<AnyTreeNode> | null) => {
          treeSig.set(t)
        },
      })

      // SSR fallback: build an engine from `nodes` + `columns` JSON attributes.
      if (treeSig() == null) {
        const rawNodes = (host as { nodes?: string }).nodes ?? host.getAttribute('nodes') ?? ''
        const rawCols = (host as { columns?: string }).columns ?? host.getAttribute('columns') ?? ''
        const parsedNodes = parseJsonNodes(rawNodes)
        const parsedCols = parseJsonColumns(rawCols)
        if (parsedNodes.length > 0 && parsedCols.length > 0) {
          const selAttr = host.getAttribute('selectable') ?? 'none'
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
        expandOnSelect: () => host.hasAttribute('expand-on-select'),
        selectable: () => (host.getAttribute('selectable') ?? 'none') !== 'none',
        multi: () => (host.getAttribute('selectable') ?? 'none') === 'multi',
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
  })

</controller>
{#if error() !== ''}
<div sx={style.display(block).w(full).fg(foreground)} role="alert">
  <error-state>{error()}</error-state>
</div>
{:else if isEmptyInitial()}
<div role="status">
  <slot name="empty"><empty-state></empty-state></slot>
</div>
{:else if isEmptyFiltered()}
<div role="status">
  <slot name="empty">
    <empty-state>
      <button slot="action" type="button" data-action="clear-filters">Clear filters</button>
    </empty-state>
  </slot>
</div>
{:else}
<column role="treegrid" gap="0" use:gridTracks={gridTemplate()} aria-label={label() || null} aria-rowcount={rowCount()} aria-multiselectable={selectable() === 'multi' ? 'true' : null} sx={style.w(full).overflow(hidden).bg(surface-1).borderWidth(1).border(border).radius(lg).fontFamily(body).fontSize(density)}>
  <grid tracks="--sail-grid-template" gap="0" role="row" sx={style.position(sticky).top(0).z(sticky).bg(surface-2).borderBWidth(1).borderB(border)}>
    {#for h of headers() key={h.id}}
      <row gap="1" align="center" role="columnheader" data-col-id={h.id} data-sortable={h.canSort ? 'true' : null} data-sorted={h.sorted || null} aria-sort={h.canSort ? h.ariaSort : null} sx={style.position(relative).padY(density-row-py).padX(density-row-px).fontWeight(semibold).fontSize(xs).textCase(uppercase).tracking(wide).fg(foreground).borderBWidth(1).borderB(border-subtle).whitespace(nowrap).when({ sortable: style.cursor(pointer) })}>
        <span grow min-w="0" sx={style.truncate()}>{h.label}</span>
        {#if h.sorted === 'asc'}
          <span aria-hidden="true" sx={style.fg(primary)}><icon name="chevron-up" size="xs"></icon></span>
        {:else if h.sorted === 'desc'}
          <span aria-hidden="true" sx={style.fg(primary)}><icon name="chevron-down" size="xs"></icon></span>
        {/if}
        {#if h.canHide}
          <button variant="ghost" size="xs" type="button" data-action="column-menu" data-col-id={h.id} aria-label="Column options" sx={style.w(density-icon).h(density-icon).pad(0).radius(sm)}>
            <icon name="more-vertical" size="xs"></icon>
          </button>
        {/if}
        {#if h.canResize}
          <span data-action="resize" data-col-id={h.id} aria-hidden="true" sx={style.position(absolute).top(0).bottom(0).right(0).w(1).cursor(col-resize).when({ hover: style.bg(primary) })}></span>
        {/if}
      </row>
    {/for}
  </grid>
  <div bind:this={viewportEl} sx={style.position(relative).w(full).overflowY(auto).maxH(panel-max-h-md)}>
      {#for w of windowItems() key={`${w.key}|${w.cellsKey}`}}
        <grid tracks="--sail-grid-template" gap="0" role="row" data-node-id={w.id} aria-level={w.level} aria-posinset={w.posInSet} aria-setsize={w.setSize} aria-rowindex={w.index + 1} aria-expanded={w.canExpand ? (w.expanded ? 'true' : 'false') : null} aria-selected={selectable() !== 'none' ? (w.selected ? 'true' : 'false') : null} data-can-expand={w.canExpand ? 'true' : null} data-expanded={w.expanded ? 'true' : null} data-selected={w.selected ? 'true' : null} tabindex={w.id === activeRowId() ? '0' : '-1'} use:virtualRow={w.start} sx={style.group().position(absolute).left(0).w(full).translateY(bind(rowY)).cursor(pointer).when({ hover: style.bg(hover-bg), selected: style.bg(selected-bg), focus: style.outline(ring) })}>
          <span aria-hidden="true" sx={style.position(absolute).left(0).top(1).bottom(1).w(half).radius(full).bg(primary).visibility(hidden).when({ groupSelected: style.visibility(visible) })}></span>
          {#for c of w.cells key={c.columnId}}
            {#if c.first}
              <row role="gridcell" use:cellPad={w.indent} sx={style.padY(density-row-py).padX(density-row-px).borderBWidth(1).borderB(border-subtle).fg(foreground).minW(0).padS(bind(cellPs))}>
                {#if w.canExpand}
                  <span data-action="toggle" aria-hidden="true" sx={style.w(density-icon).h(density-icon).minW(density-icon).fg(muted-foreground).when({ groupExpanded: style.rotate(90) })}>
                    <icon name="chevron-right" size="xs"></icon>
                  </span>
                {/if}
                <span grow min-w="0" sx={style.truncate()}>{c.value}</span>
              </row>
            {:else}
              <row role="gridcell" sx={style.padY(density-row-py).padX(density-row-px).borderBWidth(1).borderB(border-subtle).fg(foreground).minW(0)}>
                <span grow min-w="0" sx={style.truncate()}>{c.value}</span>
              </row>
            {/if}
          {/for}
        </grid>
      {/for}
    </div>
  </div>
</column>
{/if}
