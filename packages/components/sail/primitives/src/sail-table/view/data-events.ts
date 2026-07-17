// sail-table view — data-variant DOM wiring (the ONE enhance/emit pipeline).
//
// The shadow-rendered data grid converges on a single delegated event pipeline:
// header sort, gutter/header selection, column filters, row clicks, and keyboard
// nav all route through the engine, and the engine's selection / focus / empty
// state reflect back onto the recycled rows. Extracted from the controller so
// `sail-table.controller.ts` stays the thin glue (≤300) it should be.
//
// Attribute writes only — `sail-controller-purity` bans innerHTML/outerHTML, not
// setAttribute. The engine is the state owner; this file is imperative glue
// living in `mount` (Cardinal 3).

import { effect, type Disposer } from '@ship/sail-runtime'
import { findCenterRow, type AnyRow } from './wiring'
import type { Table } from '../engine/create-table'
import type { SortState } from '../engine/types'

type SortDir = 'ascending' | 'descending' | 'none'
type Emit = <T>(name: string, detail?: T) => void

interface DataHost extends HTMLElement {
  emit?: Emit
  shadowRoot: ShadowRoot | null
}

/** Toggle a reflected boolean attribute. */
function toggleAttr(el: Element, name: string, on: boolean): void {
  if (on) el.setAttribute(name, '')
  else el.removeAttribute(name)
}

function dirOf(sorting: SortState[]): SortDir {
  const last = sorting.at(-1)
  return last ? (last.desc ? 'descending' : 'ascending') : 'none'
}

/**
 * Wire the data variant's delegated shadow events + the engine→DOM reflect
 * effects. `getTable` reads the live engine signal; `getWindow` is the windowed
 * row signal (read so the reflect re-runs as rows recycle on scroll).
 */
export function wireDataVariant(
  host: DataHost,
  getTable: () => Table<AnyRow> | null,
  getWindow: () => unknown,
): Disposer {
  const emit: Emit = (name, detail) => host.emit?.(name, detail)
  const shadow = host.shadowRoot ?? host

  const emitSort = (tab: Table<AnyRow>): void => {
    const sorting = tab.state.sorting()
    emit('sail-sort-change', { sorting })
    emit('sail-sort', { key: sorting.at(-1)?.id ?? '', direction: dirOf(sorting) })
  }

  const onClick = (ev: Event): void => {
    const tab = getTable()
    if (!tab) return
    const e = ev as MouseEvent
    const target = e.target as HTMLElement
    if (target.closest('[data-action="clear-filters"]')) {
      tab.resetFilters()
      return
    }
    if (target.closest('[data-select-all]')) {
      const next = !tab.getIsAllPageRowsSelected()
      tab.toggleAllPageRowsSelected(next)
      emit('sail-row-select', { id: null, selected: next, selectedIds: tab.selectedRowIds() })
      return
    }
    const box = target.closest('[data-row-select]')
    if (box) {
      const rowId = box.getAttribute('data-row-select') || ''
      const row = findCenterRow(tab, rowId)
      if (!row || !row.getCanSelect()) return
      const next = !row.getIsSelected()
      tab.toggleRowSelected(rowId, next, { range: e.shiftKey })
      emit('sail-row-select', { id: rowId, selected: next, selectedIds: tab.selectedRowIds() })
      return
    }
    const th = target.closest('[data-sort-key]')
    if (th) {
      tab.toggleSort(th.getAttribute('data-sort-key') || '', { multi: e.shiftKey })
      emitSort(tab)
      return
    }
    const rowEl = target.closest('[data-row-id]')
    if (rowEl && !target.closest('[data-row-select]')) {
      const id = rowEl.getAttribute('data-row-id') || ''
      emit('sail-row-click', { id, index: findCenterRow(tab, id)?.index ?? -1 })
    }
  }

  const onKey = (ev: Event): void => {
    const tab = getTable()
    if (!tab) return
    const e = ev as KeyboardEvent
    if (!(e.target as HTMLElement).closest('[role="grid"],[role="treegrid"]')) return
    if (tab.nav.onKeydown(e)) e.preventDefault()
  }

  const onInput = (e: Event): void => {
    const tab = getTable()
    if (!tab) return
    const el = e.target as HTMLElement
    const col = el.closest('[data-filter-col]')?.getAttribute('data-filter-col')
    if (!col) return
    tab.setColumnFilter(col, (el as unknown as { value?: string }).value ?? '')
  }

  // Footer <sail-pagination> → engine. `sail-page-change { page, pageSize }` is
  // 1-based; the engine pageIndex is 0-based. Set pageSize first so the page
  // window is computed against the new size in the same flush.
  const onPageChange = (e: Event): void => {
    const tab = getTable()
    if (!tab) return
    const detail = (e as CustomEvent<{ page?: number; pageSize?: number }>).detail
    if (!detail) return
    const size = Number(detail.pageSize)
    if (Number.isInteger(size) && size > 0) tab.setPageSize(size)
    const page = Number(detail.page)
    if (Number.isInteger(page) && page >= 1) tab.setPageIndex(page - 1)
  }

  shadow?.addEventListener('click', onClick)
  shadow?.addEventListener('keydown', onKey)
  shadow?.addEventListener('input', onInput)
  shadow?.addEventListener('sail-change', onInput)
  shadow?.addEventListener('sail-page-change', onPageChange)

  // Reflect [data-empty] from the engine's empty discriminators.
  const stopEmpty = effect(() => {
    const tab = getTable()
    const empty = tab != null && (tab.isEmptyInitial() || tab.isEmptyFiltered())
    toggleAttr(host, 'data-empty', empty)
  })

  // Reflect selection + roving-focus onto the recycled rows + checkboxes. Reads
  // the window signal so freshly scrolled-in rows pick up current state.
  const stopReflect = effect(() => {
    void getWindow()
    const tab = getTable()
    if (!tab || !shadow) return
    const selected = new Set(tab.selectedRowIds())
    const focus = tab.nav.active().rowId ?? tab.centerRows()[0]?.id ?? null
    shadow.querySelectorAll<HTMLElement>('[data-row-id]').forEach((rowEl) => {
      const id = rowEl.getAttribute('data-row-id') ?? ''
      const isSel = selected.has(id)
      toggleAttr(rowEl, 'data-selected', isSel)
      toggleAttr(rowEl, 'aria-selected', isSel)
      rowEl.setAttribute('tabindex', focus === id ? '0' : '-1')
      const box = rowEl.querySelector<HTMLInputElement>('input[type="checkbox"][data-row-select]')
      if (box) box.checked = isSel
    })
    const headerBox = shadow.querySelector<HTMLInputElement>('[data-select-all]')
    if (headerBox) {
      const all = tab.getIsAllPageRowsSelected()
      headerBox.checked = all
      headerBox.indeterminate = !all && tab.getIsSomePageRowsSelected()
    }
  })

  return () => {
    shadow?.removeEventListener('click', onClick)
    shadow?.removeEventListener('keydown', onKey)
    shadow?.removeEventListener('input', onInput)
    shadow?.removeEventListener('sail-change', onInput)
    shadow?.removeEventListener('sail-page-change', onPageChange)
    stopEmpty()
    stopReflect()
  }
}
