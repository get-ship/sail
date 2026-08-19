import { describe, expect, it, vi } from 'vitest'
import { controller } from '../src/controller-resolver'
import { runController } from '../src/compose'
import { createScope, mount, withScope } from '../src/effect-scope'

describe('runController', () => {
  it('runs setup with the host and tears down via the disposer', () => {
    const host = document.createElement('div')
    const seen: Element[] = []
    const def = controller((_ctx, el) => {
      if (el) seen.push(el)
      return {}
    })
    const dispose = runController(host, def)
    expect(seen[0]).toBe(host)
    dispose()
  })

  it('fires mount() against the host and disposes it', () => {
    const host = document.createElement('div')
    const stop = vi.fn()
    const def = controller(() => {
      mount((el) => {
        el.setAttribute('data-on', '1')
        return stop
      })
      return {}
    })
    const root = createScope()
    let dispose = () => {}
    withScope(root, () => {
      dispose = runController(host, def)
    })
    expect(host.getAttribute('data-on')).toBe('1')
    dispose()
    expect(stop).toHaveBeenCalledTimes(1)
  })
})
