<!--
  <sail-message-list> — a keyed, windowed, scrollable column of
  <sail-message> bubbles. Items arrive via the `items` JS-property
  (`items={...}`); each must carry a stable `id` for keyed
  reconciliation. With `sticky-bottom` (default), the list auto-scrolls
  to the newest message unless the user has scrolled up to read history.

  Long threads window through the shared `createVirtualizer`: only the
  visible slice (+overscan) of <sail-message> elements is in the DOM, so
  a 10k-message thread renders ~a dozen bubbles, not 10k. A spacer
  encodes the full thread height so the scrollbar stays honest; each row
  is absolutely positioned at its virtual offset.

  Item shape (all but `id` / `content` optional):
    { id, content, from?, contentType?, timestamp?, status? }

  Pure framework primitive — no SDK, no data fetching. Consumers feed
  their own array.

  Props
  ─────────────────────────────────────────────────────────────────────
  sticky-bottom  (bool)  Auto-scroll to newest unless scrolled up.
  ─────────────────────────────────────────────────────────────────────
-->

<scope>
  props: {
    'sticky-bottom' = true
  }
</scope>

<controller>

import type { Action, Derived, ControllerScope } from '@ship/sail-runtime'
import {
  createVirtualizer,
  type Virtual,
  type VirtualItem,
} from '@ship/sail-runtime/virtual'
import type { Scope } from './sail-message-list.sail'

interface MessageItem {
  id: string
  content?: unknown
  from?: string
  contentType?: string
  timestamp?: string
  status?: string
}

/** A windowed message: the virtual-item geometry zipped with its source item so
 *  the template renders `<for {windowItems() as w} key={w.key}>` with no index
 *  arithmetic — only the ~viewport+overscan slice allocates, never the whole
 *  thread. */
interface WindowMessage {
  key: VirtualItem['key']
  index: number
  start: number
  size: number
  item: MessageItem
}

// How close to the bottom (px) still counts as "pinned" — small scroll
// drifts shouldn't disable auto-scroll.
const BOTTOM_THRESHOLD = 48

// Estimated message-bubble height (px) seeding the virtualizer before a row
// measures itself. Real heights supersede it the moment `measureElement` runs.
const ESTIMATED_MESSAGE_HEIGHT = 72

export default controller((_ctx: ControllerScope<Scope>) => {
    const items = signal<MessageItem[]>([])
    const scrollEl = signal<HTMLElement | null>(null)

    // The virtualizer windows the thread over the scroll element. `count`
    // tracks `items().length`; `getScrollElement` is the ref thunk (null until
    // captureScroll mounts). Keyed by item id so `{#for}` recycles rows on
    // append/reorder rather than re-creating the whole window.
    const virt: Virtual = createVirtualizer({
      count: () => items().length,
      estimateSize: () => ESTIMATED_MESSAGE_HEIGHT,
      getScrollElement: () => scrollEl(),
      overscan: 6,
      getItemKey: (i) => items()[i]?.id ?? i,
    })

    const totalSize = virt.totalSize
    // Map ONLY the visible window to WindowMessages. Recomputes when the window
    // identity breaks (a real scroll/measure frame) or `items` changes.
    const windowItems: Derived<WindowMessage[]> = derived<WindowMessage[]>(() => {
      const all = items()
      const out: WindowMessage[] = []
      for (const it of virt.virtualItems() as readonly VirtualItem[]) {
        const item = all[it.index]
        if (!item) continue
        out.push({ key: it.key, index: it.index, start: it.start, size: it.size, item })
      }
      return out
    })

    mount((host) => {
      // `items` JS-property: `el.items = [...]`.
      const init = (host as unknown as { items?: MessageItem[] }).items
      if (init != null) items.set(init)
      Object.defineProperty(host, 'items', {
        configurable: true,
        get: () => items(),
        set: (v: MessageItem[]) => { items.set(v) },
      })

      // Tracks whether the viewport is pinned to the bottom. A user scroll-up
      // flips this false so new items don't yank them down; scrolling back to the
      // bottom re-pins.
      let pinned = true

      const isAtBottom = (el: HTMLElement): boolean =>
        el.scrollHeight - el.scrollTop - el.clientHeight <= BOTTOM_THRESHOLD

      const onScroll = () => {
        const el = scrollEl()
        if (el) pinned = isAtBottom(el)
      }

      const stopScrollBind = effect(() => {
        const el = scrollEl()
        if (!el) return
        el.addEventListener('scroll', onScroll, { passive: true })
        return () => el.removeEventListener('scroll', onScroll)
      })

      // On every items change, scroll the newest message into view when sticky +
      // pinned. Goes through the virtualizer (`scrollToIndex`) so the windowing
      // engine offsets correctly — the rendered window holds only a slice, so the
      // raw scrollHeight no longer spans the full thread.
      const stopAutoScroll = effect(() => {
        const count = items().length // track items
        if (!scrollEl() || !_ctx.stickyBottom() || !pinned || count === 0) return
        // Defer to after the keyed window has rendered the new rows.
        queueMicrotask(() => { virt.scrollToIndex(count - 1, { align: 'end' }) })
      })

      return () => {
        stopScrollBind()
        stopAutoScroll()
        virt.dispose()
      }
    })

    const captureScroll: Action<void> = (node) => {
      scrollEl.set(node as HTMLElement)
      return { destroy: () => scrollEl.set(null) }
    }
    return { items, captureScroll, virt, totalSize, windowItems }
  })

</controller>
<scroll-area
  orientation="vertical"
  use:captureScroll style({ box: position(relative).h(full).minH(0).pad(3) })>  <span style({ box: display(contents), text: fontFamily(body) })>    <div use:virtualSpacer={totalSize()} style({ box: position(relative).w(full).h(bind(rowsH)) })>
    <for {windowItems() as w} key={w.key}>
      <column gap="0" use:virtualRow={w.start} style({ box: position(absolute).left(3).right(3), paint: translateY(bind(rowY)) })>
        <sail-message
          from={w.item.from || 'assistant'}
          content-type={w.item.contentType || 'text'}
          timestamp={w.item.timestamp || ''}
          status={w.item.status || 'sent'}
        >{w.item.content}</sail-message>
      </column>
    </for>
  </div>
  <if {items().length === 0}>
    <center><slot name="empty"></slot></center>
  </if>
  </span>
</scroll-area>
