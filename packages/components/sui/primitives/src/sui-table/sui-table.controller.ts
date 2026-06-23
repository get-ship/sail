import { controller, signal, derived, ref, bindNativeEvent } from '@ship/sui-runtime'
import { createTable, type Table } from './engine/create-table'
import { parseJsonColumns, parseJsonRows } from './engine/decode'
import {
  buildViewportVirtualizer,
  headerViews,
  windowRows,
  type AnyRow,
  type HeaderView,
} from './view/wiring'
import { wireDataVariant } from './view/data-events'
import { SUI_TABLE_LIGHT_DOM_CSS, installLightDomStyles } from './view/light-dom-css'

type SortDirection = 'ascending' | 'descending' | 'none'

const ROW_HEIGHT = 40

/** A no-op empty engine so the data-variant scope always has a Table shape to
 *  read (the real engine arrives via the host `table` property or the SSR
 *  `rows`/`columns` fallback). */
function emptyTable(): Table<AnyRow> {
  return createTable<AnyRow>({ data: [], columns: [], getRowId: (r) => r.id })
}

export default controller({
  scope: () => {
    // The live engine. Null until the consumer assigns `prop:table` (the host
    // property, wired in mount) OR the SSR `rows`/`columns` fallback seeds one.
    const tableSig = signal<Table<AnyRow> | null>(null)
    const viewportEl = ref<HTMLDivElement>(null)

    // A stable empty engine backs every read so the template never branches on
    // null; the active engine is `tableSig() ?? EMPTY`.
    const EMPTY = emptyTable()
    const table = derived<Table<AnyRow>>(() => tableSig() ?? EMPTY)

    // The virtualizer windows the center band. `getScrollElement` is the ref
    // thunk (null until bind:this mounts); `count` tracks centerRows().length.
    const virt = buildViewportVirtualizer<AnyRow>(
      () => tableSig(),
      () => viewportEl(),
      ROW_HEIGHT,
    )

    const headers = derived<HeaderView[]>(() => headerViews(table()))
    const windowItems = windowRows<AnyRow>(table, virt.virtualItems)

    const isEmptyInitial = derived<boolean>(() => table().isEmptyInitial())
    const isEmptyFiltered = derived<boolean>(() => table().isEmptyFiltered())
    const rowCount = derived<number>(() => table().rowCount())
    const colCount = derived<number>(() => table().visibleLeafColumns().length)
    const totalSize = virt.totalSize

    // ── Pagination footer (Slice 6) ─────────────────────────────────────────
    // Pagination is opt-in: pageSize 0 = no footer (the full set flows to the
    // virtualizer). When a pageSize is set, the footer <sui-pagination> reads
    // these 1-based views and drives setPageIndex/setPageSize on sui-page-change.
    const hasPagination = derived<boolean>(() => table().state.pagination().pageSize > 0)
    const pageSize = derived<number>(() => table().state.pagination().pageSize)
    const pageIndex1 = derived<number>(() => table().state.pagination().pageIndex + 1)

    return {
      tableSig,
      viewportEl,
      table,
      virt,
      headers,
      windowItems,
      isEmptyInitial,
      isEmptyFiltered,
      rowCount,
      colCount,
      totalSize,
      hasPagination,
      pageSize,
      pageIndex1,
    }
  },

  mount(state, host) {
    // ── prop:table identity wiring ──────────────────────────────────────────
    // The engine is a LIVE JS reference assigned to the host `table` property
    // (never a stringified attribute). Define a property that routes into the
    // scope signal so the windowed render reacts. The `rows`/`columns` string
    // props remain an SSR/declarative fallback (the sui-combobox pattern).
    const adoptTable = (t: Table<AnyRow> | null): void => {
      state.tableSig = t
    }
    Object.defineProperty(host, 'table', {
      configurable: true,
      get: () => state.tableSig,
      set: (t: Table<AnyRow> | null) => adoptTable(t),
    })

    // The reflected `variant` lives on the host attribute (props are not in the
    // typed scope; read the canonical reflected attribute).
    const isData = (): boolean => host.getAttribute('variant') === 'data'

    // SSR fallback: seed an engine from the `rows`/`columns` string props when
    // no identity engine was assigned and the markup carried serialized data.
    if (state.tableSig == null && isData()) {
      const rawRows = parseJsonRows((host as { rows?: string }).rows ?? '')
      const cols = parseJsonColumns((host as { columns?: string }).columns ?? '')
      if (cols.length > 0) {
        adoptTable(
          createTable<AnyRow>({
            data: rawRows,
            columns: cols,
            getRowId: (r) => r.id,
            enableRowSelection: true,
          }),
        )
      }
    }

    installLightDomStyles(host.ownerDocument)

    // ── Data variant: the ONE delegated enhance/emit pipeline + engine reflect ─
    // Installed once; no-ops harmlessly for slot/native (no shadow grid markup).
    const stopDataWiring = wireDataVariant(
      host,
      () => state.tableSig,
      () => state.$signals.windowItems(),
    )

    // ── Slot / native variant enhancement (KEEP — do not disturb) ────────────
    const enhanceSortableHeaders = (): void => {
      host.querySelectorAll<HTMLTableCellElement>('th[data-sort-key]').forEach((th) => {
        if (!th.hasAttribute('aria-sort')) th.setAttribute('aria-sort', 'none')
        if (!th.hasAttribute('tabindex')) th.setAttribute('tabindex', '0')
        if (!th.hasAttribute('role')) th.setAttribute('role', 'columnheader')
      })
    }
    const syncEmptyState = (): void => {
      const bodyRow =
        host.querySelector(':scope > table tbody > tr') ||
        host.querySelector(':scope > table > tr:not(:has(th))') ||
        host.querySelector(':scope > tbody > tr') ||
        host.querySelector(':scope > tr:not([slot="head"])')
      const hasEmptySlot = host.querySelector('[slot="empty"]') != null
      if (!bodyRow && hasEmptySlot) host.setAttribute('data-empty', '')
      else if (!isData()) host.removeAttribute('data-empty')
    }

    const cycleSlotSort = (th: HTMLTableCellElement): void => {
      const key = th.getAttribute('data-sort-key') || ''
      const current = (th.getAttribute('aria-sort') || 'none') as SortDirection
      const next: SortDirection =
        current === 'none' ? 'ascending' : current === 'ascending' ? 'descending' : 'none'
      host
        .querySelectorAll<HTMLTableCellElement>('th[data-sort-key]')
        .forEach((peer) => peer !== th && peer.setAttribute('aria-sort', 'none'))
      th.setAttribute('aria-sort', next)
      host.emit!('sui-sort', { key, direction: next })
    }

    // Light-DOM (slot/native) header click → sui-sort cycle on the host's own
    // <th>; native row click → sui-row-click. The data variant lives in the
    // shadow root and is handled by the delegated shadow listeners below.
    const onLightClick = (e: MouseEvent): void => {
      const target = e.target as HTMLElement
      const th = target.closest('th[data-sort-key]') as HTMLTableCellElement | null
      if (th && host.contains(th)) {
        cycleSlotSort(th)
        return
      }
      const td = target.closest('td') as HTMLTableCellElement | null
      if (!td || !host.contains(td)) return
      if (td.classList.contains('sui-table-gutter-cell')) return
      const tr = td.closest('tr') as HTMLTableRowElement | null
      const rowId = tr?.getAttribute('data-row-id')
      if (!rowId) return
      host.emit!('sui-row-click', { id: rowId, index: tr!.rowIndex })
    }
    const onLightKey = (e: KeyboardEvent): void => {
      if (e.key !== 'Enter' && e.key !== ' ') return
      const th = (e.target as HTMLElement).closest('th[data-sort-key]') as HTMLTableCellElement | null
      if (!th || !host.contains(th)) return
      e.preventDefault()
      cycleSlotSort(th)
    }
    bindNativeEvent(host, 'click', onLightClick as EventListener)
    bindNativeEvent(host, 'keydown', onLightKey as EventListener)

    enhanceSortableHeaders()
    syncEmptyState()
    const observer = new MutationObserver(() => {
      enhanceSortableHeaders()
      syncEmptyState()
    })
    observer.observe(host, { childList: true, subtree: true })

    return () => {
      observer.disconnect()
      stopDataWiring()
      state.virt.dispose()
    }
  },
})

export { SUI_TABLE_LIGHT_DOM_CSS }
