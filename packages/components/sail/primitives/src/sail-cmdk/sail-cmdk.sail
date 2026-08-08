<!--
  <sail-cmdk> — command palette: native <dialog> + filterable listbox.
  Items via `items` identity (CmdkItem[]) or JSON attr SSR fallback.
  Events: sail-cmdk-select / -input / -open / -close. Host: open/close/toggle.
-->

<scope>
  props: {
    open = false
    placeholder = 'Type a command or search…'
    label = 'Command palette'
    'empty-label' = 'No results found.'
    size: 'md' | 'sm' | 'lg' = 'md'
  }
</scope>
<controller>
// Dialog shell via action; filter + keyboard nav + emits here.

import type { Action, ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './sail-cmdk.sail'
import { type CmdkItem, str, parseJsonItems } from './decode'

export type { CmdkItem } from './decode'

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

/** Flat row — heading rows zero item fields (no union narrowing in template). */
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

export default controller((_ctx: ControllerScope<Scope>) => {
    const seq = ++_seq
    const itemsSig = signal<CmdkItem[]>([])
    const query = signal('')
    const highlight = signal(0)
    const listId = signal(`sail-cmdk-list-${seq}`)

    // listEl read from input keydown handlers (scrollActiveIntoView).
    const inputEl = signal<HTMLInputElement | null>(null)
    const listEl = signal<HTMLUListElement | null>(null)
    const captureInput: Action<void> = (node) => {
      inputEl.set(node as HTMLInputElement)
      return { destroy: () => inputEl.set(null) }
    }
    const captureList: Action<void> = (node) => {
      listEl.set(node as HTMLUListElement)
      return { destroy: () => listEl.set(null) }
    }

    let hostEl: (HTMLElement & { open: boolean }) | null = null

    // open prop ⇄ dialog showModal/close.
    const dialogBridge: Action<void> = (node) => {
      const el = node as HTMLDialogElement
      const stop = effect(() => {
        const want = _ctx.open() === true
        if (want && !el.open) el.showModal()
        else if (!want && el.open) el.close()
      })
      const onNativeClose = (): void => {
        if (_ctx.open() && hostEl) hostEl.open = false
      }
      el.addEventListener('close', onNativeClose)
      return {
        destroy: () => {
          stop()
          el.removeEventListener('close', onNativeClose)
        },
      }
    }


    const visible = derived<VisibleItem[]>(() => {
      const q = query().trim().toLowerCase()
      const out: VisibleItem[] = []
      for (const it of itemsSig()) {
        const hay = q ? `${it.label} ${it.keywords ?? ''}`.toLowerCase() : ''
        if (q && !hay.includes(q)) continue
        out.push({
          id: it.id,
          optId: `sail-cmdk-opt-${seq}-${it.id}`,
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


    const navigable = derived<VisibleItem[]>(() => visible().filter((v) => !v.disabled))


    const safeHighlight = derived<number>(() => {
      const n = navigable().length
      if (n === 0) return 0
      const h = highlight()
      return h < 0 ? 0 : h >= n ? n - 1 : h
    })

    const highlightId = derived<string>(() => navigable()[safeHighlight()]?.id ?? '')
    const activeId = derived<string>(() => navigable()[safeHighlight()]?.optId ?? '')


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

    mount((host) => {
      hostEl = host as HTMLElement & { open: boolean }
      const h = hostEl
      // items identity (live CmdkItem[]) — not a declared prop.
      Object.defineProperty(host, 'items', {
        configurable: true,
        get: () => itemsSig(),
        set: (v: CmdkItem[] | null) => {
          itemsSig.set(Array.isArray(v) ? v : [])
          highlight.set(0)
        },
      })
      if (itemsSig().length === 0) {
        const raw = host.getAttribute('items') ?? ''
        const parsed = parseJsonItems(raw)
        if (parsed.length > 0) itemsSig.set(parsed)
      }

      const scrollActiveIntoView = () => {
        const list = listEl() as HTMLUListElement | null
        const id = activeId() as string
        if (!list || !id) return
        // Scan by id (CSS.escape missing under jsdom).
        let target: HTMLElement | null = null
        for (const li of Array.from(list.querySelectorAll('li.item'))) {
          if ((li as HTMLElement).id === id) {
            target = li as HTMLElement
            break
          }
        }
        target?.scrollIntoView?.({ block: 'nearest' })
      }


      let wasOpen = false
      const stopOpenSync = effect(() => {
        const isOpen = _ctx.open() === true
        if (isOpen === wasOpen) return
        wasOpen = isOpen
        if (isOpen) {
          query.set('')
          highlight.set(0)
          host.emit?.('sail-cmdk-open', {})
          // Focus is grabbed by the dialog's own focus trap (the input is its
          // first focusable); nudge the highlight into view once rows paint.
          queueMicrotask(scrollActiveIntoView)
        } else {
          host.emit?.('sail-cmdk-close', {})
        }
      })

      // `open` prop ⇄ the native modal. The platform owns top layer, focus
      // trap, ESC and focus-return; the component owns this element, so it
      // calls showModal()/close() directly and mirrors a native close (ESC,
      // backdrop, form method=dialog) back into `open`.
      const input = inputEl() as HTMLInputElement | null

      const selectByItem = (item: VisibleItem | undefined) => {
        if (!item || item.disabled) return
        host.emit?.('sail-cmdk-select', { id: item.id, value: item.value })
        h.open = false
      }

      const onInput = () => {
        highlight.set(0)
        host.emit?.('sail-cmdk-input', { query: (query() as string) ?? '' })
      }

      const moveHighlight = (delta: number) => {
        const list = navigable() as VisibleItem[]
        const n = list.length
        if (n === 0) return
        const base = safeHighlight() as number
        highlight.set((base + delta + n) % n)
        queueMicrotask(scrollActiveIntoView)
      }

      const onKeydown = (e: KeyboardEvent) => {
        const list = navigable() as VisibleItem[]
        if (e.key === 'ArrowDown') {
          e.preventDefault()
          moveHighlight(1)
        } else if (e.key === 'ArrowUp') {
          e.preventDefault()
          moveHighlight(-1)
        } else if (e.key === 'Home') {
          e.preventDefault()
          highlight.set(0)
          queueMicrotask(scrollActiveIntoView)
        } else if (e.key === 'End') {
          e.preventDefault()
          highlight.set(Math.max(0, list.length - 1))
          queueMicrotask(scrollActiveIntoView)
        } else if (e.key === 'Enter') {
          e.preventDefault()
          selectByItem(list[safeHighlight() as number])
        }
        // Escape is intentionally untouched — the dialog's native `cancel`
        // handler closes the modal and fires `sail-close`.
      }

      input?.addEventListener('input', onInput)
      input?.addEventListener('keydown', onKeydown)

      // Pointer commit. mousedown (before any blur/focus shuffle) selects the row.
      const list = listEl() as HTMLUListElement | null
      const onListMousedown = (e: MouseEvent) => {
        const li = (e.target as HTMLElement | null)?.closest?.('li[data-id]') as HTMLElement | null
        if (!li || li.getAttribute('aria-disabled') === 'true') return
        e.preventDefault()
        const id = li.dataset.id ?? ''
        const match = (visible() as VisibleItem[]).find((v) => v.id === id)
        selectByItem(match)
      }
      list?.addEventListener('mousedown', onListMousedown)

      // `show()` (not `open()`) — `open` is a declared <prop>, so its accessor
      // owns that name; consumers open imperatively via `el.show()` or set
      // `el.open = true` directly.
      host.expose?.({
        show: () => {
          h.open = true
        },
        close: () => {
          h.open = false
        },
        toggle: () => {
          h.open = !_ctx.open()
        },
      })

      return () => {
        stopOpenSync()
        input?.removeEventListener('input', onInput)
        input?.removeEventListener('keydown', onKeydown)
        list?.removeEventListener('mousedown', onListMousedown)
      }
    })

    return {
      itemsSig, query, highlight, listId,
      dialogBridge, captureInput, captureList,
      visible, navigable, safeHighlight, highlightId, activeId, rows,
    }
  })

</controller>
<dialog use:dialogBridge size={size()} backdrop="blur">
  <column gap="0" style({ box: minH(0), paint: fg(foreground) })>    <span style({ box: display(contents), text: fontFamily(body) })>      <row align="center" gap="2" style({  box: grow(0).pad(3).padX(4), paint: borderBWidth(1).borderB(border-subtle)  })>
      <icon name="search" size="sm" style({ box: grow(0) })></icon>
      <input type="text" flush autocomplete="off" autocapitalize="off" autocorrect="off" spellcheck="false" role="combobox" aria-expanded="true" aria-autocomplete="list" aria-controls={listId()} aria-activedescendant={activeId()} aria-label={label()} placeholder={placeholder()} bind:value={query} use:captureInput style({ box: grow(1).minW(0), text: fontFamily(body).fontSize(base).leading(normal) })>
    </row>
    <ul role="listbox" id={listId()} aria-label={label()} use:captureList style({  box: grow(1).maxH(panel-vh-sm).pad(1).overflowY(auto).minH(0)  })>
      <for {rows() as row} key={row.key}>
        <if {row.isHeading}>
          <li role="presentation" style({ box: padY(2).padB(1), paint: fg(muted-foreground).select(none), text: fontSize(xs).fontWeight(semibold).tracking(wide).textCase(uppercase) })>{row.heading}</li>
        {else => <li role="option" id={row.optId} data-id={row.id} data-value={row.value} aria-selected={!row.disabled && row.id === highlightId() ? 'true' : 'false'} aria-disabled={row.disabled ? 'true' : 'false'} style({ box: pad(2), paint: group().radius(sm).fg(foreground).cursor(pointer).select(none), text: fontSize(sm), when: { selected: { paint: bg(selected-bg) }, disabled: { paint: fg(muted-foreground).cursor(default).opacity(50) } } })>
            <if {row.icon !== ''}>
              <icon name={row.icon} size="sm" style({ box: grow(0), paint: fg(muted-foreground), when: { groupSelected: { paint: fg(foreground) } } })></icon>
            </if>
            <span style({ box: grow(1).minW(0), text: truncate() })>{row.label}</span>
            <if {row.hint !== ''}>
              <span style({  box: grow(0).ml(auto), paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono)  })>{row.hint}</span>
            </if>
          </li>}</if>
      </for>
      <if {rows().length === 0}>
        <li role="status" style({ box: pad(6).padX(4), paint: fg(muted-foreground), text: fontSize(sm).textAlign(center) })>{emptyLabel()}</li>
      </if>
    </ul>
    </span>
</column>
</dialog>
