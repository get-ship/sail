import { describe, expect, it } from 'vitest'
import { runAction } from '../src/action'
import { tipDescribe } from '../src/actions/tip-describe'
import { createScope, withScope } from '../src/effect-scope'

function mountTooltip() {
  const host = document.createElement('span')
  host.setAttribute('content', 'Hello tip')
  host.setAttribute('placement', 'top')
  const trigger = document.createElement('button')
  trigger.textContent = 'hover me'
  const bubble = document.createElement('span')
  bubble.setAttribute('data-tooltip-bubble', '')
  host.append(trigger, bubble)
  document.body.appendChild(host)
  const scope = createScope()
  withScope(scope, () => {
    runAction(host, tipDescribe)
  })
  return { host, bubble, scope, dispose: () => { scope.dispose(); host.remove() } }
}

describe('tipDescribe action', () => {
  it('stamps role=tooltip, content text, and placement vars', () => {
    const { host, bubble, dispose } = mountTooltip()
    expect(bubble.getAttribute('role')).toBe('tooltip')
    expect(bubble.textContent).toBe('Hello tip')
    expect(host.style.getPropertyValue('--tt-bottom')).toBeTruthy()
    dispose()
  })

  it('force-hides bubble on mouseleave (no linger)', () => {
    const { host, bubble, dispose } = mountTooltip()
    host.dispatchEvent(new Event('mouseenter', { bubbles: true }))
    // Simulate CSS show then leave
    bubble.style.opacity = '1'
    host.dispatchEvent(new Event('mouseleave', { bubbles: true }))
    expect(bubble.style.opacity).toBe('0')
    expect(bubble.style.transition).toBe('none')
    dispose()
  })

  it('clears forced hide on mouseenter so CSS can show again', () => {
    const { host, bubble, dispose } = mountTooltip()
    host.dispatchEvent(new Event('mouseleave', { bubbles: true }))
    expect(bubble.style.opacity).toBe('0')
    host.dispatchEvent(new Event('mouseenter', { bubbles: true }))
    expect(bubble.style.opacity).toBe('')
    expect(bubble.style.transition).toBe('')
    dispose()
  })
})
