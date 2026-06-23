import {
  controller,
  derived,
  effect,
  ref,
  signal,
} from '@ship/sui-runtime'
import type { Derived, ViewScope } from '@ship/sui-runtime'
import {
  createVirtualizer,
  type Virtual,
  type VirtualItem,
} from '@ship/sui-runtime/virtual'
import type { Scope } from './sui-message-list.sui'

interface MessageItem {
  id: string
  content?: unknown
  role?: string
  contentType?: string
  timestamp?: string
  status?: string
}

/** A windowed message: the virtual-item geometry zipped with its source item so
 *  the template renders `{#for w of windowItems() key={w.key}}` with no index
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

export default controller({
  scope: (_ctx: ViewScope<Scope>) => {
    const items = signal<MessageItem[]>([])
    const scrollEl = ref<HTMLElement>(null)

    // The virtualizer windows the thread over the scroll element. `count`
    // tracks `items().length`; `getScrollElement` is the ref thunk (null until
    // `bind:this` mounts). Keyed by item id so `{#for}` recycles rows on
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

    return { items, scrollEl, virt, totalSize, windowItems }
  },

  mount(state, host) {
    // `items` JS-property: `el.items = [...]`.
    const init = (host as unknown as { items?: MessageItem[] }).items
    if (init != null) state.items = init
    Object.defineProperty(host, 'items', {
      configurable: true,
      get: () => state.items,
      set: (v: MessageItem[]) => { state.items = v },
    })

    // Tracks whether the viewport is pinned to the bottom. A user scroll-up
    // flips this false so new items don't yank them down; scrolling back to the
    // bottom re-pins.
    let pinned = true

    const isAtBottom = (el: HTMLElement): boolean =>
      el.scrollHeight - el.scrollTop - el.clientHeight <= BOTTOM_THRESHOLD

    const onScroll = () => {
      const el = state.scrollEl
      if (el) pinned = isAtBottom(el)
    }

    const stopScrollBind = effect(() => {
      const el = state.scrollEl
      if (!el) return
      el.addEventListener('scroll', onScroll, { passive: true })
      return () => el.removeEventListener('scroll', onScroll)
    })

    // On every items change, scroll the newest message into view when sticky +
    // pinned. Goes through the virtualizer (`scrollToIndex`) so the windowing
    // engine offsets correctly — the rendered window holds only a slice, so the
    // raw scrollHeight no longer spans the full thread.
    const stopAutoScroll = effect(() => {
      const count = state.items.length // track items
      if (!state.scrollEl || !state.stickyBottom || !pinned || count === 0) return
      // Defer to after the keyed window has rendered the new rows.
      queueMicrotask(() => { state.virt.scrollToIndex(count - 1, { align: 'end' }) })
    })

    return () => {
      stopScrollBind()
      stopAutoScroll()
      state.virt.dispose()
    }
  },
})
