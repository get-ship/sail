<!--
  <sail-message-list> — keyed, windowed scroll of <sail-message> bubbles.
  Items via `items` identity (stable `id` required). sticky-bottom auto-scrolls
  unless the user has scrolled up. Virtualizer keeps ~viewport+overscan in DOM.
-->

<scope>
  props: {
    'sticky-bottom' = true
  }
</scope>

<controller>

import type { Action, Derived, ControllerScope } from '@ship/sail-runtime'
import type { Virtual, VirtualItem } from '@ship/sail-runtime/virtual'
import type { Scope } from './sail-message-list.sail'

interface MessageItem {
  id: string
  content?: unknown
  from?: string
  contentType?: string
  timestamp?: string
  status?: string
}

interface WindowMessage {
  key: VirtualItem['key']
  index: number
  start: number
  size: number
  item: MessageItem
}

/** px slack still counting as pinned to bottom. */
const BOTTOM_THRESHOLD = 48

/** Seed height before measureElement runs. */
const ESTIMATED_MESSAGE_HEIGHT = 72

export default controller((_ctx: ControllerScope<Scope>) => {
    const items = signal<MessageItem[]>([])
    const scrollEl = signal<HostBag | null>(null)

    const virt: Virtual = createVirtualizer({
      count: () => items().length,
      estimateSize: () => ESTIMATED_MESSAGE_HEIGHT,
      getScrollElement: () => scrollEl(),
      overscan: 6,
      getItemKey: (i) => items()[i]?.id ?? i,
    })

    const totalSize = virt.totalSize
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
      const init = (host as { items?: MessageItem[] }).items
      if (init != null) items.set(init)
      defineHostProperty(asSailHost(host) as HostBag, 'items', {
        get: () => items(),
        set: (v: MessageItem[]) => { items.set(v) },
      })

      let pinned = true

      const isAtBottom = (el: HostBag): boolean =>
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

      const stopAutoScroll = effect(() => {
        const count = items().length
        if (!scrollEl() || !_ctx.stickyBottom() || !pinned || count === 0) return
        queueMicrotask(() => { virt.scrollToIndex(count - 1, { align: 'end' }) })
      })

      return () => {
        stopScrollBind()
        stopAutoScroll()
        virt.dispose()
      }
    })

    const captureScroll: Action<void> = (node) => {
      scrollEl.set(node)
      return { destroy: () => scrollEl.set(null) }
    }
    return { items, captureScroll, virt, totalSize, windowItems }
  })

</controller>
<scroll-area
  orientation="vertical"
  use:captureScroll
  style({ box: place({ position: relative }).size({ h: full, min-h: 0 }).space({ pad: 3 }) })
>
  <span style({ box: flow({ display: contents }), text: type({ family: body }) })>
    <div use:virtualSpacer={totalSize()} style({ box: place({ position: relative }).size({ w: full, h: bind(rowsH) }) })>
    <for {windowItems() as w} key={w.key}>
      <column
        gap="0"
        use:virtualRow={w.start}
        style({ box: place({ position: absolute, left: 3, right: 3 }), paint: motion({ y: bind(rowY) }) })
      >
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
