// sui-tree-grid — delegated column-header interactions (sort / hide / resize).
//
// The tree-grid inherits the REUSED Table engine's ORTHOGONAL column lane: the
// hierarchy is `createTable({ getSubRows, expanded })`, and sort/visibility/
// resize route through the SAME `toggleSort` / `setColumnVisibility` /
// `setColumnSizing` the sui-table view drives — there is no tree-specific column
// engine. This module is the imperative seam (the controller's `on:`-equivalent,
// Cardinal 3): it delegates header clicks + a pointer-drag resize on the shadow
// root to those engine mutators, and emits `sui-sort` via `host.emit!` (deduped).
//
// Attribute/event wiring only — no innerHTML, no SDK, no fetch.
//
// Authority: build-spec §"sui-tree-grid", Slice 5; behavioral spec #8, reusing
// the committed column lane (column-mutators.ts / column-api.ts).

import type { Table } from '../sui-table/engine/create-table'
import type { TreeNode } from '../sui-tree-view/sui-tree-view.controller'

type AnyTreeNode = TreeNode & Record<string, unknown>

interface HostEmit extends HTMLElement {
  emit?<T = unknown>(name: string, detail?: T): void
}

/** Map the engine's sort direction to the `sui-sort` event token. */
function sortToken(dir: 'asc' | 'desc' | false): 'ascending' | 'descending' | 'none' {
  return dir === 'asc' ? 'ascending' : dir === 'desc' ? 'descending' : 'none'
}

/**
 * Wire delegated column-header sort/hide + a pointer-drag resize onto the host's
 * shadow root. Sort fires `tree.toggleSort(id)` + emits `sui-sort {id,direction}`;
 * the column menu toggles `tree.setColumnVisibility`; the resize handle drives a
 * live `tree.setColumnSizing` (onChange) over the drag. Returns a teardown.
 */
export function wireGridColumns(host: HostEmit, getTree: () => Table<AnyTreeNode>): () => void {
  const shadow = host.shadowRoot

  const emitSort = (id: string): void => {
    const dir = getTree().getColumn(id)?.getIsSorted() ?? false
    host.emit?.('sui-sort', { id, direction: sortToken(dir) })
  }

  // ── Resize drag state (single active handle at a time) ──────────────────────
  let resizing: { id: string; startX: number; startW: number } | null = null

  const onPointerMove = (e: PointerEvent | MouseEvent): void => {
    if (!resizing) return
    const delta = e.clientX - resizing.startX
    getTree().setColumnSizing({ [resizing.id]: resizing.startW + delta }, { mode: 'onChange' })
  }
  const onPointerUp = (): void => {
    if (!resizing) return
    resizing = null
    document.removeEventListener('pointermove', onPointerMove as EventListener)
    document.removeEventListener('pointerup', onPointerUp)
  }

  const onPointerDown = (e: Event): void => {
    const handle = (e.target as HTMLElement).closest('[part="resize-handle"]')
    if (!handle) return
    const id = handle.getAttribute('data-col-id')
    if (!id) return
    e.preventDefault()
    e.stopPropagation()
    const startW = Number.parseFloat(getTree().getColumnSizeVarValue(id)) || 0
    resizing = { id, startX: (e as PointerEvent).clientX, startW }
    document.addEventListener('pointermove', onPointerMove as EventListener)
    document.addEventListener('pointerup', onPointerUp)
  }

  const onClick = (e: Event): void => {
    const target = e.target as HTMLElement
    // Column-menu trigger → hide the column (the reused dropdown-menu pattern's
    // primary action; a full menu surface is a later affordance).
    const menu = target.closest('[data-action="column-menu"]')
    if (menu) {
      const id = menu.getAttribute('data-col-id')
      if (id) {
        getTree().setColumnVisibility({ [id]: false })
        e.stopPropagation()
      }
      return
    }
    // The resize handle never sorts.
    if (target.closest('[part="resize-handle"]')) return
    // A sortable header click cycles its sort none→asc→desc→none.
    const head = target.closest<HTMLElement>('[role="columnheader"][data-sortable]')
    if (head) {
      const id = head.getAttribute('data-col-id')
      if (id) {
        getTree().toggleSort(id)
        emitSort(id)
      }
    }
  }

  shadow?.addEventListener('click', onClick)
  shadow?.addEventListener('pointerdown', onPointerDown)
  return () => {
    shadow?.removeEventListener('click', onClick)
    shadow?.removeEventListener('pointerdown', onPointerDown)
    document.removeEventListener('pointermove', onPointerMove as EventListener)
    document.removeEventListener('pointerup', onPointerUp)
  }
}
