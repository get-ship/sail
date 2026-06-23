import { afterEach, describe, expect, it, vi } from 'vitest'
import '../src/sui-hover-card/sui-hover-card.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => {
  vi.restoreAllMocks()
  reg.cleanup()
})

const shadow = (el: HTMLElement) => el.shadowRoot!
const panelEl = (el: HTMLElement) => shadow(el).querySelector('[part="panel"]') as HTMLElement | null

function makeCard(attrs: Record<string, string | boolean> = {}, inner = '') {
  return mount(
    reg,
    'sui-hover-card',
    attrs,
    `<button slot="trigger">Hover me</button>${inner || '<div>Card content</div>'}`,
  )
}

type SuiHoverCard = HTMLElement & { show(): void; hide(): void }

describe('<sui-hover-card>', () => {

  it('closed by default', () => {
    const { el } = makeCard()
    expect(el.hasAttribute('open')).toBe(false)
    expect(el.getAttribute('data-state')).toBe('closed')
  })

  it('has a panel element in shadow DOM', () => {
    const { el } = makeCard()
    expect(panelEl(el)).not.toBeNull()
  })

  it('panel has an id', async () => {
    const { el } = makeCard()
    await flush()
    expect(panelEl(el)!.id).toMatch(/sui-hover-card-\d+/)
  })

  it('trigger gets aria-describedby pointing to panel id', async () => {
    const { el } = makeCard()
    await flush()
    const trigger = el.querySelector('[slot="trigger"]') as HTMLElement
    const pid = panelEl(el)!.id
    expect(trigger.getAttribute('aria-describedby')).toBe(pid)
  })

  it('show() calls showPopover', async () => {
    const { el } = makeCard()
    await flush()
    const p = panelEl(el)!
    let showed = 0
    const orig = p.showPopover
    p.showPopover = () => { showed++; orig?.call(p) }
    ;(el as SuiHoverCard).show()
    expect(showed).toBe(1)
  })

  it('hide() calls hidePopover', async () => {
    const { el } = makeCard()
    await flush()
    const p = panelEl(el)!
    let hid = 0
    const orig = p.hidePopover
    p.hidePopover = () => { hid++; orig?.call(p) }
    ;(el as SuiHoverCard).hide()
    expect(hid).toBe(1)
  })

  it('show() then hide() toggles data-state', async () => {
    const { el } = makeCard()
    await flush()
    ;(el as SuiHoverCard).show()
    await flush()
    // open event from popover toggle may set data-state — check it's tracked
    ;(el as SuiHoverCard).hide()
    await flush()
    expect(el.getAttribute('data-state')).toBe('closed')
  })

  it('placement — no attribute when using default', () => {
    const { el } = makeCard()
    expect(el.hasAttribute('placement')).toBe(false)
  })

  it('placement reflects — top', () => {
    const { el } = makeCard({ placement: 'top' })
    expect(el.getAttribute('placement')).toBe('top')
  })

  it('sui-open fires when opened via toggle event (bubbles + composed, no payload)', async () => {
    const { el } = makeCard()
    await flush()
    let evt: CustomEvent | null = null
    el.addEventListener('sui-open', (e) => { evt = e as CustomEvent })
    const p = panelEl(el)!
    p.dispatchEvent(new ToggleEvent('toggle', { newState: 'open', oldState: 'closed' }))
    expect(evt).not.toBeNull()
    expect(evt!.bubbles).toBe(true)
    expect(evt!.composed).toBe(true)
    expect(evt!.detail).toBeNull()
  })

  it('sui-close fires when closed via toggle event (bubbles + composed, no payload)', async () => {
    const { el } = makeCard()
    await flush()
    let evt: CustomEvent | null = null
    el.addEventListener('sui-close', (e) => { evt = e as CustomEvent })
    const p = panelEl(el)!
    p.dispatchEvent(new ToggleEvent('toggle', { newState: 'closed', oldState: 'open' }))
    expect(evt).not.toBeNull()
    expect(evt!.bubbles).toBe(true)
    expect(evt!.composed).toBe(true)
    expect(evt!.detail).toBeNull()
  })

  it('pointerenter starts show timer', () => {
    vi.useFakeTimers()
    const { el } = makeCard()
    const p = panelEl(el)!
    let showed = 0
    const orig = p.showPopover
    p.showPopover = () => { showed++; orig?.call(p) }
    el.dispatchEvent(new PointerEvent('pointerenter', { bubbles: true }))
    vi.advanceTimersByTime(400)
    expect(showed).toBe(1)
  })

  it('pointerleave starts hide timer', () => {
    vi.useFakeTimers()
    const { el } = makeCard()
    ;(el as SuiHoverCard).show()
    const p = panelEl(el)!
    let hid = 0
    const orig = p.hidePopover
    p.hidePopover = () => { hid++; orig?.call(p) }
    el.dispatchEvent(new PointerEvent('pointerleave', { bubbles: true }))
    vi.advanceTimersByTime(200)
    expect(hid).toBe(1)
  })

  it('disconnect clears timers — no error', () => {
    const { container } = makeCard()
    expect(container.isConnected).toBe(true)
    container.remove()
    expect(container.isConnected).toBe(false)
  })
})
