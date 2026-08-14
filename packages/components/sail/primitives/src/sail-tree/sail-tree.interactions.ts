// sail-tree — delegated row interactions + engine-state emission.
//
// Shared by sail-tree and sail-tree-grid. Expansion and selection are OWNED by
// the Table engine (`toggleExpanded` / `toggleRowSelected`). This module is the
// CE host seam only (Cardinal 3): listen on the shadow root, resolve the row
// via ARIA identity (`data-node-id`), call the engine. No `data-action` markers,
// no invent-allowlists, no second expand protocol.
//
// Branch click → engine toggle expand (+ select if selectable).
// Leaf click → select only.
// Keyboard → engine nav (ArrowLeft/Right expand/collapse).
//
// Authority: build-spec §"Events", §"Keyboard nav", behavioral specs #2/#3/#4/#7.

import { effect } from '@ship/sail-runtime'
import { invokeCallback } from '@ship/sail-runtime/internal'
import type { Table } from '../sail-table/engine/create-table'
import type { ExpandedState } from '../sail-table/engine/types'
import type { TreeNode } from './tree-node'

type AnyTreeNode = TreeNode & Record<string, unknown>

type HostEmit = HTMLElement & {
  emit?(name: string, detail?: unknown): void
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
  /** After expand/collapse so the virtualizer can remeasure (spacer must shrink). */
  onExpanded?: () => void
}

function toggleAttr(el: any, name: string, on: boolean): void {
  if (on) el.setAttribute(name, '')
  else el.removeAttribute(name)
}

/** Wire delegated row click + engine keyboard nav onto the host's shadow root. */
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
    opts.onExpanded?.()
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

  const selectOnly = (id: string, shift: boolean): void => {
    const tree = getTree()
    if (!opts.selectable()) return
    if (opts.multi()) {
      tree.toggleRowSelected(id, undefined, { range: shift })
    } else {
      tree.setRowSelection({ mode: 'include', ids: new Set<string>() })
      tree.toggleRowSelected(id, true)
    }
    // L1 author surface is on-select={fn} → host.onSelect (bindProp). CustomEvent
    // remains for residual listeners/tests; callback is what studio explorer uses.
    const detail = { id }
    host.emit?.('sail-select', detail)
    invokeCallback(host, 'onSelect', detail)
    emitSelect()
  }

  const selectRow = (id: string, shift: boolean): void => {
    // expand-on-select prop: expand branch when selecting (opt-in).
    if (opts.expandOnSelect()) {
      const row = getTree().getRow(id)
      if (row?.getCanExpand()) toggleExpand(id)
    }
    selectOnly(id, shift)
  }

  const onClick = (e: Event): void => {
    const target = e.target as any
    // Checkbox is its own control — don't treat as row activation.
    if (target.closest('input[type="checkbox"]')) return

    const rowEl = target.closest('[data-node-id]')
    if (!rowEl) return
    const id = rowEl.getAttribute('data-node-id')
    if (!id) return

    const me = e as any
    const tree = getTree()
    const row = tree.getRow(id)
    // Engine capability OR consumer lazy flag (children not loaded yet).
    const orig = row?.original as { lazy?: boolean } | undefined
    const canExpand = (row?.getCanExpand() ?? false) || orig?.lazy === true
    // Engine capability — not a data-action marker. Branch row = expand target.
    if (canExpand) {
      toggleExpand(id)
      selectOnly(id, me.shiftKey === true)
    } else {
      selectRow(id, me.shiftKey === true)
    }
    host.emit?.('sail-node-click', {
      id,
      depth: Number(rowEl.getAttribute('aria-level') ?? '1') - 1,
      index: Number(rowEl.getAttribute('aria-rowindex') ?? '0') - 1,
    })
  }

  const onKeydown = (e: Event): void => {
    const ke = e as any
    if (!(ke.target as any).closest(opts.navRoleSelector)) return
    const tree = getTree()
    if (tree.nav.active().rowId == null) {
      const first = tree.rows()[0]
      const col = tree.visibleLeafColumns()[0]
      if (first) tree.nav.focusCell(first.id, col?.id ?? '')
    }
    const before = tree.state.expanded()
    const selBefore = tree.selectedRowIds().join(',')
    if (tree.nav.onKeydown(ke)) {
      ke.preventDefault()
      if (tree.state.expanded() !== before) emitExpand()
      if (tree.selectedRowIds().join(',') !== selBefore) emitSelect()
    }
  }

  // Engine → live row DOM for selection / roving focus (virtualizer recycles nodes).
  const stopReflect = effect(() => {
    void opts.getWindow()
    const tree = getTree()
    if (!shadow) return
    const selected = new Set(tree.selectedRowIds())
    const focus = tree.nav.active().rowId ?? tree.rows()[0]?.id ?? null
    shadow.querySelectorAll('[data-node-id]').forEach((rowEl) => {
      const id = rowEl.getAttribute('data-node-id') ?? ''
      const row = tree.getRow(id)
      if (!row) return
      const isSel = selected.has(id)
      toggleAttr(rowEl, 'data-selected', isSel)
      if (opts.selectable()) rowEl.setAttribute('aria-selected', String(isSel))
      rowEl.setAttribute('tabindex', focus === id ? '0' : '-1')
      // Reflect expand state so caret rotation / ARIA stay honest after recycle.
      if (row.getCanExpand()) {
        const open = row.getIsExpanded()
        rowEl.setAttribute('aria-expanded', open ? 'true' : 'false')
        toggleAttr(rowEl, 'data-expanded', open)
      }
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
