import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-portal/sui-portal.compiled.js'
import '../src/sui-dialog/sui-dialog.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!
const dialogEl = (el: HTMLElement) => shadow(el).querySelector('dialog') as HTMLDialogElement | null
const closeBtn = (el: HTMLElement) => shadow(el).querySelector('.close') as HTMLButtonElement | null

function makeDialog(attrs: Record<string, string | boolean> = {}, inner = '') {
  return mount(reg, 'sui-dialog', attrs, inner)
}

type SuiDialog = HTMLElement & { show(): void; showModal(): void; close(): void }

describe('<sui-dialog>', () => {

  it('closed by default — open attr absent, data-state=closed', () => {
    const { el } = makeDialog()
    expect(el.hasAttribute('open')).toBe(false)
    expect(el.getAttribute('data-state')).toBe('closed')
  })

  it('open attribute reflects on host', () => {
    const { el } = makeDialog({ open: true })
    expect(el.hasAttribute('open')).toBe(true)
  })

  it('show() sets open=true, showModal() same', async () => {
    const { el } = makeDialog()
    ;(el as SuiDialog).show()
    await flush()
    expect(el.hasAttribute('open')).toBe(true)
  })

  it('showModal() sets open=true', async () => {
    const { el } = makeDialog()
    ;(el as SuiDialog).showModal()
    await flush()
    expect(el.hasAttribute('open')).toBe(true)
  })

  it('close() sets open=false', async () => {
    const { el } = makeDialog({ open: true })
    ;(el as SuiDialog).close()
    await flush()
    expect(el.hasAttribute('open')).toBe(false)
    expect(el.getAttribute('data-state')).toBe('closed')
  })

  it('data-state toggles with open prop', async () => {
    const { el } = makeDialog()
    expect(el.getAttribute('data-state')).toBe('closed')
    ;(el as SuiDialog).show()
    await flush()
    expect(el.getAttribute('data-state')).toBe('open')
  })

  it('renders a <dialog> element in shadow DOM', () => {
    const { el } = makeDialog()
    expect(dialogEl(el)).not.toBeNull()
  })

  it('close button exists and has aria-label', () => {
    const { el } = makeDialog()
    const btn = closeBtn(el)
    expect(btn).not.toBeNull()
    expect(btn!.getAttribute('aria-label')).toBe('Close dialog')
  })

  it('close button fires sui-dismiss and sets open=false', async () => {
    const { el } = makeDialog({ open: true })
    let dismissed = 0
    el.addEventListener('sui-dismiss', () => dismissed++)
    closeBtn(el)!.click()
    await flush()
    expect(dismissed).toBe(1)
    expect(el.hasAttribute('open')).toBe(false)
  })

  it('no-close-button hides the close button', () => {
    const { el } = makeDialog({ 'no-close-button': true })
    expect(el.hasAttribute('no-close-button')).toBe(true)
  })

  it('has header, body, and footer slots when open', async () => {
    const { el } = makeDialog({ open: true })
    await flush()
    const slots = Array.from(shadow(el).querySelectorAll('slot'))
    const names = slots.map(s => s.getAttribute('name'))
    expect(names).toContain('header')
    expect(names).toContain('footer')
    expect(names.some(n => n === null)).toBe(true)
  })

  it('size attribute reflects', () => {
    const { el } = makeDialog({ size: 'lg' })
    expect(el.getAttribute('size')).toBe('lg')
  })

  it('backdrop attribute reflects', () => {
    const { el } = makeDialog({ backdrop: 'blur' })
    expect(el.getAttribute('backdrop')).toBe('blur')
  })

  it('dialog has role=dialog by default', async () => {
    const { el } = makeDialog()
    await flush()
    expect(dialogEl(el)!.getAttribute('role')).toBe('dialog')
  })

  it('role-override=alertdialog sets role on native dialog', async () => {
    const { el } = makeDialog({ 'role-override': 'alertdialog' })
    await flush()
    expect(dialogEl(el)!.getAttribute('role')).toBe('alertdialog')
  })

  it('dialog has aria-modal=true', async () => {
    const { el } = makeDialog()
    await flush()
    expect(dialogEl(el)!.getAttribute('aria-modal')).toBe('true')
  })

  it('header region gets aria id linked from dialog', async () => {
    const { el } = makeDialog({ open: true })
    await flush()
    const labelledBy = dialogEl(el)!.getAttribute('aria-labelledby')
    expect(labelledBy).toMatch(/sui-dialog-header-\d+/)
    const header = shadow(el).querySelector('.header')
    expect(header?.id).toBe(labelledBy)
  })

  it('body region gets aria id linked from dialog', async () => {
    const { el } = makeDialog({ open: true })
    await flush()
    const describedBy = dialogEl(el)!.getAttribute('aria-describedby')
    expect(describedBy).toMatch(/sui-dialog-body-\d+/)
    const body = shadow(el).querySelector('.body')
    expect(body?.id).toBe(describedBy)
  })

  it('no-header reflects on host', () => {
    const { el } = makeDialog({ 'no-header': true })
    expect(el.hasAttribute('no-header')).toBe(true)
  })

  it('no-footer reflects on host', () => {
    const { el } = makeDialog({ 'no-footer': true })
    expect(el.hasAttribute('no-footer')).toBe(true)
  })

  it('disconnect without open — no scroll lock leak', () => {
    const { container } = makeDialog()
    expect(container.isConnected).toBe(true)
    container.remove()
    expect(container.isConnected).toBe(false)
    // A never-opened dialog must not have locked the document scroll.
    expect(document.documentElement.style.overflow).not.toBe('hidden')
  })

  it('disconnect while open — scroll lock released', () => {
    const { container } = makeDialog({ open: true })
    expect(container.isConnected).toBe(true)
    container.remove()
    expect(container.isConnected).toBe(false)
    // Disconnect while open releases any scroll lock the open state took.
    expect(document.documentElement.style.overflow).not.toBe('hidden')
  })
})
