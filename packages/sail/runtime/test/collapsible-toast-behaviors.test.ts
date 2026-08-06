// Collapsible + toast capability behaviors — detailsToggle / detailsDisabled /
// autoDismiss / dismissControl.
//
// Ports the element-private collapsible/toast action contracts onto the
// stacked `use:` mix-ins the L1 stdlib stamps instead. Each action is attached
// directly (runAction lifecycle is covered elsewhere).
import { describe, expect, it, afterEach, vi, beforeEach } from 'vitest'
import { detailsToggle } from '../src/actions/details-toggle'
import { detailsDisabled } from '../src/actions/details-disabled'
import { autoDismiss } from '../src/actions/auto-dismiss'
import { dismissControl } from '../src/actions/dismiss-control'

function makeDetails(attrs: Record<string, string | boolean> = {}): HTMLDetailsElement {
  const el = document.createElement('details')
  el.className = 'sail-accordion'
  for (const [k, v] of Object.entries(attrs)) {
    if (v === false) continue
    if (v === true) el.setAttribute(k, '')
    else el.setAttribute(k, v)
  }
  const summary = document.createElement('summary')
  summary.textContent = 'Trigger'
  el.append(summary)
  const body = document.createElement('div')
  body.textContent = 'Content'
  el.append(body)
  document.body.append(el)
  return el
}

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

describe('detailsToggle / detailsDisabled (use: actions)', () => {
  afterEach(() => {
    document.body.replaceChildren()
  })

  it('detailsToggle: open flip → onToggle({ open })', () => {
    const el = makeDetails() as HTMLDetailsElement & {
      onToggle?: (d: { open: boolean }) => void
    }
    detailsToggle(el)

    const events: Array<{ open: boolean }> = []
    el.onToggle = (d) => {
      events.push(d)
    }

    el.open = true
    el.dispatchEvent(new Event('toggle'))
    expect(events).toEqual([{ open: true }])

    el.open = false
    el.dispatchEvent(new Event('toggle'))
    expect(events).toEqual([{ open: true }, { open: false }])
  })

  it('detailsDisabled: disabled blocks click activation path', () => {
    const el = makeDetails({ disabled: true })
    detailsDisabled(el)
    const summary = el.querySelector('summary')!

    const click = new MouseEvent('click', { bubbles: true, cancelable: true })
    summary.dispatchEvent(click)
    // capture listener on the details host preventDefaults while [disabled]
    expect(click.defaultPrevented).toBe(true)

    el.removeAttribute('disabled')
    const click2 = new MouseEvent('click', { bubbles: true, cancelable: true })
    summary.dispatchEvent(click2)
    expect(click2.defaultPrevented).toBe(false)
  })

  it('detailsDisabled: Enter/Space preventDefault while disabled', () => {
    const el = makeDetails({ disabled: true })
    detailsDisabled(el)

    const enter = new KeyboardEvent('keydown', { key: 'Enter', bubbles: true, cancelable: true })
    el.dispatchEvent(enter)
    expect(enter.defaultPrevented).toBe(true)

    const space = new KeyboardEvent('keydown', { key: ' ', bubbles: true, cancelable: true })
    el.dispatchEvent(space)
    expect(space.defaultPrevented).toBe(true)

    el.removeAttribute('disabled')
    const enter2 = new KeyboardEvent('keydown', { key: 'Enter', bubbles: true, cancelable: true })
    el.dispatchEvent(enter2)
    expect(enter2.defaultPrevented).toBe(false)
  })
})

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
