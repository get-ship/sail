// Alert dismissControl — live region is compiler anatomy (tone → role/aria-live),
// not an action. See sail-compiler semantic_props::live_region_for_tone.
import { describe, expect, it, afterEach } from 'vitest'
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

describe('dismissControl (alert)', () => {
  afterEach(() => {
    document.body.replaceChildren()
  })

  it('dismissible + dismiss control → onDismiss + removed', () => {
    const el = makeAlert({ dismissible: true, tone: 'info' })
    dismissControl(el)

    let dismissed = 0
    ;(el as any).onDismiss = () => {
      dismissed++
    }

    const close = el.querySelector('[data-dismiss]') as HTMLElement
    close.click()
    expect(dismissed).toBe(1)
    expect(document.body.contains(el)).toBe(false)
  })

  it('not dismissible → close is a no-op', () => {
    const el = makeAlert({ tone: 'info' })
    dismissControl(el)

    let dismissed = 0
    ;(el as any).onDismiss = () => {
      dismissed++
    }

    const close = el.querySelector('[data-dismiss]') as HTMLElement
    close.click()
    expect(dismissed).toBe(0)
    expect(document.body.contains(el)).toBe(true)
  })
})
