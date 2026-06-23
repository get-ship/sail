import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-dropdown-menu/sui-dropdown-menu.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!
const panelEl = (el: HTMLElement) => shadow(el).querySelector('[part="panel"]') as HTMLElement | null
const triggerWrap = (el: HTMLElement) => shadow(el).querySelector('.trigger-wrap') as HTMLElement | null

function makeMenu(attrs: Record<string, string | boolean> = {}, items = '') {
  return mount(
    reg,
    'sui-dropdown-menu',
    attrs,
    `<button slot="trigger">Menu</button>${items}`,
  )
}

function makeMenuWithItems(attrs: Record<string, string | boolean> = {}) {
  return makeMenu(attrs, `
    <button data-value="rename">Rename</button>
    <button data-value="delete">Delete</button>
    <button data-value="copy">Copy</button>
  `)
}

describe('<sui-dropdown-menu>', () => {

  it('closed by default', () => {
    const { el } = makeMenu()
    expect(el.hasAttribute('open')).toBe(false)
    expect(el.getAttribute('data-state')).toBe('closed')
  })

  it('renders a trigger slot', () => {
    const { el } = makeMenu()
    const slot = shadow(el).querySelector('slot[name="trigger"]')
    expect(slot).not.toBeNull()
  })

  it('renders a panel element with role=menu', () => {
    const { el } = makeMenu()
    const panel = panelEl(el)
    expect(panel).not.toBeNull()
    expect(panel!.getAttribute('role')).toBe('menu')
  })

  it('trigger has aria-haspopup=menu and aria-controls after flush', async () => {
    const { el } = makeMenu()
    await flush()
    const trigger = el.querySelector('button[slot="trigger"]') as HTMLElement
    expect(trigger.getAttribute('aria-haspopup')).toBe('menu')
    expect(trigger.getAttribute('aria-controls')).toBeTruthy()
  })

  it('trigger aria-expanded=false when closed', async () => {
    const { el } = makeMenu()
    await flush()
    const trigger = el.querySelector('button[slot="trigger"]') as HTMLElement
    expect(trigger.getAttribute('aria-expanded')).toBe('false')
  })

  it('items get role=menuitem after setup', async () => {
    const { el } = makeMenuWithItems()
    await flush()
    const items = el.querySelectorAll('button[data-value]')
    expect(items[0].getAttribute('role')).toBe('menuitem')
    expect(items[1].getAttribute('role')).toBe('menuitem')
  })

  it('items get roving tabindex — first=0, rest=-1', async () => {
    const { el } = makeMenuWithItems()
    await flush()
    const items = Array.from(el.querySelectorAll<HTMLElement>('button[data-value]'))
    expect(items[0].getAttribute('tabindex')).toBe('0')
    expect(items[1].getAttribute('tabindex')).toBe('-1')
    expect(items[2].getAttribute('tabindex')).toBe('-1')
  })

  it('trigger click opens panel (showPopover called)', async () => {
    const { el } = makeMenu()
    const tw = triggerWrap(el)!
    let popovers = 0
    const p = panelEl(el)!
    const orig = p.showPopover
    p.showPopover = () => { popovers++; orig?.call(p) }
    tw.click()
    await flush()
    expect(popovers).toBe(1)
  })

  it('fires sui-select with data-value on item click', async () => {
    const { el } = makeMenuWithItems({ open: true })
    await flush()
    let detail: { value: string } | null = null
    el.addEventListener('sui-select', (e) => { detail = (e as CustomEvent).detail })
    const deleteBtn = el.querySelector<HTMLButtonElement>('button[data-value="delete"]')!
    deleteBtn.click()
    await flush()
    expect(detail).toEqual({ value: 'delete' })
  })

  it('panel closes after item click', async () => {
    const { el } = makeMenuWithItems({ open: true })
    await flush()
    const p = panelEl(el)!
    let hideCount = 0
    const orig = p.hidePopover
    p.hidePopover = () => { hideCount++; orig?.call(p) }
    el.querySelector<HTMLButtonElement>('button[data-value="rename"]')!.click()
    await flush()
    expect(hideCount).toBe(1)
  })

  it('ArrowDown key moves focus to next item', async () => {
    const { el } = makeMenuWithItems({ open: true })
    await flush()
    const items = Array.from(el.querySelectorAll<HTMLElement>('button[data-value]'))
    items[0].focus()
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowDown', bubbles: true }))
    expect(document.activeElement).toBe(items[1])
  })

  it('ArrowUp key wraps to last item', async () => {
    const { el } = makeMenuWithItems({ open: true })
    await flush()
    const items = Array.from(el.querySelectorAll<HTMLElement>('button[data-value]'))
    items[0].focus()
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowUp', bubbles: true }))
    expect(document.activeElement).toBe(items[items.length - 1])
  })

  it('Home key focuses first item', async () => {
    const { el } = makeMenuWithItems({ open: true })
    await flush()
    const items = Array.from(el.querySelectorAll<HTMLElement>('button[data-value]'))
    items[2].focus()
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Home', bubbles: true }))
    expect(document.activeElement).toBe(items[0])
  })

  it('End key focuses last item', async () => {
    const { el } = makeMenuWithItems({ open: true })
    await flush()
    const items = Array.from(el.querySelectorAll<HTMLElement>('button[data-value]'))
    items[0].focus()
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'End', bubbles: true }))
    expect(document.activeElement).toBe(items[items.length - 1])
  })

  it('Escape key hides panel', async () => {
    const { el } = makeMenuWithItems({ open: true })
    await flush()
    const p = panelEl(el)!
    let hideCount = 0
    const orig = p.hidePopover
    p.hidePopover = () => { hideCount++; orig?.call(p) }
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))
    expect(hideCount).toBe(1)
  })

  it('disconnect — no error', () => {
    const { container } = makeMenuWithItems()
    expect(container.isConnected).toBe(true)
    container.remove()
    expect(container.isConnected).toBe(false)
  })
})

describe('<sui-dropdown-menu> — aria-expanded follows open state', () => {
  it('trigger click flips aria-expanded to true via the panel toggle', async () => {
    const { el } = makeMenuWithItems()
    await flush()
    const trigger = el.querySelector('button[slot="trigger"]') as HTMLElement
    expect(trigger.getAttribute('aria-expanded')).toBe('false')
    ;(shadow(el).querySelector('.trigger-wrap') as HTMLElement).click()
    await flush()
    expect(trigger.getAttribute('aria-expanded')).toBe('true')
    expect(el.getAttribute('data-state')).toBe('open')
  })
})

describe('<sui-dropdown-menu> — keyboard activation + typeahead', () => {
  it('Enter activates the focused item — sui-select carries the value', async () => {
    const { el } = makeMenuWithItems({ open: true })
    await flush()
    const items = Array.from(el.querySelectorAll<HTMLElement>('button[data-value]'))
    items[1].focus()
    let evt: CustomEvent | null = null
    el.addEventListener('sui-select', (e) => { evt = e as CustomEvent })
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }))
    await flush()
    expect(evt).not.toBeNull()
    expect(evt!.detail).toEqual({ value: 'delete' })
  })

  it('Space activates the focused item — sui-select carries the value', async () => {
    const { el } = makeMenuWithItems({ open: true })
    await flush()
    const items = Array.from(el.querySelectorAll<HTMLElement>('button[data-value]'))
    items[2].focus()
    let evt: CustomEvent | null = null
    el.addEventListener('sui-select', (e) => { evt = e as CustomEvent })
    el.dispatchEvent(new KeyboardEvent('keydown', { key: ' ', bubbles: true }))
    await flush()
    expect(evt!.detail).toEqual({ value: 'copy' })
  })

  it('roving tabindex tracks arrow navigation', async () => {
    const { el } = makeMenuWithItems({ open: true })
    await flush()
    const items = Array.from(el.querySelectorAll<HTMLElement>('button[data-value]'))
    items[0].focus()
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowDown', bubbles: true }))
    expect(items[1].getAttribute('tabindex')).toBe('0')
    expect(items[0].getAttribute('tabindex')).toBe('-1')
    expect(items[2].getAttribute('tabindex')).toBe('-1')
  })

  it('typeahead: printable key focuses the next item whose label matches', async () => {
    // Items: Rename, Delete, Copy.
    const { el } = makeMenuWithItems({ open: true })
    await flush()
    const items = Array.from(el.querySelectorAll<HTMLElement>('button[data-value]'))
    items[0].focus()
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'c', bubbles: true }))
    expect(document.activeElement).toBe(items[2]) // Copy
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'd', bubbles: true }))
    // New buffer would be 'cd' (no match) — buffer keeps accumulating within
    // the 350ms window, so focus stays where it was.
    expect(document.activeElement).toBe(items[2])
  })

  it('Tab closes the panel (focus moves on naturally)', async () => {
    const { el } = makeMenuWithItems({ open: true })
    await flush()
    const p = panelEl(el)!
    let hid = 0
    const orig = p.hidePopover
    p.hidePopover = () => { hid++; orig?.call(p) }
    const items = Array.from(el.querySelectorAll<HTMLElement>('button[data-value]'))
    items[0].focus()
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Tab', bubbles: true }))
    expect(hid).toBe(1)
  })
})

describe('<sui-dropdown-menu> — event contract', () => {
  it('sui-select: bubbles + composed with {value} payload', async () => {
    const { el, container } = makeMenuWithItems({ open: true })
    await flush()
    let evt: CustomEvent | null = null
    // Listen on the container, not the host — proves bubbling for real.
    container.addEventListener('sui-select', (e) => { evt = e as CustomEvent })
    el.querySelector<HTMLButtonElement>('button[data-value="rename"]')!.click()
    await flush()
    expect(evt).not.toBeNull()
    expect(evt!.bubbles).toBe(true)
    expect(evt!.composed).toBe(true)
    expect(evt!.detail).toEqual({ value: 'rename' })
  })
})
