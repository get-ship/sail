<!--
  <sail-tree> — engine-backed, virtualized hierarchical tree (role=tree).

  Proves the SHARED-ENGINE REUSE law: a hierarchy is the Table engine's
  expansion model turned on, NOT a parallel engine. There is no createTree().
  The controller calls createTable({ getSubRows, getRowId, columns: [],
  initialState: { expanded } }), reads the flattened depth-first, expansion-
  respecting table.rows(), and renders it WINDOWED through createVirtualizer —
  one [role=treeitem] per VISIBLE node, indented by row.depth.

  The live Table<T> arrives via tree IDENTITY binding (never a serialized
  object attribute); a `nodes` JSON attribute is the SSR/declarative fallback.

  Density is the platform axis (<html data-density>) — rows read the --density-*
  channel. There is NO density/dense prop.

  Events (host.emit): sail-toggle {id, expanded}, sail-expand-change {expanded},
  sail-select {id}, sail-node-select {selected, selectedIds}, sail-node-click
  {id, depth, index}.

  NEW SURFACE (re-port from feat/next, parity-gate pass)
  ─────────────────────────────────────────────────────────────────────────────
  - No tone prop on this component — rule "tone is global now" doesn't apply.
  - The row + its caret are the L1 layout keyword `row` (align/justify
    attributes), not a hand-styled flex `<div>`/`<span>` — display:flex,
    align-items and gap for both now come from the keyword, not authored CSS.
    The row's cursor/radius/ink/hover-bg are sx (`sx={style.radius(sm).cursor(pointer).fg(foreground).when({ hover: style.bg(hover-bg) })}` / `sx={style.fg(muted-foreground)}` on the
    caret) instead of raw `.row`/`.caret` declarations.
  - L1 controls were already framework-composed pre-gate (sail-checkbox,
    sail-icon, sail-empty-state, sail-error-state) — nothing hand-skinned.
  - Dropped a dead line from the style block: a bare (outside any selector,
    so inert in real CSS — a browser's parser discards top-level declarations
    it can't attach to a rule) list of every token this file's raw CSS
    references, each given an EMPTY value (`--font-body: ; --foreground: ;
    …`). Harmless at runtime but pure noise; every name in it is a real,
    already-registered global token, so it worked around nothing. Not in
    feat/next; removed rather than re-ported.
  - Anatomy (light DOM, class-based — part= left on per the wave sweep note):
    .tree (root, role=tree) > .viewport > .spacer > row.row (role=treeitem,
    one per visible node) > row.caret (chevron icon), span.gutter (multi-
    select checkbox), sail-icon.node-icon / slot[name=node-icon], span.label,
    span.trailing (slot[name=node-trailing]). .tree-error / .tree-empty are
    the error/empty-state branches.

  See the second comment below (unchanged from the prior pass) for why the
  remaining `.row`/`.caret`/`.gutter`/`.node-icon`/`.label`/`.trailing` chrome
  stays in the style block rather than sx: it's overwhelmingly descendant
  state (`.row[data-selected] .node-icon`, `.row:not([data-can-expand])
  .caret`, etc.) — sx and `<variants>` both only match HOST attributes, never
  a descendant's data-* state, so there is no closed-vocabulary path to any
  of it. Not a new LANG-GAP; the prior pass already identified and documented
  this residual correctly.
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
// <sail-tree> controller — engine-backed, virtualized hierarchical tree.
//
// REUSES the Table engine (createTable with getSubRows+expanded) and the shipped
// createVirtualizer; there is NO createTree(). The live `Table<T>` arrives via
// the `tree` IDENTITY host property (never serialized); a `nodes` JSON
// attribute is the SSR/declarative fallback. The windowed body renders
// `tree.rows()` (flattened, depth-first, expansion-respecting) through a
// createVirtualizer viewport, one [role=treeitem] per VISIBLE node.
//
// Templates declare, controllers do: mutations + emit live in mount's delegated
// listeners; aria/level/posinset/setsize are derived in the TEMPLATE from the
// TreeRowView this builds.
//
// Authority: build-spec §"Controller shape", §"Backbone reuse", Slices 2-4.

import type { ControllerScope } from '@ship/sail-runtime'
import { createVirtualizer, type Virtual } from '@ship/sail-runtime/virtual'
import { createTable, type Table } from '../sail-table/engine/create-table'
import type { TreeNode } from '../sail-tree-view/sail-tree-view.controller'
import { parseJsonNodes } from './decode'
import { treeTableOptions } from './sail-tree.engine'
import { windowTreeRows } from './sail-tree.view'
import { wireTreeInteractions } from './sail-tree.interactions'
import type { Scope } from './sail-tree.sail'

type AnyTreeNode = TreeNode & Record<string, unknown>

/** Fallback indent step (px) when the density channel is unmeasured (jsdom). */
const INDENT_STEP = 16
/** Row-height estimate seed for the virtualizer (density-derived at runtime). */
const ROW_HEIGHT = 32

/** A stable empty engine so the windowed read never branches on null. */
function emptyTree(): Table<AnyTreeNode> {
  return createTable<AnyTreeNode>(treeTableOptions<AnyTreeNode>({ data: [] }))
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

    mount((host) => {
      // ── tree IDENTITY wiring ────────────────────────────────────────────
      // The engine is a LIVE JS reference assigned to the host `tree` property
      // (never a stringified attribute). Route it into the scope signal so the
      // windowed render reacts.
      Object.defineProperty(host, 'tree', {
        configurable: true,
        get: () => tree(),
        set: (t: Table<AnyTreeNode> | null) => {
          treeSig.set(t)
        },
      })

      // SSR fallback: build an engine from the `nodes` JSON attribute when no
      // identity engine was assigned (the sail-combobox pattern).
      if (treeSig() == null) {
        const raw = (host as { nodes?: string }).nodes ?? host.getAttribute('nodes') ?? ''
        const parsed = parseJsonNodes(raw)
        if (parsed.length > 0) {
          const selAttr = host.getAttribute('selectable') ?? 'none'
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

      // ── Delegated caret/row interactions + deduped engine-state emission ──────
      const stop = wireTreeInteractions(host, () => tree(), {
        expandOnSelect: () => host.hasAttribute('expand-on-select'),
        selectable: () => (host.getAttribute('selectable') ?? 'none') !== 'none',
        multi: () => (host.getAttribute('selectable') ?? 'none') === 'multi',
        navRoleSelector: '[role="tree"]',
        getWindow: () => windowItems(),
      })

      return () => {
        stop()
        virt.dispose()
      }
    })

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
  })

</controller>
<if {error() !== ''}>
<div role="alert">
  <error-state>{error()}</error-state>
</div>
<else-if {isEmptyInitial()}>
<div role="status">
  <slot name="empty"><empty-state></empty-state></slot>
</div>
</else-if><else-if {isEmptyFiltered()}>
<div role="status">
  <slot name="empty">
    <empty-state>
      <button slot="action" type="button" data-action="clear-filters">Clear filters</button>
    </empty-state>
  </slot>
</div>
</else-if><else>
<div role="tree" sx={style.w(full).fontFamily(body).fontSize(density).fg(foreground)} aria-label={label() || null} aria-multiselectable={selectable() === 'multi' ? 'true' : null}>
  <div bind:this={viewportEl} sx={style.position(relative).w(full).overflowY(auto).maxH(panel-max-h-md)}>
    <div use:virtualSpacer={totalSize()} sx={style.h(bind(rowsH)).position(relative).w(full)}>
      <for {windowItems()} as="w" key={w.key}>
        <!--
          virtualRow writes --row-y; sx MUST apply translateY(bind(rowY)) or every
          absolute row stacks at y=0 (studio explorer "mojibake" / illegible list).
          Same pattern as sail-tree-grid + sail-message-list. cellPad → padS(bind(cellPs)).
        -->
        <row align="center" gap="density" nowrap role="treeitem" data-node-id={w.id} aria-level={w.level} aria-posinset={w.posInSet} aria-setsize={w.setSize} aria-rowindex={w.index + 1} aria-expanded={w.canExpand ? (w.expanded ? 'true' : 'false') : null} aria-selected={selectable() !== 'none' ? (w.selected ? 'true' : 'false') : null} data-can-expand={w.canExpand ? 'true' : null} data-expanded={w.expanded ? 'true' : null} data-selected={w.selected ? 'true' : null} data-pending={w.pending ? 'true' : null} aria-busy={w.pending ? 'true' : null} tabindex={w.id === activeRowId() ? '0' : '-1'} use:virtualRow={w.start} use:cellPad={w.indent} sx={style.group().position(absolute).left(0).w(full).translateY(bind(rowY)).padY(density-row-py).padR(density-row-px).padS(bind(cellPs)).radius(sm).cursor(pointer).fg(foreground).when({ hover: style.bg(hover-bg), selected: style.bg(selected-bg).fontWeight(medium), busy: style.fg(muted-foreground).italic(true).cursor(default), focus: style.outline(ring) })}>
          <if {w.canExpand}>
            <row data-action="toggle" aria-hidden="true" align="center" justify="center" sx={style.w(density-icon).h(density-icon).minW(density-icon).fg(muted-foreground).when({ groupExpanded: style.rotate(90) })}>
              <icon name="chevron-right" size="xs"></icon>
            </row>
          <else>
            <span aria-hidden="true" sx={style.w(density-icon).minW(density-icon)}></span>
          </else></if>

          <if {selectable() === 'multi'}>
            <span sx={style}>
              <input
                type="checkbox"
                size="sm"
                aria-label={`Select ${w.label}`}
                checked={w.checked}
                indeterminate={w.someSelected}
              />
            </span>
          </if>
          <if {w.icon !== ''}>
            <icon name={w.icon} size="sm" sx={style.fg(muted-foreground).when({ groupSelected: style.fg(foreground) })}></icon>
          <else>
            <slot name="node-icon"></slot>
          </else></if>
          <span grow min-w="0" sx={style.truncate()}>{w.label}</span>
          <!--
            Trail: data `node.trailing` (e.g. dirty ●) then host slot for custom meta.
            Meta lives here — never folded into the label (557 B / stacked-text lesson).
          -->
          <span sx={style.ml(auto).fontSize(xs)}>
            <!--
              Trailing meta: dirty ● (warning) vs text badges (muted, e.g. L1/L2/L3).
              Never fold meta into the label (557 B / stacked-text lesson).
            -->
            <if {w.trailing !== ''}>
              <if {w.trailing === '●'}>
                <span sx={style.fg(warning)} aria-label="Unsaved">●</span>
              <else>
                <span sx={style.fg(muted-foreground)} aria-label={`Meta ${w.trailing}`}>{w.trailing}</span>
              </else></if>
            </if>
            <span sx={style.fg(muted-foreground)}><slot name="node-trailing"></slot></span>
          </span>
        </row>
      </for>
    </div>
  </div>
</div>
</else></if>
