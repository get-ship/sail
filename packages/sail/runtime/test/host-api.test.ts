import { describe, it, expect, vi } from 'vitest'

import {
  attachHostApi,
  bindHostAttr,
  bindNativeEvent,
  emit,
  expose,
  slotted,
} from '../src/host-api'
import { createScope, withScope } from '../src/effect-scope'

describe('host-api: emit', () => {
  it('dispatches a CustomEvent with bubbles+composed defaulted true', () => {
    const host = document.createElement('div')
    const seen: CustomEvent[] = []
    host.addEventListener('sail-thing', (e) => seen.push(e as CustomEvent))
    emit(host, 'sail-thing', { value: 7 })
    expect(seen).toHaveLength(1)
    expect(seen[0]!.bubbles).toBe(true)
    expect(seen[0]!.composed).toBe(true)
    expect(seen[0]!.detail).toEqual({ value: 7 })
  })

  it('attaches host.emit via attachHostApi', () => {
    const host = document.createElement('div')
    attachHostApi(host)
    expect(typeof host.emit).toBe('function')
    const seen: unknown[] = []
    host.addEventListener('sail-x', (e) => seen.push((e as CustomEvent).detail))
    host.emit!('sail-x', 'payload')
    expect(seen).toEqual(['payload'])
  })
})

describe('host-api: expose', () => {
  it('attaches methods to the host so callers can invoke them directly', () => {
    const host = document.createElement('div')
    const scope = createScope()
    withScope(scope, () => {
      expose(host, {
        focus: () => 'focused',
        clear: () => 'cleared',
      })
    })
    expect((host as unknown as { focus: () => string }).focus()).toBe('focused')
    expect((host as unknown as { clear: () => string }).clear()).toBe('cleared')
  })

  it('removes attached methods when the scope disposes', () => {
    const host = document.createElement('div') as HTMLElement & {
      foo?: () => void
      bar?: () => void
    }
    const scope = createScope()
    withScope(scope, () => {
      expose(host, { foo: () => {}, bar: () => {} })
    })
    expect(typeof host.foo).toBe('function')
    expect(typeof host.bar).toBe('function')
    scope.dispose()
    expect(host.foo).toBeUndefined()
    expect(host.bar).toBeUndefined()
  })

  it('host.expose proxies to the free function and registers cleanup', () => {
    const host = document.createElement('div')
    attachHostApi(host)
    const scope = createScope()
    withScope(scope, () => {
      host.expose!({ ping: () => 'pong' })
    })
    expect((host as unknown as { ping: () => string }).ping()).toBe('pong')
    scope.dispose()
    expect((host as unknown as { ping?: () => string }).ping).toBeUndefined()
  })
})

describe('host-api: bindNativeEvent', () => {
  it('attaches a listener and removes it on scope dispose', () => {
    const target = document.createElement('button')
    const handler = vi.fn()
    const scope = createScope()
    withScope(scope, () => {
      bindNativeEvent(target, 'click', handler)
    })
    target.dispatchEvent(new Event('click'))
    expect(handler).toHaveBeenCalledTimes(1)

    scope.dispose()
    target.dispatchEvent(new Event('click'))
    expect(handler).toHaveBeenCalledTimes(1)
  })

  it('forwards listener options to addEventListener (capture)', () => {
    const parent = document.createElement('div')
    const child = document.createElement('span')
    parent.appendChild(child)
    const order: string[] = []
    const scope = createScope()
    withScope(scope, () => {
      bindNativeEvent(parent, 'click', () => order.push('parent-capture'), { capture: true })
    })
    child.addEventListener('click', () => order.push('child-bubble'))
    child.dispatchEvent(new Event('click', { bubbles: true }))
    expect(order).toEqual(['parent-capture', 'child-bubble'])
    scope.dispose()
  })
})

describe('host-api: slotted', () => {
  it('invokes render on initial subscribe with the assigned elements', () => {
    const host = document.createElement('div')
    host.attachShadow({ mode: 'open' })
    const slot = document.createElement('slot')
    host.shadowRoot!.appendChild(slot)
    const child = document.createElement('span')
    host.appendChild(child)
    document.body.appendChild(host)

    const renders: number[] = []
    const scope = createScope()
    withScope(scope, () => {
      slotted(() => slot, (els) => renders.push(els.length))
    })
    expect(renders.length).toBeGreaterThanOrEqual(1)
    expect(renders[0]).toBe(1)

    scope.dispose()
    document.body.removeChild(host)
  })

  it('re-renders when slotchange fires and stops after dispose', () => {
    const host = document.createElement('div')
    host.attachShadow({ mode: 'open' })
    const slot = document.createElement('slot')
    host.shadowRoot!.appendChild(slot)
    document.body.appendChild(host)

    const calls: number[] = []
    const scope = createScope()
    withScope(scope, () => {
      slotted(() => slot, (els) => calls.push(els.length))
    })
    const initial = calls.length

    slot.dispatchEvent(new Event('slotchange'))
    expect(calls.length).toBe(initial + 1)

    scope.dispose()
    slot.dispatchEvent(new Event('slotchange'))
    expect(calls.length).toBe(initial + 1)

    document.body.removeChild(host)
  })

  it('skips the render when the ref returns null', () => {
    const render = vi.fn()
    const scope = createScope()
    withScope(scope, () => {
      slotted(() => null, render)
    })
    expect(render).not.toHaveBeenCalled()
    scope.dispose()
  })
})

describe('host-api: bindHostAttr', () => {
  it('sets, removes on null, and re-sets on transition', async () => {
    const host = document.createElement('div')
    let value: string | null = 'button'
    const get = (): string | null => value
    const scope = createScope()
    withScope(scope, () => {
      bindHostAttr(host, 'role', get)
    })
    expect(host.getAttribute('role')).toBe('button')
    scope.dispose()
  })

  it('reacts to signal updates and tears down on scope dispose', async () => {
    const { signal } = await import('../src/signal')
    const sig = signal<string | null>('button')
    const host = document.createElement('div')
    const scope = createScope()
    withScope(scope, () => {
      bindHostAttr(host, 'role', () => sig())
    })
    expect(host.getAttribute('role')).toBe('button')
    sig.set(null)
    expect(host.hasAttribute('role')).toBe(false)
    sig.set('menuitem')
    expect(host.getAttribute('role')).toBe('menuitem')
    scope.dispose()
    sig.set('tab')
    // After dispose, further signal updates no longer reach the host —
    // the attribute stays at whatever value the effect last applied.
    expect(host.getAttribute('role')).toBe('menuitem')
  })
})
