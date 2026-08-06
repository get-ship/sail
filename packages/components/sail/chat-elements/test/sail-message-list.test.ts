// Behavioral suite for <sail-message-list> — keyed reconciliation, windowing,
// and sticky-bottom auto-scroll. Renders <sail-message> children, so import
// both compiled outputs (AOT registration on import).
import { afterEach, describe, expect, it } from 'vitest'
import '../src/sail-message/sail-message.compiled.js'
import '../src/sail-message-list/sail-message-list.compiled.js'

const containers: HTMLElement[] = []
const mk = (): HTMLElement => {
  const c = document.createElement('div')
  document.body.appendChild(c)
  containers.push(c)
  return c
}
afterEach(() => {
  for (const c of containers.splice(0)) c.remove()
})
const flush = () => new Promise((r) => setTimeout(r, 0))
// Drain deferred work (microtasks + jsdom rAF, which runs on a ~16ms timer):
// the virtualizer defers its post-measure re-scroll, so a freshly-mounted
// pinned list keeps scrolling for a frame or two after `items` is set. Tests
// that install a scroll spy must let that initial settle land first.
const settle = async () => {
  for (let i = 0; i < 3; i++) await new Promise((r) => setTimeout(r, 20))
}

interface MsgItem {
  id: string
  from?: string
  content?: string
  contentType?: string
  timestamp?: string
  status?: string
}
type MessageListEl = HTMLElement & { items: MsgItem[] }

/** Give the scroll viewport a measurable, scrollable geometry. JSDOM reports 0
 *  for all layout, so the controller's pinned/at-bottom check has nothing to
 *  read; stub the metrics it consults. */
function stubScrollGeometry(
  scroll: HTMLElement,
  opts: { scrollHeight: number; clientHeight: number; scrollTop?: number },
): void {
  let top = opts.scrollTop ?? 0
  Object.defineProperty(scroll, 'scrollHeight', {
    configurable: true,
    get: () => opts.scrollHeight,
  })
  Object.defineProperty(scroll, 'clientHeight', {
    configurable: true,
    get: () => opts.clientHeight,
  })
  Object.defineProperty(scroll, 'offsetHeight', {
    configurable: true,
    get: () => opts.clientHeight,
  })
  Object.defineProperty(scroll, 'scrollTop', {
    configurable: true,
    get: () => top,
    set: (v: number) => { top = v },
  })
}

/** Spy on the viewport's `scrollTo` — the channel the virtualizer uses to move
 *  the viewport (JSDOM's native `scrollTo` is an inert no-op, so observing the
 *  CALL is the behavioral signal that auto-scroll fired). */
function spyScrollTo(scroll: HTMLElement): { tops: number[] } {
  const rec = { tops: [] as number[] }
  Object.defineProperty(scroll, 'scrollTo', {
    configurable: true,
    writable: true,
    value: (arg: number | ScrollToOptions, y?: number) => {
      if (typeof arg === 'object') rec.tops.push(arg.top ?? 0)
      else rec.tops.push(y ?? 0)
    },
  })
  return rec
}

const mountList = async (
  markup = '<sail-message-list></sail-message-list>',
): Promise<MessageListEl> => {
  const c = mk()
  c.innerHTML = markup
  const el = c.querySelector('sail-message-list')! as MessageListEl
  await flush()
  return el
}
// Light DOM, no class hooks. The component's root IS the scroll viewport; the
// spacer is the sized box inside it, and each windowed row is the box that
// holds one <sail-message>.
const scrollOf = (el: MessageListEl): HTMLElement =>
  el.firstElementChild as HTMLElement
const spacerOf = (el: MessageListEl): HTMLElement =>
  scrollOf(el).firstElementChild as HTMLElement
const rowsOf = (el: MessageListEl): HTMLElement[] =>
  Array.from(el.querySelectorAll<HTMLElement>('sail-message')).map(
    (m) => m.parentElement as HTMLElement,
  )
const renderedMessages = (el: MessageListEl): NodeListOf<HTMLElement> =>
  el.querySelectorAll<HTMLElement>('sail-message')
// A message renders its meta row (from, timestamp) alongside the projected turn,
// so the ITEM's text is what landed in that message's default slot.
const contentOf = (msg: HTMLElement): string =>
  msg.querySelector<HTMLSlotElement>('slot:not([name])')!.textContent!

describe('<sail-message-list>', () => {
  it('registers with the global custom-element registry', () => {
    expect(customElements.get('sail-message-list')).toBeDefined()
  })

  it('renders one <sail-message> per keyed item, in order, set via the items prop', async () => {
    const el = await mountList()
    el.items = [
      { id: 'a', from: 'user', content: 'one' },
      { id: 'b', from: 'assistant', content: 'two' },
    ]
    await flush()
    const msgs = renderedMessages(el)
    expect(msgs.length).toBe(2)
    expect(msgs[0]!.getAttribute('from')).toBe('user')
    expect(msgs[0]!.getAttribute('role')).toBeNull()
    expect(contentOf(msgs[0]!)).toBe('one')
    expect(msgs[1]!.getAttribute('from')).toBe('assistant')
    expect(contentOf(msgs[1]!)).toBe('two')
  })

  it('windows a long thread: renders only a slice while the spacer keeps the scrollbar honest', async () => {
    const el = await mountList()
    el.items = Array.from({ length: 5000 }, (_, i) => ({
      id: String(i),
      from: i % 2 ? 'assistant' : 'user',
      content: `msg ${i}`,
    }))
    await flush()
    const msgs = renderedMessages(el)
    // A window (+overscan), FAR fewer than 5000 DOM bubbles.
    expect(msgs.length).toBeGreaterThan(0)
    expect(msgs.length).toBeLessThan(500)
    // The spacer encodes the FULL thread height so the scrollbar is honest.
    // `use:virtualSpacer` (packages/sail/runtime/src/actions/virtual-row.ts)
    // writes the height as the `--rows-h` custom property — sx's
    // `h-(--rows-h)` atom is what turns it into layout — never a direct
    // `.style.height` write (the floor: no inline style outside a `use:`
    // action's own custom-property seam).
    const spacer = spacerOf(el)
    expect(parseFloat(spacer.style.getPropertyValue('--rows-h') || '0')).toBeGreaterThan(5000)
  })

  it('positions windowed rows at distinct, increasing virtual offsets', async () => {
    const el = await mountList()
    el.items = Array.from({ length: 2000 }, (_, i) => ({ id: String(i), content: `m${i}` }))
    await flush()
    const rows = rowsOf(el)
    expect(rows.length).toBeGreaterThan(1)
    // `use:virtualRow` writes the offset as the `--row-y` custom property
    // (sx's `translate-y-(--row-y)` atom does the actual `translate:`), not
    // a direct `.style.transform` write.
    const offsets = rows.map((r) => r.style.getPropertyValue('--row-y'))
    // First row anchors at the top; every row sits at a unique offset (not
    // stacked at 0).
    expect(offsets[0]).toBe('0px')
    expect(new Set(offsets).size).toBe(offsets.length)
  })

  it('streams: appending a message adds one bubble and recycles the prior DOM node', async () => {
    const el = await mountList()
    el.items = [{ id: 'a', content: 'hello' }]
    await flush()
    expect(renderedMessages(el).length).toBe(1)
    const firstBubble = renderedMessages(el)[0]!
    el.items = [...el.items, { id: 'b', content: 'world' }]
    await flush()
    const msgs = renderedMessages(el)
    expect(msgs.length).toBe(2)
    // Keyed window recycles the existing 'a' bubble rather than recreating it.
    expect(msgs[0]).toBe(firstBubble)
    expect(contentOf(msgs[1]!)).toBe('world')
  })

  it('streams: growing the last message in place updates its text without remounting', async () => {
    const el = await mountList()
    el.items = [{ id: 'a', content: 'The ' }]
    await flush()
    const bubble = renderedMessages(el)[0]!
    expect(contentOf(bubble)).toBe('The ')
    el.items = [{ id: 'a', content: 'The quick brown fox' }]
    await flush()
    expect(renderedMessages(el)[0]).toBe(bubble)
    expect(contentOf(bubble)).toBe('The quick brown fox')
  })

  it('pinned-to-bottom: a new streamed message auto-scrolls toward the newest', async () => {
    const el = await mountList()
    const scroll = scrollOf(el)
    stubScrollGeometry(scroll, { scrollHeight: 1000, clientHeight: 200, scrollTop: 800 })
    el.items = [{ id: 'a', content: 'one' }]
    await flush()
    const spy = spyScrollTo(scroll)
    el.items = [...el.items, { id: 'b', content: 'two' }]
    await flush()
    await flush()
    expect(spy.tops.length).toBeGreaterThan(0)
    expect(Math.max(...spy.tops)).toBeGreaterThan(0)
  })

  it('scrolled-up: a new message does NOT yank the viewport back to the bottom', async () => {
    const el = await mountList()
    const scroll = scrollOf(el)
    el.items = Array.from({ length: 50 }, (_, i) => ({ id: String(i), content: `m${i}` }))
    // Let the initial pinned mount fully settle BEFORE unpinning, so its
    // deferred auto-scroll isn't misattributed to the append below.
    await settle()
    // User scrolls up to read history — far from the bottom → un-pins.
    stubScrollGeometry(scroll, { scrollHeight: 4000, clientHeight: 200, scrollTop: 10 })
    scroll.dispatchEvent(new Event('scroll'))
    await flush()
    const spy = spyScrollTo(scroll)
    // A new message streams in while the user is reading up-thread.
    el.items = [...el.items, { id: 'new', content: 'incoming' }]
    await settle()
    // The viewport is left where the user parked it — nothing scrolled it
    // toward the newest (bottom) message.
    expect(spy.tops.length).toBe(0)
  })

  it('re-pins when the user scrolls back to the bottom, resuming auto-scroll', async () => {
    const el = await mountList()
    const scroll = scrollOf(el)
    el.items = Array.from({ length: 30 }, (_, i) => ({ id: String(i), content: `m${i}` }))
    await flush()
    stubScrollGeometry(scroll, { scrollHeight: 3000, clientHeight: 200, scrollTop: 5 })
    scroll.dispatchEvent(new Event('scroll'))
    await flush()
    scroll.scrollTop = 2800
    scroll.dispatchEvent(new Event('scroll'))
    await flush()
    const spy = spyScrollTo(scroll)
    el.items = [...el.items, { id: 'tail', content: 'caught up' }]
    await flush()
    await flush()
    expect(spy.tops.length).toBeGreaterThan(0)
  })

  it('sticky-bottom=false disables auto-scroll entirely', async () => {
    const el = await mountList('<sail-message-list sticky-bottom="false"></sail-message-list>')
    const scroll = scrollOf(el)
    stubScrollGeometry(scroll, { scrollHeight: 1000, clientHeight: 200, scrollTop: 0 })
    el.items = [{ id: 'a', content: 'one' }]
    await flush()
    const spy = spyScrollTo(scroll)
    el.items = [...el.items, { id: 'b', content: 'two' }]
    await flush()
    await flush()
    expect(spy.tops.length).toBe(0)
  })

  it('shows the empty slot only while there are no messages', async () => {
    const el = await mountList(
      '<sail-message-list><div slot="empty">No messages</div></sail-message-list>',
    )
    expect(el.querySelector('slot[name="empty"]')).not.toBeNull()
    el.items = [{ id: 'a', content: 'hi' }]
    await flush()
    expect(el.querySelector('slot[name="empty"]')).toBeNull()
  })
})
