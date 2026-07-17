// sail-tree — delegated caret/row interactions + deduped engine-state emission.
//
// Kept off the controllers so both `sail-tree` and `sail-tree-grid` share ONE
// interaction+emit path (one way to do things). The shadow body carries
// `.caret` (disclosure) and `[data-node-id]` (row) markers; this module
// delegates clicks/keydown on the SHADOW ROOT (composed:false events never
// escape to the host) to `tree.toggleExpanded` / `tree.toggleRowSelected` /
// `nav.onKeydown`, and emits the `sail-*` events via `host.emit!` (deduped, the
// sail-combobox lastEmitted pattern).
//
// Attribute/event wiring only — `sail-controller-purity` bans innerHTML, not DOM
// listeners. The engine is the state owner; this is the controller's imperative
// seam (Cardinal 3). No SDK, no fetch.
//
// Authority: build-spec §"Events", §"Keyboard nav", behavioral specs #2/#3/#4/#7.

import { effect } from '@ship/sail-runtime'
import type { Table } from '../sail-table/engine/create-table'
import type { ExpandedState } from '../sail-table/engine/types'
import type { TreeNode } from '../sail-tree-view/sail-tree-view.controller'

type AnyTreeNode = TreeNode & Record<string, unknown>

interface HostEmit extends HTMLElement {
  emit?<T = unknown>(name: string, detail?: T): void
}

export interface TreeInteractionOpts {
  expandOnSelect: () => boolean
  selectable: () => boolean
  /** True only when `selectable=multi` — gates shift-range + additive select. */
  multi: () => boolean
  /** Role-scope guard for keydown (`tree` for sail-tree, `treegrid` for grid). */
  navRoleSelector: string
  /** Read the windowed rows so the reflect re-runs as rows recycle on scroll. */
  getWindow: () => unknown
}

/** Toggle a reflected boolean attribute. */
function toggleAttr(el: Element, name: string, on: boolean): void {
  if (on) el.setAttribute(name, '')
  else el.removeAttribute(name)
}

/** Wire delegated caret/row click + engine keyboard nav onto the host's shadow
 *  root. Returns a teardown that removes every listener. */
export function wireTreeInteractions(
  host: HostEmit,
  getTree: () => Table<AnyTreeNode>,
  opts: TreeInteractionOpts,
): () => void {
  const shadow = host.shadowRoot ?? host
  let lastExpanded = ''
  let lastSelected = ''

  const emitExpand = (): void => {
    const expanded: ExpandedState = getTree().state.expanded()
    const snap = JSON.stringify(expanded)
    if (snap === lastExpanded) return
    lastExpanded = snap
    host.emit?.('sail-expand-change', { expanded })
  }
  const emitSelect = (): void => {
    const selectedIds = getTree().selectedRowIds()
    const snap = selectedIds.join(',')
    if (snap === lastSelected) return
    lastSelected = snap
    host.emit?.('sail-node-select', { selectedIds, selected: selectedIds.length > 0 })
  }

  const toggleExpand = (id: string): void => {
    const tree = getTree()
    tree.toggleExpanded(id)
    host.emit?.('sail-toggle', { id, expanded: tree.getRow(id)?.getIsExpanded() ?? false })
    emitExpand()
  }
  const selectRow = (id: string, shift: boolean): void => {
    const tree = getTree()
    if (opts.expandOnSelect()) toggleExpand(id)
    if (!opts.selectable()) return
    // single mode: a plain click REPLACES the selection (one row at a time) and
    // ignores shift-range; multi mode honors the shift-range span over the
    // displayed order, pivoting around the anchor.
    if (opts.multi()) {
      tree.toggleRowSelected(id, undefined, { range: shift })
    } else {
      // Replace the whole selection, then select the target (which also sets the
      // anchor) so single mode is exactly one selected row at a time.
      tree.setRowSelection({ mode: 'include', ids: new Set<string>() })
      tree.toggleRowSelected(id, true)
    }
    host.emit?.('sail-select', { id })
    emitSelect()
  }

  const onClick = (e: Event): void => {
    const target = e.target as HTMLElement
    // Caret first (disclosure toggle, never a selection).
    const caret = target.closest('[data-action="toggle"]')
    if (caret) {
      const id = caret.closest('[data-node-id]')?.getAttribute('data-node-id')
      if (id) {
        toggleExpand(id)
        e.stopPropagation()
      }
      return
    }
    const rowEl = target.closest('[data-node-id]')
    if (rowEl) {
      const id = rowEl.getAttribute('data-node-id')
      if (!id) return
      const me = e as MouseEvent
      selectRow(id, me.shiftKey === true)
      host.emit?.('sail-node-click', {
        id,
        depth: Number(rowEl.getAttribute('aria-level') ?? '1') - 1,
        index: Number(rowEl.getAttribute('aria-rowindex') ?? '0') - 1,
      })
    }
  }

  const onKeydown = (e: Event): void => {
    const ke = e as KeyboardEvent
    if (!(ke.target as HTMLElement).closest(opts.navRoleSelector)) return
    const tree = getTree()
    // Seed nav focus to the first visible row before the first key so the
    // engine's tree-nav (treeArrow / move) has an active row to act on — the
    // roving-tabindex already defaults the visual focus to row 0.
    if (tree.nav.active().rowId == null) {
      const first = tree.rows()[0]
      const col = tree.visibleLeafColumns()[0]
      if (first) tree.nav.focusCell(first.id, col?.id ?? '')
    }
    const before = tree.state.expanded()
    const selBefore = tree.selectedRowIds().join(',')
    if (tree.nav.onKeydown(ke)) {
      ke.preventDefault()
      // A key may have toggled expansion/selection (Right/Left/Space) — emit.
      if (tree.state.expanded() !== before) emitExpand()
      if (tree.selectedRowIds().join(',') !== selBefore) emitSelect()
    }
  }

  // ── Engine → DOM reflect (selection / branch tri-state / roving focus) ───────
  // A virtualized {#for} recycles row DOM on scroll/expand but the per-row prop
  // bindings do not re-fire on a pure selection change (no window move), so the
  // engine's selection + nav focus are reflected onto the live row + checkbox
  // here. Reads getWindow() so freshly recycled rows pick up current state.
  const stopReflect = effect(() => {
    void opts.getWindow()
    const tree = getTree()
    if (!shadow) return
    const selected = new Set(tree.selectedRowIds())
    const focus = tree.nav.active().rowId ?? tree.rows()[0]?.id ?? null
    shadow.querySelectorAll<HTMLElement>('[data-node-id]').forEach((rowEl) => {
      const id = rowEl.getAttribute('data-node-id') ?? ''
      const row = tree.getRow(id)
      if (!row) return
      // `data-selected` reflects the ENGINE selection set exactly (the displayed
      // selectedRowIds). A branch is highlighted only when it is itself in the
      // range span, never merely because all its leaf descendants are selected —
      // that "all leaves selected" state surfaces as the checkbox indeterminate/
      // checked tri-state below, not a row highlight.
      const isSel = selected.has(id)
      toggleAttr(rowEl, 'data-selected', isSel)
      if (opts.selectable()) rowEl.setAttribute('aria-selected', String(isSel))
      rowEl.setAttribute('tabindex', focus === id ? '0' : '-1')
      const box = rowEl.querySelector<HTMLInputElement>('input[type="checkbox"]')
      if (box) {
        box.checked = row.getIsSelected()
        box.indeterminate = row.getIsSomeSelected()
      }
    })
  })

  shadow?.addEventListener('click', onClick)
  shadow?.addEventListener('keydown', onKeydown)
  return () => {
    stopReflect()
    shadow?.removeEventListener('click', onClick)
    shadow?.removeEventListener('keydown', onKeydown)
  }
}
