// Keyboard/focus depth suite for <sui-drawer> — same overlay contract as
// <sui-dialog>: focus trap via sentinel redirects, focus return to the
// invoking element, Escape (`cancel`) dismiss policy, backdrop dismiss
// policy, and event contracts (name + detail + bubbles/composed).
//
// jsdom limits, stated honestly: no real Tab-key sequential navigation, no
// Escape→cancel synthesis, no top layer. This suite asserts the trap
// mechanism (sentinel focus redirect), activeElement bookkeeping, and the
// cancel policy hook; full Tab cycling and key-driven Escape are browser-tier.

import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-drawer/sui-drawer.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => (el.shadowRoot ?? el)
const nativeDialog = (el: HTMLElement) => shadow(el).querySelector('dialog') as HTMLDialogElement
// Anchor on the public part contract, not the styling class (.close).
const closeBtn = (el: HTMLElement) => shadow(el).querySelector('[part="close"]') as HTMLButtonElement
const sentinelsOf = (el: HTMLElement) =>
  Array.from(shadow(el).querySelectorAll('.sentinel')) as HTMLElement[]

type SuiDrawer = HTMLElement & { show(): void; close(): void; dismissible: boolean }

function makeDrawer(attrs: Record<string, string | boolean> = {}, inner = '') {
  return mount(reg, 'sui-drawer', attrs, inner)
}

const twoButtons = '<button id="d-first">First</button><button id="d-second">Second</button>'

describe('<sui-drawer> — dialog semantics', () => {
  it('panel is role=dialog with aria-modal=true', async () => {
    const { el } = makeDrawer()
    await flush()
    const dlg = nativeDialog(el)
    expect(dlg.getAttribute('role')).toBe('dialog')
    expect(dlg.getAttribute('aria-modal')).toBe('true')
  })
})

describe('<sui-drawer> — focus trap (sentinel mechanism)', () => {
  it('focus reaching the end sentinel wraps to the first focusable', async () => {
    const { el } = makeDrawer({ open: true }, twoButtons)
    await flush()
    const sentinels = sentinelsOf(el)
    sentinels[sentinels.length - 1].focus()
    expect(document.activeElement).toBe(el.querySelector('#d-first'))
  })

  it('focus reaching the start sentinel wraps to the last focusable (shadow close button)', async () => {
    const { el } = makeDrawer({ open: true }, twoButtons)
    await flush()
    sentinelsOf(el)[0].focus()
    expect(shadow(el).activeElement).toBe(closeBtn(el))
  })

  it('opening moves focus to the first non-close focusable', async () => {
    const { el } = makeDrawer({}, twoButtons)
    ;(el as SuiDrawer).show()
    await flush()
    expect(document.activeElement).toBe(el.querySelector('#d-first'))
  })

  it('opening honors [autofocus] over document order', async () => {
    const { el } = makeDrawer({}, '<button>plain</button><button autofocus id="daf">af</button>')
    ;(el as SuiDrawer).show()
    await flush()
    expect(document.activeElement).toBe(el.querySelector('#daf'))
  })
})

describe('<sui-drawer> — focus return', () => {
  it('closing returns focus to the element focused before open', async () => {
    const container = reg.mk()
    container.innerHTML = `<button id="d-trigger">open</button><sui-drawer>${twoButtons}</sui-drawer>`
    const trigger = container.querySelector('#d-trigger') as HTMLButtonElement
    const el = container.querySelector('sui-drawer') as SuiDrawer
    await flush()
    trigger.focus()
    el.show()
    await flush()
    expect(document.activeElement).toBe(el.querySelector('#d-first'))
    el.close()
    await flush()
    expect(document.activeElement).toBe(trigger)
  })
})

describe('<sui-drawer> — Escape (cancel) dismiss policy', () => {
  it('cancel proceeds when dismissible (default) — native close then syncs open=false', async () => {
    const { el } = makeDrawer({ open: true }, twoButtons)
    await flush()
    const dlg = nativeDialog(el)
    const cancel = new Event('cancel', { cancelable: true })
    dlg.dispatchEvent(cancel)
    expect(cancel.defaultPrevented).toBe(false)
    dlg.close()
    await flush()
    expect(el.hasAttribute('open')).toBe(false)
    expect(el.getAttribute('data-state')).toBe('closed')
  })

  it('cancel is prevented when dismissible=false — Escape must not close', async () => {
    const { el } = makeDrawer({ open: true }, twoButtons)
    await flush()
    ;(el as SuiDrawer).dismissible = false
    await flush()
    const cancel = new Event('cancel', { cancelable: true })
    nativeDialog(el).dispatchEvent(cancel)
    expect(cancel.defaultPrevented).toBe(true)
    expect(el.hasAttribute('open')).toBe(true)
  })
})

describe('<sui-drawer> — backdrop dismiss policy', () => {
  const backdropClick = (dlg: HTMLDialogElement) => {
    const evt = new MouseEvent('click', { bubbles: true, composed: true })
    Object.defineProperty(evt, 'target', { value: dlg, writable: false })
    dlg.dispatchEvent(evt)
  }

  it('backdrop click is ignored when dismissible=false', async () => {
    const { el } = makeDrawer({ open: true }, twoButtons)
    await flush()
    ;(el as SuiDrawer).dismissible = false
    await flush()
    backdropClick(nativeDialog(el))
    await flush()
    expect(el.hasAttribute('open')).toBe(true)
  })

  it('click on drawer content does not dismiss', async () => {
    const { el } = makeDrawer({ open: true }, twoButtons)
    await flush()
    ;(el.querySelector('#d-first') as HTMLButtonElement).click()
    await flush()
    expect(el.hasAttribute('open')).toBe(true)
  })
})

describe('<sui-drawer> — event contracts', () => {
  it('sui-open: fires on open; bubbles + composed; no payload (detail null)', async () => {
    const { el } = makeDrawer({}, twoButtons)
    let evt: CustomEvent | null = null
    el.addEventListener('sui-open', (e) => { evt = e as CustomEvent })
    ;(el as SuiDrawer).show()
    await flush()
    expect(evt).not.toBeNull()
    expect(evt!.bubbles).toBe(true)
    expect(evt!.composed).toBe(true)
    expect(evt!.detail).toBeNull()
  })

  it('sui-close: fires on close; bubbles + composed; no payload (detail null)', async () => {
    const { el } = makeDrawer({ open: true }, twoButtons)
    await flush()
    let evt: CustomEvent | null = null
    el.addEventListener('sui-close', (e) => { evt = e as CustomEvent })
    ;(el as SuiDrawer).close()
    await flush()
    expect(evt).not.toBeNull()
    expect(evt!.bubbles).toBe(true)
    expect(evt!.composed).toBe(true)
    expect(evt!.detail).toBeNull()
  })

  it('sui-dismiss: fires on close-button activation; bubbles + composed; no payload', async () => {
    const { el } = makeDrawer({ open: true }, twoButtons)
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
