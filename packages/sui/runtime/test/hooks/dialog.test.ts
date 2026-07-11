// useDialog — the blessed handle for a native <dialog>. This suite covers the
// parts useDialog + bindOpen actually OWN: the reactive open API, the
// signal<->showModal/close binding, the close-event sync, the ESC (cancel)
// dismiss policy, the aria wiring (the one parity gap over the platform), and
// teardown.
//
// jsdom limits, stated honestly: jsdom has no top layer, so showModal/close
// don't do modal stacking, the focus trap, or focus-return — those are
// browser-tier and unverifiable here. We polyfill ONLY the open-state
// bookkeeping (el.open + the native `close` event) so the binding is
// exercisable; everything asserted below is real DOM/signal behavior.

import { afterEach, beforeAll, describe, expect, it } from 'vitest'
import { useDialog } from '../../src/index'

beforeAll(() => {
  const P = HTMLDialogElement.prototype as unknown as {
    showModal: () => void
    close: () => void
  }
  // Deterministic open-state bookkeeping across jsdom versions (jsdom's native
  // dialog support is partial and top-layer parts are unimplemented).
  P.showModal = function (this: HTMLDialogElement) {
    this.open = true
  }
  P.close = function (this: HTMLDialogElement) {
    const was = this.open
    this.open = false
    if (was) this.dispatchEvent(new Event('close'))
  }
})

type Handle = ReturnType<typeof useDialog>

function makeDialog(): HTMLDialogElement {
  const dlg = document.createElement('dialog')
  dlg.innerHTML =
    '<header>Delete project?</header><section>This cannot be undone.</section>'
  document.body.appendChild(dlg)
  return dlg
}

// The handle IS the `use:` action — attaching it is calling it with the node.
function attach(h: Handle, dlg: HTMLDialogElement): () => void {
  const r = h(dlg) as { destroy: () => void }
  return r.destroy
}

afterEach(() => {
  document.querySelectorAll('dialog').forEach((d) => d.remove())
})

describe('useDialog', () => {
  it('exposes a reactive open API (isOpen/open/close/toggle)', () => {
    const h = useDialog()
    expect(h.isOpen()).toBe(false)
    h.open()
    expect(h.isOpen()).toBe(true)
    h.close()
    expect(h.isOpen()).toBe(false)
    h.toggle()
    expect(h.isOpen()).toBe(true)
    h.toggle()
    expect(h.isOpen()).toBe(false)
  })

  it('attaching (use:) drives the native dialog from the signal', () => {
    const h = useDialog()
    const dlg = makeDialog()
    const destroy = attach(h, dlg)
    expect(dlg.open).toBe(false)
    h.open()
    expect(dlg.open).toBe(true)
    h.close()
    expect(dlg.open).toBe(false)
    destroy()
  })

  it('a native close (ESC/backdrop/button/.close()) syncs open=false back', () => {
    const h = useDialog()
    const dlg = makeDialog()
    const destroy = attach(h, dlg)
    h.open()
    expect(h.isOpen()).toBe(true)
    dlg.close() // the platform closed it
    expect(h.isOpen()).toBe(false) // signal followed
    destroy()
  })

  it('wires aria-labelledby from <header> and aria-describedby from <section>', () => {
    const h = useDialog()
    const dlg = makeDialog()
    const destroy = attach(h, dlg)
    const header = dlg.querySelector('header') as HTMLElement
    const section = dlg.querySelector('section') as HTMLElement
    expect(header.id).toBeTruthy()
    expect(section.id).toBeTruthy()
    expect(dlg.getAttribute('aria-labelledby')).toBe(header.id)
    expect(dlg.getAttribute('aria-describedby')).toBe(section.id)
    destroy()
  })

  it('never overrides author-set aria-labelledby/describedby', () => {
    const h = useDialog()
    const dlg = makeDialog()
    dlg.setAttribute('aria-labelledby', 'custom-title')
    dlg.setAttribute('aria-describedby', 'custom-desc')
    const destroy = attach(h, dlg)
    expect(dlg.getAttribute('aria-labelledby')).toBe('custom-title')
    expect(dlg.getAttribute('aria-describedby')).toBe('custom-desc')
    destroy()
  })

  it('dismissible=false prevents the ESC cancel (dialog must not dismiss)', () => {
    const h = useDialog({ dismissible: false })
    const dlg = makeDialog()
    const destroy = attach(h, dlg)
    h.open()
    const cancel = new Event('cancel', { cancelable: true })
    dlg.dispatchEvent(cancel)
    expect(cancel.defaultPrevented).toBe(true)
    destroy()
  })

  it('dismissible (default) lets the ESC cancel proceed', () => {
    const h = useDialog()
    const dlg = makeDialog()
    const destroy = attach(h, dlg)
    h.open()
    const cancel = new Event('cancel', { cancelable: true })
    dlg.dispatchEvent(cancel)
    expect(cancel.defaultPrevented).toBe(false)
    destroy()
  })

  it('destroy() tears down the binding — a later native close no longer syncs', () => {
    const h = useDialog()
    const dlg = makeDialog()
    const destroy = attach(h, dlg)
    h.open()
    expect(h.isOpen()).toBe(true)
    destroy()
    // The close listener is gone; a stray native close must not flip the signal.
    dlg.dispatchEvent(new Event('close'))
    expect(h.isOpen()).toBe(true)
  })
})
