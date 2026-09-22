// use:drawerSwipe — swipe / Escape / light-dismiss for non-modal <drawer>.

import { afterEach, describe, expect, it, vi } from 'vitest'
import { drawerSwipe } from '../src/actions/drawer-swipe'
import { initJsdomPolyfills } from '../../test-setup/src/jsdom-polyfills'

initJsdomPolyfills()

function makeDialog(side = 'end'): HTMLDialogElement {
  const el = document.createElement('dialog')
  el.setAttribute('side', side)
  el.setAttribute('dismissible', '')
  if (typeof el.show !== 'function') {
    el.show = function show(this: HTMLDialogElement) {
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
  el.show()
  return el
}

function pointer(
  el: EventTarget,
  type: 'pointerdown' | 'pointermove' | 'pointerup',
  x: number,
  y: number,
): void {
  el.dispatchEvent(
    new PointerEvent(type, {
      bubbles: true,
      cancelable: true,
      clientX: x,
      clientY: y,
      pointerId: 1,
      button: 0,
    }),
  )
}

describe('use:drawerSwipe', () => {
  afterEach(() => {
    document.body.replaceChildren()
    vi.useRealTimers()
  })

  it('swipe toward the end edge closes', () => {
    const el = makeDialog('end')
    const close = vi.spyOn(el, 'close')
    const handle = drawerSwipe(el) as { destroy: () => void }

    pointer(el, 'pointerdown', 100, 80)
    pointer(el, 'pointermove', 220, 80)
    pointer(el, 'pointerup', 220, 80)

    expect(close).toHaveBeenCalled()
    handle.destroy()
  })

  it('swipe the wrong way does not close', () => {
    const el = makeDialog('end')
    const close = vi.spyOn(el, 'close')
    const handle = drawerSwipe(el) as { destroy: () => void }

    pointer(el, 'pointerdown', 200, 80)
    pointer(el, 'pointermove', 80, 80)
    pointer(el, 'pointerup', 80, 80)

    expect(close).not.toHaveBeenCalled()
    handle.destroy()
  })

  it('Escape closes when dismissible', () => {
    const el = makeDialog('start')
    const close = vi.spyOn(el, 'close')
    const handle = drawerSwipe(el) as { destroy: () => void }

    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))
    expect(close).toHaveBeenCalled()
    handle.destroy()
  })

  it('dismissible=false blocks swipe and Escape', () => {
    const el = makeDialog('end')
    el.setAttribute('dismissible', 'false')
    const close = vi.spyOn(el, 'close')
    const handle = drawerSwipe(el) as { destroy: () => void }

    pointer(el, 'pointerdown', 100, 80)
    pointer(el, 'pointermove', 220, 80)
    pointer(el, 'pointerup', 220, 80)
    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))

    expect(close).not.toHaveBeenCalled()
    handle.destroy()
  })

  it('close event invokes onClose and onDismiss', () => {
    const el = makeDialog('end')
    const onClose = vi.fn()
    const onDismiss = vi.fn()
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    ;(el as any).onClose = onClose
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    ;(el as any).onDismiss = onDismiss
    const handle = drawerSwipe(el) as { destroy: () => void }

    el.close()
    expect(onClose).toHaveBeenCalledTimes(1)
    expect(onDismiss).toHaveBeenCalledTimes(1)
    handle.destroy()
  })

  it('command=show on a matching invoker opens a closed drawer', () => {
    const el = makeDialog('end')
    el.close()
    el.id = 'order-filters'
    const btn = document.createElement('button')
    btn.setAttribute('command', 'show')
    btn.setAttribute('commandfor', 'order-filters')
    btn.textContent = 'Filters'
    document.body.appendChild(btn)
    const handle = drawerSwipe(el) as { destroy: () => void }
    const show = vi.spyOn(el, 'show')

    btn.click()

    expect(show).toHaveBeenCalled()
    handle.destroy()
  })
})
