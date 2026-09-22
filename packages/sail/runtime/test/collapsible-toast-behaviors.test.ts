// Collapsible + toast capability behaviors — autoDismiss / dismissControl.
//
// Accordion previously used detailsToggle / detailsDisabled; now native
// <details> + controller on:toggle/on:click/on:keydown owns that.
// Toast actions remain stacked use: mix-ins.
import { describe, expect, it, afterEach, vi, beforeEach } from 'vitest'
import { autoDismiss } from '../src/actions/auto-dismiss'
import { dismissControl } from '../src/actions/dismiss-control'

function makeToast(attrs: Record<string, string | boolean> = {}): HTMLElement {
  const el = document.createElement('div')
  el.setAttribute('data-sail-host', 'toast')
  el.setAttribute('role', 'status')
  el.setAttribute('aria-live', 'polite')
  for (const [k, v] of Object.entries(attrs)) {
    if (v === false) continue
    if (v === true) el.setAttribute(k, '')
    else el.setAttribute(k, v)
  }
  const span = document.createElement('span')
  span.textContent = 'Hello'
  el.append(span)
  const close = document.createElement('button')
  close.setAttribute('data-toast-close', '')
  close.setAttribute('aria-label', 'Dismiss')
  close.type = 'button'
  close.textContent = '×'
  el.append(close)
  document.body.append(el)
  return el
}

describe('autoDismiss / dismissControl (use: actions)', () => {
  beforeEach(() => {
    vi.useFakeTimers()
  })
  afterEach(() => {
    vi.useRealTimers()
    document.body.replaceChildren()
  })

  it('autoDismiss: timer dismisses with onDismiss + remove', () => {
    const el = makeToast({ duration: '1000' })
    autoDismiss(el)

    let dismissed = 0
    ;(el as any).onDismiss = () => { dismissed++ }

    vi.advanceTimersByTime(999)
    expect(document.body.contains(el)).toBe(true)
    expect(dismissed).toBe(0)

    vi.advanceTimersByTime(1)
    expect(dismissed).toBe(1)
    expect(document.body.contains(el)).toBe(false)
  })

  it('autoDismiss: default duration is 5000 when attr missing', () => {
    const el = makeToast()
    autoDismiss(el)
    vi.advanceTimersByTime(4999)
    expect(document.body.contains(el)).toBe(true)
    vi.advanceTimersByTime(1)
    expect(document.body.contains(el)).toBe(false)
  })

  it('autoDismiss: duration 0 is sticky (no timer)', () => {
    const el = makeToast({ duration: '0' })
    autoDismiss(el)
    vi.advanceTimersByTime(60_000)
    expect(document.body.contains(el)).toBe(true)
  })

  it('autoDismiss: pause on pointerenter, resume on pointerleave', () => {
    const el = makeToast({ duration: '1000' })
    autoDismiss(el)

    vi.advanceTimersByTime(400)
    el.dispatchEvent(new Event('pointerenter'))
    vi.advanceTimersByTime(2000)
    expect(document.body.contains(el)).toBe(true)

    el.dispatchEvent(new Event('pointerleave'))
    vi.advanceTimersByTime(999)
    expect(document.body.contains(el)).toBe(true)
    vi.advanceTimersByTime(1)
    expect(document.body.contains(el)).toBe(false)
  })

  it('dismissControl: data-toast-close click → onDismiss + removed', () => {
    const el = makeToast({ duration: '0' })
    dismissControl(el)

    let dismissed = 0
    ;(el as any).onDismiss = () => { dismissed++ }

    const close = el.querySelector('[data-toast-close]') as HTMLElement
    close.click()
    expect(dismissed).toBe(1)
    expect(document.body.contains(el)).toBe(false)
  })

  it('composed: autoDismiss + dismissControl share one dismiss (no double event)', () => {
    const el = makeToast({ duration: '5000' })
    autoDismiss(el)
    dismissControl(el)

    let dismissed = 0
    ;(el as any).onDismiss = () => { dismissed++ }

    const close = el.querySelector('[data-toast-close]') as HTMLElement
    close.click()
    expect(dismissed).toBe(1)
    expect(document.body.contains(el)).toBe(false)

    // timer would have fired later — already dismissed, no second event
    vi.advanceTimersByTime(10_000)
    expect(dismissed).toBe(1)
  })
})
