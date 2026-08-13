// Alert capability behaviors — liveRegion / dismissControl.
//
// Ports the retired use:alert contracts onto the stacked `use:` mix-ins the
// L1 <alert> stamps instead. Each action is attached directly (runAction
// lifecycle is covered elsewhere). MutationObserver callbacks are async in
// jsdom — attribute-flip assertions flush microtasks.
import { describe, expect, it, afterEach } from 'vitest'
import { liveRegion } from '../src/actions/live-region'
import { dismissControl } from '../src/actions/dismiss-control'

function makeAlert(attrs: Record<string, string | boolean> = {}): HTMLElement {
  const el = document.createElement('div')
  el.className = 'sail-alert'
  el.setAttribute('role', 'status')
  el.setAttribute('aria-live', 'polite')
  el.setAttribute('aria-atomic', 'true')
  for (const [k, v] of Object.entries(attrs)) {
    if (v === false) continue
    if (v === true) el.setAttribute(k, '')
    else el.setAttribute(k, v)
  }
  const content = document.createElement('div')
  content.textContent = 'Message'
  el.append(content)
  const close = document.createElement('button')
  close.type = 'button'
  close.setAttribute('data-dismiss', '')
  close.setAttribute('aria-label', 'Dismiss')
  close.textContent = '×'
  el.append(close)
  document.body.append(el)
  return el
}

describe('liveRegion / dismissControl (alert use: stack)', () => {
  afterEach(() => {
    document.body.replaceChildren()
  })

  it('liveRegion: tone warning → role=alert aria-live=assertive', () => {
    const el = makeAlert({ tone: 'warning' })
    liveRegion(el)
    expect(el.getAttribute('role')).toBe('alert')
    expect(el.getAttribute('aria-live')).toBe('assertive')
  })

  it('liveRegion: tone destructive → assertive alert', () => {
    const el = makeAlert({ tone: 'destructive' })
    liveRegion(el)
    expect(el.getAttribute('role')).toBe('alert')
    expect(el.getAttribute('aria-live')).toBe('assertive')
  })

  it('liveRegion: info/success stay status/polite', () => {
    const el = makeAlert({ tone: 'info' })
    liveRegion(el)
    expect(el.getAttribute('role')).toBe('status')
    expect(el.getAttribute('aria-live')).toBe('polite')
  })

  it('liveRegion: tone flip warning → info restores status/polite', async () => {
    const el = makeAlert({ tone: 'warning' })
    liveRegion(el)
    expect(el.getAttribute('role')).toBe('alert')
    expect(el.getAttribute('aria-live')).toBe('assertive')

    el.setAttribute('tone', 'info')
    await Promise.resolve()
    expect(el.getAttribute('role')).toBe('status')
    expect(el.getAttribute('aria-live')).toBe('polite')
  })

  it('liveRegion: first paint syncs without waiting for observer', () => {
    // Static host props are status/polite; action must overwrite on attach.
    const el = makeAlert({ tone: 'destructive' })
    expect(el.getAttribute('role')).toBe('status')
    liveRegion(el)
    expect(el.getAttribute('role')).toBe('alert')
    expect(el.getAttribute('aria-live')).toBe('assertive')
  })

  it('dismissControl: dismissible + dismiss control → onDismiss + removed', () => {
    const el = makeAlert({ dismissible: true, tone: 'info' })
    dismissControl(el)

    let dismissed = 0
    ;(el as any).onDismiss = () => { dismissed++ }

    const close = el.querySelector('[data-dismiss]') as HTMLElement
    close.click()
    expect(dismissed).toBe(1)
    expect(document.body.contains(el)).toBe(false)
  })

  it('dismissControl: not dismissible → close is a no-op', () => {
    const el = makeAlert({ tone: 'info' })
    dismissControl(el)

    let dismissed = 0
    ;(el as any).onDismiss = () => { dismissed++ }

    const close = el.querySelector('[data-dismiss]') as HTMLElement
    close.click()
    expect(dismissed).toBe(0)
    expect(document.body.contains(el)).toBe(true)
  })

  it('composed: liveRegion + dismissControl on dismissible alert', () => {
    const el = makeAlert({ tone: 'warning', dismissible: true })
    liveRegion(el)
    dismissControl(el)

    expect(el.getAttribute('role')).toBe('alert')
    expect(el.getAttribute('aria-live')).toBe('assertive')

    let dismissed = 0
    ;(el as any).onDismiss = () => { dismissed++ }
    ;(el.querySelector('[data-dismiss]') as HTMLElement).click()
    expect(dismissed).toBe(1)
    expect(document.body.contains(el)).toBe(false)
  })
})
