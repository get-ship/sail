import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-drawer/sui-drawer.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!
const dialogEl = (el: HTMLElement) => shadow(el).querySelector('dialog') as HTMLDialogElement | null
// Anchor on the public part contract, not the styling class (.close).
const closeBtn = (el: HTMLElement) => shadow(el).querySelector('[part="close"]') as HTMLButtonElement | null

function makeDrawer(attrs: Record<string, string | boolean> = {}, inner = '') {
  return mount(reg, 'sui-drawer', attrs, inner)
}

type SuiDrawer = HTMLElement & { show(): void; close(): void }

describe('<sui-drawer>', () => {

  it('closed by default', () => {
    const { el } = makeDrawer()
    expect(el.hasAttribute('open')).toBe(false)
    expect(el.getAttribute('data-state')).toBe('closed')
  })

  it('open attribute reflects', () => {
    const { el } = makeDrawer({ open: true })
    expect(el.hasAttribute('open')).toBe(true)
  })

  it('show() opens the drawer', async () => {
    const { el } = makeDrawer()
    ;(el as SuiDrawer).show()
    await flush()
    expect(el.hasAttribute('open')).toBe(true)
    expect(el.getAttribute('data-state')).toBe('open')
  })

  it('close() closes the drawer', async () => {
    const { el } = makeDrawer({ open: true })
    ;(el as SuiDrawer).close()
    await flush()
    expect(el.hasAttribute('open')).toBe(false)
    expect(el.getAttribute('data-state')).toBe('closed')
  })

  it('renders a <dialog> in shadow DOM', () => {
    const { el } = makeDrawer()
    expect(dialogEl(el)).not.toBeNull()
  })

  it('close button fires sui-dismiss + closes', async () => {
    const { el } = makeDrawer({ open: true })
    let dismissed = 0
    el.addEventListener('sui-dismiss', () => dismissed++)
    closeBtn(el)!.click()
    await flush()
    expect(dismissed).toBe(1)
    expect(el.hasAttribute('open')).toBe(false)
  })

  it('close button fires sui-close event', async () => {
    const { el } = makeDrawer({ open: true })
    let closed = 0
    el.addEventListener('sui-close', () => closed++)
    closeBtn(el)!.click()
    await flush()
    expect(closed).toBe(1)
  })

  it('close button aria-label is set', () => {
    const { el } = makeDrawer()
    const btn = closeBtn(el)
    expect(btn!.getAttribute('aria-label')).toBe('Close drawer')
  })

  it('side — no attribute when using default (right)', () => {
    // SUI only reflects when explicitly set; defaults stay as JS property only.
    const { el } = makeDrawer()
    expect(el.hasAttribute('side')).toBe(false)
  })

  it('side reflects — left', () => {
    const { el } = makeDrawer({ side: 'left' })
    expect(el.getAttribute('side')).toBe('left')
  })

  it('side reflects when explicitly set to right', () => {
    const { el } = makeDrawer({ side: 'right' })
    expect(el.getAttribute('side')).toBe('right')
  })

  it('size — no attribute when using default (md)', () => {
    const { el } = makeDrawer()
    expect(el.hasAttribute('size')).toBe(false)
  })

  it('size reflects — lg', () => {
    const { el } = makeDrawer({ size: 'lg' })
    expect(el.getAttribute('size')).toBe('lg')
  })

  it('has title, description, footer, and default slots', () => {
    const { el } = makeDrawer()
    const slots = Array.from(shadow(el).querySelectorAll('slot'))
    const names = slots.map(s => s.getAttribute('name'))
    expect(names).toContain('title')
    expect(names).toContain('description')
    expect(names).toContain('footer')
    expect(names.some(n => n === null)).toBe(true)
  })

  it('backdrop click closes when dismissible (default true)', async () => {
    const { el } = makeDrawer({ open: true })
    const dlg = dialogEl(el)!
    const evt = new MouseEvent('click', { bubbles: true, composed: true })
    Object.defineProperty(evt, 'target', { value: dlg, writable: false })
    dlg.dispatchEvent(evt)
    await flush()
    expect(el.hasAttribute('open')).toBe(false)
  })

  it('dialog has aria-modal=true', async () => {
    const { el } = makeDrawer()
    await flush()
    expect(dialogEl(el)!.getAttribute('aria-modal')).toBe('true')
  })

  it('header aria id linked on drawer dialog', async () => {
    const { el } = makeDrawer()
    await flush()
    const labelledBy = dialogEl(el)!.getAttribute('aria-labelledby')
    expect(labelledBy).toMatch(/sui-drawer-header-\d+/)
  })

  it('disconnect while open — no error', () => {
    const { container } = makeDrawer({ open: true })
    expect(container.isConnected).toBe(true)
    container.remove()
    expect(container.isConnected).toBe(false)
  })
})
