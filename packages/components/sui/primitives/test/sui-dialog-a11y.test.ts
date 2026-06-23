// Keyboard/focus depth suite for <sui-dialog> — focus trap (sentinel
// redirects), focus return to the invoking element, Escape dismiss policy
// (the native `cancel` event), backdrop dismiss policy, and the overlay
// event contracts (name + detail + bubbles/composed).
//
// jsdom limits, stated honestly: jsdom cannot drive a real Tab key through
// sequential focus navigation and never fires `cancel` from an Escape
// keypress, and `dialog` has no top layer. What jsdom CAN express — and what
// this suite asserts — is the component's own trap mechanism (sentinel focus
// redirect), activeElement bookkeeping for initial focus + focus return, and
// the `cancel` policy hook. The full Tab cycle and key-driven Escape are
// browser-tier concerns.

import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-portal/sui-portal.compiled.js'
import '../src/sui-dialog/sui-dialog.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!
const nativeDialog = (el: HTMLElement) => shadow(el).querySelector('dialog') as HTMLDialogElement
const closeBtn = (el: HTMLElement) => shadow(el).querySelector('.close') as HTMLButtonElement
const sentinelsOf = (el: HTMLElement) =>
  Array.from(shadow(el).querySelectorAll('.sentinel')) as HTMLElement[]

type SuiDialog = HTMLElement & { show(): void; close(): void; dismissible: boolean }

function makeDialog(attrs: Record<string, string | boolean> = {}, inner = '') {
  return mount(reg, 'sui-dialog', attrs, inner)
}

const twoButtons = '<button id="b-first">First</button><button id="b-second">Second</button>'

describe('<sui-dialog> — focus trap (sentinel mechanism)', () => {
  it('focus reaching the end sentinel wraps to the first focusable', async () => {
    const { el } = makeDialog({ open: true }, twoButtons)
    await flush()
    const [, end] = sentinelsOf(el)
    end.focus()
    expect(document.activeElement).toBe(el.querySelector('#b-first'))
  })

  it('focus reaching the start sentinel wraps to the last focusable (shadow close button)', async () => {
    const { el } = makeDialog({ open: true }, twoButtons)
    await flush()
    const [start] = sentinelsOf(el)
    start.focus()
    // Last focusable in trap order = the shadow-DOM close button.
    expect(shadow(el).activeElement).toBe(closeBtn(el))
  })

  it('opening moves focus to the first non-close focusable', async () => {
    const { el } = makeDialog({}, twoButtons)
    ;(el as SuiDialog).show()
    await flush()
    expect(document.activeElement).toBe(el.querySelector('#b-first'))
  })

  it('opening honors [autofocus] over document order', async () => {
    const { el } = makeDialog({}, '<button>plain</button><button autofocus id="af">af</button>')
    ;(el as SuiDialog).show()
    await flush()
    expect(document.activeElement).toBe(el.querySelector('#af'))
  })
})

describe('<sui-dialog> — focus return', () => {
  it('closing returns focus to the element focused before open', async () => {
    const container = reg.mk()
    container.innerHTML = `<button id="trigger">open</button><sui-dialog>${twoButtons}</sui-dialog>`
    const trigger = container.querySelector('#trigger') as HTMLButtonElement
    const el = container.querySelector('sui-dialog') as SuiDialog
    await flush()
    trigger.focus()
    el.show()
    await flush()
    expect(document.activeElement).toBe(el.querySelector('#b-first'))
    el.close()
    await flush()
    expect(document.activeElement).toBe(trigger)
  })
})

describe('<sui-dialog> — Escape (cancel) dismiss policy', () => {
  it('cancel proceeds when dismissible (default) — native close then syncs open=false', async () => {
    const { el } = makeDialog({ open: true }, twoButtons)
    await flush()
    const dlg = nativeDialog(el)
    const cancel = new Event('cancel', { cancelable: true })
    dlg.dispatchEvent(cancel)
    expect(cancel.defaultPrevented).toBe(false)
    // The browser follows an unprevented cancel with a native close.
    dlg.close()
    await flush()
    expect(el.hasAttribute('open')).toBe(false)
    expect(el.getAttribute('data-state')).toBe('closed')
  })

  it('cancel is prevented when dismissible=false — Escape must not close', async () => {
    const { el } = makeDialog({ open: true }, twoButtons)
    await flush()
    ;(el as SuiDialog).dismissible = false
    await flush()
    const cancel = new Event('cancel', { cancelable: true })
    nativeDialog(el).dispatchEvent(cancel)
    expect(cancel.defaultPrevented).toBe(true)
    expect(el.hasAttribute('open')).toBe(true)
  })
})

describe('<sui-dialog> — backdrop dismiss policy', () => {
  // jsdom has no layout, so "the backdrop" is expressed the way the
  // component sees it: a click whose target is the <dialog> itself.
  const backdropClick = (dlg: HTMLDialogElement) => {
    const evt = new MouseEvent('click', { bubbles: true, composed: true })
    Object.defineProperty(evt, 'target', { value: dlg, writable: false })
    dlg.dispatchEvent(evt)
  }

  it('backdrop click closes when dismissible (default)', async () => {
    const { el } = makeDialog({ open: true }, twoButtons)
    await flush()
    backdropClick(nativeDialog(el))
    await flush()
    expect(el.hasAttribute('open')).toBe(false)
  })

  it('backdrop click is ignored when dismissible=false', async () => {
    const { el } = makeDialog({ open: true }, twoButtons)
    await flush()
    ;(el as SuiDialog).dismissible = false
    await flush()
    backdropClick(nativeDialog(el))
    await flush()
    expect(el.hasAttribute('open')).toBe(true)
  })

  it('click on dialog content does not dismiss', async () => {
    const { el } = makeDialog({ open: true }, twoButtons)
    await flush()
    ;(el.querySelector('#b-first') as HTMLButtonElement).click()
    await flush()
    expect(el.hasAttribute('open')).toBe(true)
  })
})

describe('<sui-dialog> — event contracts', () => {
  it('sui-open: fires on open; bubbles + composed; no payload (detail null)', async () => {
    const { el } = makeDialog({}, twoButtons)
    let evt: CustomEvent | null = null
    el.addEventListener('sui-open', (e) => { evt = e as CustomEvent })
    ;(el as SuiDialog).show()
    await flush()
    expect(evt).not.toBeNull()
    expect(evt!.bubbles).toBe(true)
    expect(evt!.composed).toBe(true)
    expect(evt!.detail).toBeNull()
  })

  it('sui-close: fires on close; bubbles + composed; no payload (detail null)', async () => {
    const { el } = makeDialog({ open: true }, twoButtons)
    await flush()
    let evt: CustomEvent | null = null
    el.addEventListener('sui-close', (e) => { evt = e as CustomEvent })
    ;(el as SuiDialog).close()
    await flush()
    expect(evt).not.toBeNull()
    expect(evt!.bubbles).toBe(true)
    expect(evt!.composed).toBe(true)
    expect(evt!.detail).toBeNull()
  })

  it('sui-dismiss: fires on close-button activation; bubbles + composed; no payload', async () => {
    const { el } = makeDialog({ open: true }, twoButtons)
    await flush()
    let evt: CustomEvent | null = null
    el.addEventListener('sui-dismiss', (e) => { evt = e as CustomEvent })
    closeBtn(el).click()
    await flush()
    expect(evt).not.toBeNull()
    expect(evt!.bubbles).toBe(true)
    expect(evt!.composed).toBe(true)
    expect(evt!.detail).toBeNull()
  })
})
