import { describe, expect, it, vi } from 'vitest'
import { signal } from '../src/signal'
import { bindIf } from '../src/if'

function setup() {
  const host = document.createElement('div')
  document.body.appendChild(host)
  const anchor = document.createComment('if')
  host.appendChild(anchor)
  return { host, anchor }
}

describe('bindIf', () => {
  it('renders then-branch when cond is true', () => {
    const { host, anchor } = setup()
    const cond = signal(true)
    bindIf({
      anchor,
      cond: () => cond(),
      then_: () => ({ nodes: [document.createTextNode('YES')] }),
      else_: () => ({ nodes: [document.createTextNode('NO')] }),
    })
    expect(host.textContent).toBe('YES')
  })

  it('swaps branches on cond change', () => {
    const { host, anchor } = setup()
    const cond = signal(true)
    bindIf({
      anchor,
      cond: () => cond(),
      then_: () => ({ nodes: [document.createTextNode('YES')] }),
      else_: () => ({ nodes: [document.createTextNode('NO')] }),
    })
    cond.set(false)
    expect(host.textContent).toBe('NO')
    cond.set(true)
    expect(host.textContent).toBe('YES')
  })

  it('runs branch cleanup on swap', () => {
    const { host: _h, anchor } = setup()
    const cond = signal(true)
    const cleanup = vi.fn()
    bindIf({
      anchor,
      cond: () => cond(),
      then_: () => ({ nodes: [document.createTextNode('YES')], cleanup }),
      else_: () => ({ nodes: [document.createTextNode('NO')] }),
    })
    expect(cleanup).not.toHaveBeenCalled()
    cond.set(false)
    expect(cleanup).toHaveBeenCalledTimes(1)
  })

  it('omitting else renders nothing on false', () => {
    const { host, anchor } = setup()
    const cond = signal(false)
    bindIf({
      anchor,
      cond: () => cond(),
      then_: () => ({ nodes: [document.createTextNode('YES')] }),
    })
    expect(host.textContent).toBe('')
    cond.set(true)
    expect(host.textContent).toBe('YES')
  })

  it('disposer tears down active branch', () => {
    const { host, anchor } = setup()
    const cond = signal(true)
    const cleanup = vi.fn()
    const dispose = bindIf({
      anchor,
      cond: () => cond(),
      then_: () => ({ nodes: [document.createTextNode('YES')], cleanup }),
    })
    expect(host.textContent).toBe('YES')
    dispose()
    expect(host.textContent).toBe('')
    expect(cleanup).toHaveBeenCalledTimes(1)
  })

  it('does not re-render when cond unchanged', () => {
    const { anchor } = setup()
    const cond = signal(true)
    const factory = vi.fn(() => ({ nodes: [document.createTextNode('YES')] }))
    bindIf({ anchor, cond: () => cond(), then_: factory })
    expect(factory).toHaveBeenCalledTimes(1)
    cond.set(true)
    expect(factory).toHaveBeenCalledTimes(1)
  })
})
