import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-dialog/sui-dialog.compiled.js'
import '../src/sui-button/sui-button.compiled.js'
import '../src/sui-alert-dialog/sui-alert-dialog.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!
const innerDialog = (el: HTMLElement) =>
  shadow(el).querySelector('sui-dialog') as HTMLElement | null
const innerButtons = (el: HTMLElement) =>
  Array.from(shadow(el).querySelectorAll('sui-button')) as HTMLElement[]
const cancelBtn = (el: HTMLElement) =>
  innerButtons(el).find((b) => b.getAttribute('variant') === 'outline') ?? null
const confirmBtn = (el: HTMLElement) =>
  innerButtons(el).find((b) => b.getAttribute('variant') === 'solid') ?? null

function makeAlert(attrs: Record<string, string | boolean> = {}, inner = '') {
  return mount(reg, 'sui-alert-dialog', attrs, inner)
}

describe('<sui-alert-dialog>', () => {

  it('renders an inner sui-dialog with alertdialog role', () => {
    const { el } = makeAlert()
    const dlg = innerDialog(el)
    expect(dlg).not.toBeNull()
    expect(dlg!.getAttribute('role-override')).toBe('alertdialog')
    expect(dlg!.getAttribute('size')).toBe('sm')
    expect(dlg!.hasAttribute('no-close-button')).toBe(true)
  })

  it('open attribute reflects on host', () => {
    const { el } = makeAlert({ open: true })
    expect(el.hasAttribute('open')).toBe(true)
  })

  it('tone attribute reflects on host', () => {
    const { el } = makeAlert({ tone: 'destructive' })
    expect(el.getAttribute('tone')).toBe('destructive')
  })

  it('renders default confirm + cancel labels', async () => {
    const { el } = makeAlert()
    await flush()
    expect(cancelBtn(el)!.textContent?.trim()).toBe('Cancel')
    expect(confirmBtn(el)!.textContent?.trim()).toBe('Confirm')
  })

  it('honors custom confirm-label and cancel-label', async () => {
    const { el } = makeAlert({
      'confirm-label': 'Delete it',
      'cancel-label': 'Keep',
    })
    await flush()
    expect(cancelBtn(el)!.textContent?.trim()).toBe('Keep')
    expect(confirmBtn(el)!.textContent?.trim()).toBe('Delete it')
  })

  it('tone=default applies default tone to confirm button', async () => {
    const { el } = makeAlert()
    await flush()
    expect(confirmBtn(el)!.getAttribute('tone')).toBe('default')
  })

  it('tone=destructive propagates to the confirm button', async () => {
    const { el } = makeAlert({ tone: 'destructive' })
    await flush()
    expect(confirmBtn(el)!.getAttribute('tone')).toBe('destructive')
  })

  it('confirm button fires sui-confirm (bubbles + composed, detail null) and closes', async () => {
    const { el } = makeAlert({ open: true })
    await flush()
    let evt: CustomEvent | null = null
    el.addEventListener('sui-confirm', (e) => { evt = e as CustomEvent })
    confirmBtn(el)!.click()
    await flush()
    expect(evt).not.toBeNull()
    expect(evt!.bubbles).toBe(true)
    expect(evt!.composed).toBe(true)
    expect(evt!.detail).toBeNull()
    expect(el.hasAttribute('open')).toBe(false)
  })

  it('cancel button fires sui-cancel (bubbles + composed, detail null) and closes', async () => {
    const { el } = makeAlert({ open: true })
    await flush()
    let evt: CustomEvent | null = null
    el.addEventListener('sui-cancel', (e) => { evt = e as CustomEvent })
    cancelBtn(el)!.click()
    await flush()
    expect(evt).not.toBeNull()
    expect(evt!.bubbles).toBe(true)
    expect(evt!.composed).toBe(true)
    expect(evt!.detail).toBeNull()
    expect(el.hasAttribute('open')).toBe(false)
  })

  it('renders body slot content and title slot content', () => {
    const { el } = makeAlert(
      { open: true },
      '<span slot="title">Heads up</span>Are you sure?',
    )
    // Slots project; light-DOM children remain attached to the host.
    const title = el.querySelector('[slot="title"]')
    expect(title?.textContent).toBe('Heads up')
    expect(el.textContent).toContain('Are you sure?')
  })
})
