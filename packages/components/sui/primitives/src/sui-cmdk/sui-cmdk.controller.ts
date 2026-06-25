// <sui-cmdk> controller — generic command palette.
//
// Composes <sui-dialog> (modal shell + focus trap + scroll lock + ESC) and
// owns the search/filter/keyboard-nav layer on top of it. RUNTIME-FREE: data
// arrives via the `items` IDENTITY property (a live CmdkItem[]) or the `items`
// JSON attribute (SSR fallback), and selections leave as `sui-cmdk-*` events.
// No client, no router — the consuming app wires those.
//
// Templates declare, controllers do: the filtered/grouped render rows are
// derived here; mutations (highlight, select, open/close sync) + emission live
// in mount's listeners.

import { controller, signal, derived, effect, ref } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './sui-cmdk.sui'
import { type CmdkItem, str, parseJsonItems } from './decode'

export type { CmdkItem } from './decode'

/** A normalized, filter-position-tagged item (concrete fields, no optionals). */
interface VisibleItem {
  id: string
  optId: string
  value: string
  label: string
  icon: string
  hint: string
  group: string
  disabled: boolean
}

/** A flat render row — uniform shape (heading rows zero the item fields) so the
 *  template never relies on discriminated-union narrowing. */
interface RenderRow {
  key: string
  isHeading: boolean
  heading: string
  id: string
  optId: string
  value: string
  label: string
  icon: string
  hint: string
  disabled: boolean
}

let _seq = 0

export default controller({
  scope: (_ctx: ViewScope<Scope>) => {
    const seq = ++_seq
    const itemsSig = signal<CmdkItem[]>([])
    const query = signal('')
    const highlight = signal(0)
    const listId = signal(`sui-cmdk-list-${seq}`)

    const dialogEl = ref<HTMLElement>(null)
    const inputEl = ref<HTMLInputElement>(null)
    const listEl = ref<HTMLUListElement>(null)

    // Filter case-insensitively over label + keywords, preserving order, and
    // tag each survivor with a stable per-instance option id.
    const visible = derived<VisibleItem[]>(() => {
      const q = query().trim().toLowerCase()
      const out: VisibleItem[] = []
      for (const it of itemsSig()) {
        const hay = q ? `${it.label} ${it.keywords ?? ''}`.toLowerCase() : ''
        if (q && !hay.includes(q)) continue
        out.push({
          id: it.id,
          optId: `sui-cmdk-opt-${seq}-${it.id}`,
          value: it.value != null ? it.value : it.id,
          label: it.label,
          icon: str(it.icon),
          hint: str(it.hint),
          group: str(it.group),
          disabled: it.disabled === true,
        })
      }
      return out
    })

    // The keyboard-navigable subset (enabled rows only); highlight indexes here.
    const navigable = derived<VisibleItem[]>(() => visible().filter((v) => !v.disabled))

    // Highlight clamped to the live navigable range (or 0 when empty), so a
    // shrinking filtered set can never strand the selection past the end.
    const safeHighlight = derived<number>(() => {
      const n = navigable().length
      if (n === 0) return 0
      const h = highlight()
      return h < 0 ? 0 : h >= n ? n - 1 : h
    })

    const highlightId = derived<string>(() => navigable()[safeHighlight()]?.id ?? '')
    const activeId = derived<string>(() => navigable()[safeHighlight()]?.optId ?? '')

    // Flatten to render rows, injecting a heading row whenever `group` changes.
    const rows = derived<RenderRow[]>(() => {
      const out: RenderRow[] = []
      let lastGroup: string | null = null
      for (const v of visible()) {
        if (v.group !== '' && v.group !== lastGroup) {
          out.push({
            key: `g:${v.group}`,
            isHeading: true,
            heading: v.group,
            id: '',
            optId: '',
            value: '',
            label: '',
            icon: '',
            hint: '',
            disabled: false,
          })
        }
        lastGroup = v.group
        out.push({
          key: `i:${v.id}`,
          isHeading: false,
          heading: '',
          id: v.id,
          optId: v.optId,
          value: v.value,
          label: v.label,
          icon: v.icon,
          hint: v.hint,
          disabled: v.disabled,
        })
      }
      return out
    })

    return {
      itemsSig, query, highlight, listId,
      dialogEl, inputEl, listEl,
      visible, navigable, safeHighlight, highlightId, activeId, rows,
    }
  },

  mount(state, host) {
    // ── `items` IDENTITY property + JSON-attribute SSR fallback ──────────────
    // A live CmdkItem[] assigned to the host `items` property (never a string).
    // Defined on the instance so it shadows nothing — `items` is intentionally
    // NOT a declared <prop>, so there is no compiler accessor to clash with.
    Object.defineProperty(host, 'items', {
      configurable: true,
      get: () => state.itemsSig,
      set: (v: CmdkItem[] | null) => {
        state.itemsSig = Array.isArray(v) ? v : []
        state.highlight = 0
      },
    })
    if (state.itemsSig.length === 0) {
      const raw = host.getAttribute('items') ?? ''
      const parsed = parseJsonItems(raw)
      if (parsed.length > 0) state.itemsSig = parsed
    }

    const scrollActiveIntoView = () => {
      const list = state.listEl as HTMLUListElement | null
      const id = state.activeId as string
      if (!list || !id) return
      // Match by id without a CSS selector — option ids derive from caller
      // data and `CSS.escape` is absent under jsdom, so a plain scan is the
      // robust path. scrollIntoView is optional (jsdom stubs it).
      let target: HTMLElement | null = null
      for (const li of Array.from(list.querySelectorAll('li.item'))) {
        if ((li as HTMLElement).id === id) {
          target = li as HTMLElement
          break
        }
      }
      target?.scrollIntoView?.({ block: 'nearest' })
    }

    // Reset query + highlight on each open; emit open/close. The dialog drives
    // its own native open via prop:open={open()}; here we react to the state.
    let wasOpen = false
    const stopOpenSync = effect(() => {
      const isOpen = state.open === true
      if (isOpen === wasOpen) return
      wasOpen = isOpen
      if (isOpen) {
        state.query = ''
        state.highlight = 0
        host.emit?.('sui-cmdk-open', {})
        // Focus is grabbed by the dialog's own focus trap (the input is its
        // first focusable); nudge the highlight into view once rows paint.
        queueMicrotask(scrollActiveIntoView)
      } else {
        host.emit?.('sui-cmdk-close', {})
      }
    })

    // The dialog closes itself on ESC / backdrop click; mirror that back into
    // our `open` prop so the two never drift.
    const dialog = state.dialogEl as HTMLElement | null
    const onDialogClose = () => {
      if (state.open) state.open = false
    }
    dialog?.addEventListener('sui-close', onDialogClose)

    const input = state.inputEl as HTMLInputElement | null

    const selectByItem = (item: VisibleItem | undefined) => {
      if (!item || item.disabled) return
      host.emit?.('sui-cmdk-select', { id: item.id, value: item.value })
      state.open = false
    }

    const onInput = () => {
      state.highlight = 0
      host.emit?.('sui-cmdk-input', { query: (state.query as string) ?? '' })
    }

    const moveHighlight = (delta: number) => {
      const list = state.navigable as VisibleItem[]
      const n = list.length
      if (n === 0) return
      const base = state.safeHighlight as number
      state.highlight = (base + delta + n) % n
      queueMicrotask(scrollActiveIntoView)
    }

    const onKeydown = (e: KeyboardEvent) => {
      const list = state.navigable as VisibleItem[]
      if (e.key === 'ArrowDown') {
        e.preventDefault()
        moveHighlight(1)
      } else if (e.key === 'ArrowUp') {
        e.preventDefault()
        moveHighlight(-1)
      } else if (e.key === 'Home') {
        e.preventDefault()
        state.highlight = 0
        queueMicrotask(scrollActiveIntoView)
      } else if (e.key === 'End') {
        e.preventDefault()
        state.highlight = Math.max(0, list.length - 1)
        queueMicrotask(scrollActiveIntoView)
      } else if (e.key === 'Enter') {
        e.preventDefault()
        selectByItem(list[state.safeHighlight as number])
      }
      // Escape is intentionally untouched — the dialog's native `cancel`
      // handler closes the modal and fires `sui-close`.
    }

    input?.addEventListener('input', onInput)
    input?.addEventListener('keydown', onKeydown)

    // Pointer commit. mousedown (before any blur/focus shuffle) selects the row.
    const list = state.listEl as HTMLUListElement | null
    const onListMousedown = (e: MouseEvent) => {
      const li = (e.target as HTMLElement | null)?.closest?.('li.item[data-id]') as HTMLElement | null
      if (!li || li.getAttribute('aria-disabled') === 'true') return
      e.preventDefault()
      const id = li.dataset.id ?? ''
      const match = (state.visible as VisibleItem[]).find((v) => v.id === id)
      selectByItem(match)
    }
    list?.addEventListener('mousedown', onListMousedown)

    // `show()` (not `open()`) — `open` is a declared <prop>, so its accessor
    // owns that name; consumers open imperatively via `el.show()` or set
    // `el.open = true` directly.
    host.expose?.({
      show: () => {
        state.open = true
      },
      close: () => {
        state.open = false
      },
      toggle: () => {
        state.open = !state.open
      },
    })

    return () => {
      stopOpenSync()
      dialog?.removeEventListener('sui-close', onDialogClose)
      input?.removeEventListener('input', onInput)
      input?.removeEventListener('keydown', onKeydown)
      list?.removeEventListener('mousedown', onListMousedown)
    }
  },
})
