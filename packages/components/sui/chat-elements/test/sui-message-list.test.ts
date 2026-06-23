// Behavioral suite for <sui-message-list> — keyed reconciliation, windowing,
// and sticky-bottom auto-scroll. Renders <sui-message> children, so import
// both compiled outputs (AOT registration on import).
import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-message/sui-message.compiled.js'
import '../src/sui-message-list/sui-message-list.compiled.js'

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
  role?: string
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
  markup = '<sui-message-list></sui-message-list>',
): Promise<MessageListEl> => {
  const c = mk()
  c.innerHTML = markup
  const el = c.querySelector('sui-message-list')! as MessageListEl
  await flush()
  return el
}
const scrollOf = (el: MessageListEl): HTMLElement =>
  el.shadowRoot!.querySelector<HTMLElement>('[part="scroll"]')!
const renderedMessages = (el: MessageListEl): NodeListOf<HTMLElement> =>
  el.shadowRoot!.querySelectorAll<HTMLElement>('sui-message')

describe('<sui-message-list>', () => {
  it('registers with the global custom-element registry', () => {
    expect(customElements.get('sui-message-list')).toBeDefined()
  })

  it('renders one <sui-message> per keyed item, in order, set via the items prop', async () => {
    const el = await mountList()
    el.items = [
      { id: 'a', role: 'user', content: 'one' },
      { id: 'b', role: 'assistant', content: 'two' },
    ]
    await flush()
    const msgs = renderedMessages(el)
    expect(msgs.length).toBe(2)
    expect(msgs[0]!.getAttribute('role')).toBe('user')
    expect(msgs[0]!.textContent).toBe('one')
    expect(msgs[1]!.getAttribute('role')).toBe('assistant')
    expect(msgs[1]!.textContent).toBe('two')
  })

  it('windows a long thread: renders only a slice while the spacer keeps the scrollbar honest', async () => {
    const el = await mountList()
    el.items = Array.from({ length: 5000 }, (_, i) => ({
      id: String(i),
      role: i % 2 ? 'assistant' : 'user',
      content: `msg ${i}`,
    }))
    await flush()
    const msgs = renderedMessages(el)
    // A window (+overscan), FAR fewer than 5000 DOM bubbles.
    expect(msgs.length).toBeGreaterThan(0)
    expect(msgs.length).toBeLessThan(500)
    // The spacer encodes the FULL thread height so the scrollbar is honest.
    const spacer = el.shadowRoot!.querySelector<HTMLElement>('[part="spacer"]')!
    expect(parseFloat(spacer.style.height || '0')).toBeGreaterThan(5000)
  })

  it('positions windowed rows at distinct, increasing virtual offsets', async () => {
    const el = await mountList()
    el.items = Array.from({ length: 2000 }, (_, i) => ({ id: String(i), content: `m${i}` }))
    await flush()
    const rows = el.shadowRoot!.querySelectorAll<HTMLElement>('[part="row"]')
    expect(rows.length).toBeGreaterThan(1)
    const offsets = Array.from(rows).map((r) => r.style.transform)
    // First row anchors at the top; every row sits at a unique offset (not
    // stacked at 0).
    expect(offsets[0]).toContain('translateY(0px)')
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
    expect(msgs[1]!.textContent).toBe('world')
  })

  it('streams: growing the last message in place updates its text without remounting', async () => {
    const el = await mountList()
    el.items = [{ id: 'a', content: 'The ' }]
    await flush()
    const bubble = renderedMessages(el)[0]!
    expect(bubble.textContent).toBe('The ')
    el.items = [{ id: 'a', content: 'The quick brown fox' }]
    await flush()
    expect(renderedMessages(el)[0]).toBe(bubble)
    expect(bubble.textContent).toBe('The quick brown fox')
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
    const el = await mountList('<sui-message-list sticky-bottom="false"></sui-message-list>')
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
      '<sui-message-list><div slot="empty">No messages</div></sui-message-list>',
    )
    expect(el.shadowRoot!.querySelector('slot[name="empty"]')).not.toBeNull()
    el.items = [{ id: 'a', content: 'hi' }]
    await flush()
    expect(el.shadowRoot!.querySelector('slot[name="empty"]')).toBeNull()
  })
})
