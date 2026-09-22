<!--
  <sail-tree> — engine-backed hierarchical tree (role=tree).
  Reuses Table engine expansion (createTable + getSubRows); no createTree().
  Live Table via `tree` identity; `nodes` JSON is SSR/declarative fallback.
  Visible rows render in document flow (SPEC-TREE-1 path A — no absolute virtualizer).
-->

<scope>
  props: {
    nodes = ''
    label = ''
    selectable: 'none' | 'single' | 'multi' = 'none'
    'expand-on-select' = false
    'show-guides' = true
    size: 'md' | 'sm' | 'lg' = 'md'
    loading = false
    error = ''
  }
</scope>
<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import { createTable, type Table } from '../sail-table/engine/create-table'
import type { TreeNode } from './tree-node'
import { parseJsonNodes } from './decode'
import { treeTableOptions } from './sail-tree.engine'
import { flowTreeRows } from './sail-tree.view'
import { wireTreeInteractions } from './sail-tree.interactions'
import type { Scope } from './sail-tree.sail'

type AnyTreeNode = TreeNode & Record<string, unknown>

/** Fallback indent step (px) when the density channel is unmeasured (jsdom). */
const INDENT_STEP = 16

/** Stable empty engine so the list read never branches on null. */
function emptyTree(): Table<AnyTreeNode> {
  return createTable<AnyTreeNode>(treeTableOptions<AnyTreeNode>({ data: [] }))
}

export default controller((_ctx: ControllerScope<Scope>) => {
    const treeSig = signal<Table<AnyTreeNode> | null>(null)

    const EMPTY = emptyTree()
    const tree = derived<Table<AnyTreeNode>>(() => treeSig() ?? EMPTY)

    const flowItems = flowTreeRows<AnyTreeNode>(tree, () => INDENT_STEP)
    const rowCount = derived<number>(() => tree().rowCount())

    // Roving-tabindex: nav focus, else first visible row.
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
      // Live Table identity on host.tree (not a stringified attribute).
      const preTree = (host as { tree?: Table<AnyTreeNode> | null }).tree
      if (preTree != null) treeSig.set(preTree)
      defineHostProperty(asSailHost(host) as HostBag, 'tree', {
        get: () => tree(),
        set: (t: Table<AnyTreeNode> | null) => {
          treeSig.set(t)
        },
      })

      // SSR fallback: build engine from nodes JSON when no identity assigned.
      if (treeSig() == null) {
        const raw = (host as { nodes?: string }).nodes ?? hostGetAttr(host, 'nodes') ?? ''
        const parsed = parseJsonNodes(raw)
        if (parsed.length > 0) {
          const selAttr = hostGetAttr(host, 'selectable') ?? 'none'
          treeSig.set(
            createTable<AnyTreeNode>(
              treeTableOptions<AnyTreeNode>({
                data: parsed as AnyTreeNode[],
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
        navRoleSelector: '[role="tree"]',
        getWindow: () => flowItems(),
      })

      return () => {
        stop()
      }
    })

    return {
      treeSig,
      tree,
      flowItems,
      rowCount,
      activeRowId,
      isEmptyInitial,
      isEmptyFiltered,
    }
  })

</controller>
<if {error() !== ''}>
<div role="alert">
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
</div>}{else => <div
  role="tree"
  aria-label={label() || null}
  aria-multiselectable={selectable() === 'multi' ? 'true' : null}
  style({ size({ w: 100%, h: 100%, min-h: 0 }).overflow({ all: hidden }).fg(foreground) })
>
  <!-- display(contents) text wrap: no box so scroll child % height resolves on host. -->
  <span style({ display(contents).type({ family: body, size: density }) })>
  <div style({ size({ w: 100%, h: 100%, min-h: 0 }).overflow({ y: auto }) })>
    <for {flowItems() as w} key={w.key}>
      <row
        align="center"
        gap="density"
        nowrap
        role="treeitem"
        data-node-id={w.id}
        aria-level={w.level}
        aria-posinset={w.posInSet}
        aria-setsize={w.setSize}
        aria-rowindex={w.index + 1}
        aria-expanded={w.canExpand ? (w.expanded ? 'true' : 'false') : null}
        aria-selected={selectable() !== 'none' ? (w.selected ? 'true' : 'false') : null}
        data-can-expand={w.canExpand ? 'true' : null}
        data-expanded={w.expanded ? 'true' : null}
        data-selected={w.selected ? 'true' : null}
        data-pending={w.pending ? 'true' : null}
        aria-busy={w.pending ? 'true' : null}
        tabindex={w.id === activeRowId() ? '0' : '-1'}
        style({ size({ w: 100% }).pad({ y: density-row-py, l: bind(w.ps), r: density-row-px }).group().radius(sm).cursor(pointer).fg(foreground), when: { hover: bg(hover-bg), selected: bg(selected-bg), busy: fg(muted-foreground).cursor(default), focus: border({ kind: outline, color: ring }) } })
      >
        <span style({ display(contents), when: { selected: type({ weight: medium }), busy: type({ italic: true }) } })>
        <!-- Chevron paint only; expand is the row (aria-expanded / data-expanded). -->
        <if {w.canExpand}>
          <row
            data-tree-caret
            aria-hidden="true"
            align="center"
            justify="center"
            style({ size({ w: 8, h: 8, min-w: 8 }).fg(muted-foreground), when: { groupExpanded: rotate(90) } })
          >
            <icon name="chevron-right" size="sm"></icon>
          </row>
        {else => <span aria-hidden="true" style({ size({ w: density-icon, min-w: density-icon }) })></span>}</if>

        <if {selectable() === 'multi'}>
          <span>
            <checkbox
       
       size="sm"
       aria-label={`Select ${w.label}`}
       checked={w.checked}
       indeterminate={w.someSelected}
      />
          </span>
        </if>
        <if {w.icon !== ''}>
          <icon
            name={w.icon}
            size="sm"
            style({ fg(muted-foreground), when: { groupSelected: fg(foreground) } })
          ></icon>
        {else => <slot name="node-icon"></slot>}</if>
        <span style({ grow(1).size({ min-w: 0 }).type({ wrap: truncate }) })>{w.label}</span>
        <span style({ margin({ l: auto }).type({ size: xs }) })>
          <if {w.trailing !== ''}>
            <if {w.trailing === '●'}>
              <span aria-label="Unsaved" style({ fg(warning) })>●</span>
            {else => <span aria-label={`Meta ${w.trailing}`} style({ fg(muted-foreground) })>{w.trailing}</span>}</if>
          </if>
          <span style({ fg(muted-foreground) })><slot name="node-trailing"></slot></span>
        </span>
        </span>
      </row>
    </for>
  </div>
  </span>
</div>}</if>
