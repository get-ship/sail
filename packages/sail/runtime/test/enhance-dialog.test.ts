// bindOpen — modal open signal + dismissible opts (ESC / backdrop).
// jsdom's HTMLDialogElement is partial; we exercise the cancel/close/backdrop
// listeners that bindOpen owns, not platform showModal top-layer.

import { describe, expect, it, vi } from 'vitest'
import { signal } from '../src/signal'
import { bindOpen } from '../src/enhance-dialog'

function makeDialog(): HTMLDialogElement {
  const el = document.createElement('dialog')
  // jsdom may not implement showModal/close — stub for signal → modal path.
  if (typeof el.showModal !== 'function') {
    el.showModal = function showModal(this: HTMLDialogElement) {
      this.setAttribute('open', '')
      Object.defineProperty(this, 'open', { configurable: true, get: () => true })
    }
  }
  if (typeof el.close !== 'function') {
    el.close = function close(this: HTMLDialogElement) {
      this.removeAttribute('open')
      Object.defineProperty(this, 'open', { configurable: true, get: () => false })
      this.dispatchEvent(new Event('close'))
    }
  }
  document.body.appendChild(el)
  return el
}

describe('bindOpen', () => {
  it('signal true → showModal; signal false → close; native close writes back', () => {
    const el = makeDialog()
    const open = signal(false)
    const show = vi.spyOn(el, 'showModal')
    bindOpen(el, open)

    open.set(true)
    expect(show).toHaveBeenCalled()

    // Native close (ESC / form method=dialog / .close()) → signal false.
    open.set(true)
    el.dispatchEvent(new Event('close'))
    expect(open()).toBe(false)
  })

  it('dismissible default: cancel is not prevented (ESC can close)', () => {
    const el = makeDialog()
    const open = signal(true)
    bindOpen(el, open)

    const cancel = new Event('cancel', { cancelable: true })
    el.dispatchEvent(cancel)
    expect(cancel.defaultPrevented).toBe(false)
  })

  it('dismissible:false — cancel is prevented (ESC must not close)', () => {
    const el = makeDialog()
    const open = signal(true)
    bindOpen(el, open, { dismissible: false })

    const cancel = new Event('cancel', { cancelable: true })
    el.dispatchEvent(cancel)
    expect(cancel.defaultPrevented).toBe(true)
  })

  it('dismissible:false — backdrop click does not close', () => {
    const el = makeDialog()
    // Force open state so close() is meaningful if it fires.
    Object.defineProperty(el, 'open', { configurable: true, get: () => true })
    const close = vi.spyOn(el, 'close')
    const open = signal(true)
    bindOpen(el, open, { dismissible: false })

    // Backdrop: click target is the dialog itself, outside the panel rect.
    vi.spyOn(el, 'getBoundingClientRect').mockReturnValue({
      left: 100,
      right: 200,
      top: 100,
      bottom: 200,
      width: 100,
      height: 100,
      x: 100,
      y: 100,
      toJSON: () => ({}),
    })
    el.dispatchEvent(
      new MouseEvent('click', { bubbles: true, clientX: 10, clientY: 10 }),
    )
    expect(close).not.toHaveBeenCalled()
  })

  it('dismissible default — backdrop click closes', () => {
    const el = makeDialog()
    Object.defineProperty(el, 'open', { configurable: true, get: () => true })
    const close = vi.spyOn(el, 'close')
    const open = signal(true)
    bindOpen(el, open)

    vi.spyOn(el, 'getBoundingClientRect').mockReturnValue({
      left: 100,
      right: 200,
      top: 100,
      bottom: 200,
      width: 100,
      height: 100,
      x: 100,
      y: 100,
      toJSON: () => ({}),
    })
    el.dispatchEvent(
      new MouseEvent('click', { bubbles: true, clientX: 10, clientY: 10 }),
    )
    expect(close).toHaveBeenCalled()
  })

  it('disposer removes listeners (cancel no longer prevented)', () => {
    const el = makeDialog()
    const open = signal(true)
    const dispose = bindOpen(el, open, { dismissible: false })
    dispose()

    const cancel = new Event('cancel', { cancelable: true })
    el.dispatchEvent(cancel)
    expect(cancel.defaultPrevented).toBe(false)
  })
})
